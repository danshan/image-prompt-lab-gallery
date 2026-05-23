use super::{albums::parse_smart_query, database_error};
use crate::{
    AlbumId, AlbumKind, AlbumMembershipView, DomainError, DomainResult, GalleryAlbumFilter,
    GalleryAssetView, GalleryQuery, GallerySort, ReviewStatusFilter,
};
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

pub(super) fn effective_gallery_album_filter(query: &GalleryQuery) -> GalleryAlbumFilter {
    match &query.album_filter {
        GalleryAlbumFilter::Any => {
            GalleryAlbumFilter::from_legacy_album_id(query.album_id.clone()).normalized()
        }
        album_filter => album_filter.clone().normalized(),
    }
}

pub(super) fn validate_gallery_album_context(
    query: &GalleryQuery,
    album_filter: &GalleryAlbumFilter,
    album_contexts: &[AlbumFilterContext],
) -> DomainResult<()> {
    if query.sort != GallerySort::AlbumOrder {
        return Ok(());
    }

    let is_single_manual = matches!(
        (album_filter, album_contexts),
        (GalleryAlbumFilter::InAny(album_ids), [AlbumFilterContext::Manual(_)])
            if album_ids.len() == 1
    );
    if !is_single_manual {
        return Err(DomainError::InvalidGalleryQuery {
            message: "album_order sort requires a single manual album filter".to_string(),
        });
    }
    Ok(())
}

pub(super) fn single_album_filter_id(album_filter: &GalleryAlbumFilter) -> Option<&AlbumId> {
    match album_filter {
        GalleryAlbumFilter::InAny(album_ids) if album_ids.len() == 1 => album_ids.first(),
        GalleryAlbumFilter::Any | GalleryAlbumFilter::InAny(_) | GalleryAlbumFilter::Unassigned => {
            None
        }
    }
}

pub(super) fn single_album_filter_context<'a>(
    album_filter: &GalleryAlbumFilter,
    album_contexts: &'a [AlbumFilterContext],
) -> Option<&'a AlbumFilterContext> {
    single_album_filter_id(album_filter).and_then(|_| album_contexts.first())
}

pub(super) fn load_album_membership_context(
    connection: &Connection,
    album_id: &AlbumId,
) -> DomainResult<AlbumMembershipView> {
    connection
        .query_row(
            "SELECT id, name, kind FROM albums WHERE id = ?1",
            params![album_id.0],
            |row| {
                let kind: String = row.get(2)?;
                Ok(AlbumMembershipView {
                    id: AlbumId(row.get(0)?),
                    name: row.get(1)?,
                    kind: if kind == "smart" {
                        AlbumKind::Smart
                    } else {
                        AlbumKind::Manual
                    },
                })
            },
        )
        .map_err(database_error)
}

fn load_album_asset_ids(
    connection: &Connection,
    album_id: &AlbumId,
) -> DomainResult<HashSet<String>> {
    let mut statement = connection
        .prepare("SELECT asset_id FROM album_items WHERE album_id = ?1")
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![album_id.0], |row| row.get::<_, String>(0))
        .map_err(database_error)?;
    let mut asset_ids = HashSet::new();
    for row in rows {
        asset_ids.insert(row.map_err(database_error)?);
    }
    Ok(asset_ids)
}

pub(super) fn apply_gallery_album_filter(
    connection: &Connection,
    items: &mut Vec<GalleryAssetView>,
    album_filter: &GalleryAlbumFilter,
    album_contexts: &[AlbumFilterContext],
) -> DomainResult<()> {
    match album_filter {
        GalleryAlbumFilter::Any => {}
        GalleryAlbumFilter::InAny(_) => {
            let mut matching_asset_ids = HashSet::new();
            for context in album_contexts {
                match context {
                    AlbumFilterContext::Manual(album_id) => {
                        matching_asset_ids.extend(load_album_asset_ids(connection, album_id)?);
                    }
                    AlbumFilterContext::Smart(smart_query) => {
                        let mut smart_items = items.clone();
                        apply_smart_album_query(&mut smart_items, smart_query);
                        matching_asset_ids.extend(smart_items.into_iter().map(|item| item.id.0));
                    }
                }
            }
            items.retain(|item| matching_asset_ids.contains(&item.id.0));
        }
        GalleryAlbumFilter::Unassigned => {
            let smart_membership_ids = load_smart_album_asset_ids(connection, items)?;
            items.retain(|item| {
                item.albums.is_empty() && !smart_membership_ids.contains(&item.id.0)
            });
        }
    }
    Ok(())
}

fn load_smart_album_asset_ids(
    connection: &Connection,
    items: &[GalleryAssetView],
) -> DomainResult<HashSet<String>> {
    let mut statement = connection
        .prepare("SELECT smart_query_json FROM albums WHERE kind = 'smart'")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| row.get::<_, Option<String>>(0))
        .map_err(database_error)?;
    let mut asset_ids = HashSet::new();
    for row in rows {
        let Some(query_json) = row.map_err(database_error)? else {
            return Err(DomainError::InvalidSmartAlbumQuery {
                message: "smart album is missing query".to_string(),
            });
        };
        let smart_query = parse_smart_query(&query_json)?;
        let mut smart_items = items.to_vec();
        apply_smart_album_query(&mut smart_items, &smart_query);
        asset_ids.extend(smart_items.into_iter().map(|item| item.id.0));
    }
    Ok(asset_ids)
}

pub(super) enum AlbumFilterContext {
    Manual(AlbumId),
    Smart(crate::SmartAlbumQuery),
}

pub(super) struct GalleryFilterSpec<'a> {
    text: Option<&'a str>,
    providers: &'a [String],
    min_rating: Option<u8>,
    review_pending_only: bool,
    tags: &'a [String],
    category: Option<&'a str>,
    status: Option<&'a str>,
    created_at_from: Option<&'a str>,
    created_at_to: Option<&'a str>,
}

impl<'a> GalleryFilterSpec<'a> {
    pub(super) fn from_gallery_query(query: &'a GalleryQuery) -> Self {
        Self {
            text: query.text.as_deref(),
            providers: &query.providers,
            min_rating: query.min_rating,
            review_pending_only: query.review_status == ReviewStatusFilter::Pending,
            tags: &query.tags,
            category: None,
            status: None,
            created_at_from: None,
            created_at_to: None,
        }
    }

    fn from_smart_album_query(query: &'a crate::SmartAlbumQuery) -> Self {
        Self {
            text: query.text.as_deref(),
            providers: &query.providers,
            min_rating: query.min_rating,
            review_pending_only: query.review_status == ReviewStatusFilter::Pending,
            tags: &query.tags,
            category: query.category.as_deref(),
            status: query.status.as_deref(),
            created_at_from: query.created_at_from.as_deref(),
            created_at_to: query.created_at_to.as_deref(),
        }
    }
}

fn load_album_filter_context(
    connection: &Connection,
    album_id: &AlbumId,
) -> DomainResult<AlbumFilterContext> {
    let (kind, smart_query_json): (String, Option<String>) = connection
        .query_row(
            "SELECT kind, smart_query_json FROM albums WHERE id = ?1",
            params![album_id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => unknown_album_error(album_id),
            other => database_error(other),
        })?;
    if kind == "smart" {
        let query_json = smart_query_json.ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: "smart album is missing query".to_string(),
        })?;
        Ok(AlbumFilterContext::Smart(parse_smart_query(&query_json)?))
    } else {
        Ok(AlbumFilterContext::Manual(album_id.clone()))
    }
}

pub(super) fn load_album_filter_contexts(
    connection: &Connection,
    album_filter: &GalleryAlbumFilter,
) -> DomainResult<Vec<AlbumFilterContext>> {
    match album_filter {
        GalleryAlbumFilter::Any | GalleryAlbumFilter::Unassigned => Ok(Vec::new()),
        GalleryAlbumFilter::InAny(album_ids) => album_ids
            .iter()
            .map(|album_id| load_album_filter_context(connection, album_id))
            .collect(),
    }
}

fn unknown_album_error(album_id: &AlbumId) -> DomainError {
    DomainError::InvalidGalleryQuery {
        message: format!("unknown album id: {}", album_id.0),
    }
}

fn apply_smart_album_query(items: &mut Vec<GalleryAssetView>, query: &crate::SmartAlbumQuery) {
    apply_gallery_filter_spec(items, GalleryFilterSpec::from_smart_album_query(query));
}

pub(super) fn apply_gallery_filter_spec(
    items: &mut Vec<GalleryAssetView>,
    spec: GalleryFilterSpec<'_>,
) {
    if let Some(text) = spec.text.map(str::trim).filter(|text| !text.is_empty()) {
        let needle = text.to_ascii_lowercase();
        items.retain(|item| gallery_item_matches_text(item, &needle));
    }
    if !spec.providers.is_empty() {
        items.retain(|item| {
            item.provider
                .as_ref()
                .map(|provider| spec.providers.iter().any(|wanted| wanted == provider))
                .unwrap_or(false)
        });
    }
    if let Some(min_rating) = spec.min_rating {
        items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
    }
    if spec.review_pending_only {
        items.retain(|item| item.review_pending_count > 0);
    }
    if !spec.tags.is_empty() {
        items.retain(|item| spec.tags.iter().all(|tag| item.tags.contains(tag)));
    }
    if let Some(category) = spec.category {
        items.retain(|item| item.category.as_deref() == Some(category));
    }
    if let Some(status) = spec.status {
        items.retain(|item| item.status == status);
    }
    if let Some(from) = spec.created_at_from {
        items.retain(|item| item.created_at.as_str() >= from);
    }
    if let Some(to) = spec.created_at_to {
        items.retain(|item| item.created_at.as_str() <= to);
    }
}

pub(super) fn sort_gallery_items(
    items: &mut [GalleryAssetView],
    sort: GallerySort,
    album_context: Option<&AlbumFilterContext>,
    connection: &Connection,
) -> DomainResult<()> {
    match sort {
        GallerySort::Newest => items.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.created_at.cmp(&left.created_at))
        }),
        GallerySort::Oldest => items.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.updated_at.cmp(&right.updated_at))
        }),
        GallerySort::RatingDesc => items.sort_by(|left, right| {
            right
                .rating
                .unwrap_or_default()
                .cmp(&left.rating.unwrap_or_default())
                .then_with(|| left.title.cmp(&right.title))
        }),
        GallerySort::TitleAsc => items.sort_by(|left, right| left.title.cmp(&right.title)),
        GallerySort::ProviderAsc => items.sort_by(|left, right| left.provider.cmp(&right.provider)),
        GallerySort::AlbumOrder => {
            let Some(AlbumFilterContext::Manual(album_id)) = album_context else {
                return Err(DomainError::InvalidGalleryQuery {
                    message: "album_order sort requires a manual album filter".to_string(),
                });
            };
            let sort_orders = load_album_item_sort_orders(connection, album_id)?;
            items.sort_by(|left, right| {
                sort_orders
                    .get(&left.id.0)
                    .copied()
                    .unwrap_or(i64::MAX)
                    .cmp(&sort_orders.get(&right.id.0).copied().unwrap_or(i64::MAX))
            });
        }
    }
    Ok(())
}

pub(super) fn gallery_item_matches_text(item: &GalleryAssetView, needle: &str) -> bool {
    item.title
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains(needle)
        || item
            .category
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(needle)
        || item
            .provider
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(needle)
        || item
            .model_label
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(needle)
        || item
            .prompt
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(needle)
        || item
            .tags
            .iter()
            .any(|tag| tag.to_ascii_lowercase().contains(needle))
}

fn load_album_item_sort_orders(
    connection: &Connection,
    album_id: &AlbumId,
) -> DomainResult<HashMap<String, i64>> {
    let mut statement = connection
        .prepare("SELECT asset_id, sort_order FROM album_items WHERE album_id = ?1")
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![album_id.0], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(database_error)?;
    let mut map = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(database_error)?;
        map.insert(key, value);
    }
    Ok(map)
}
