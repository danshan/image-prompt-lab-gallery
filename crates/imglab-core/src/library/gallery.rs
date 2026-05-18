use super::{
    albums::parse_smart_query,
    assets::{load_generation_event, load_version, version_summary_from_row},
    database_error,
    storage::file_digest,
    LocalLibraryService,
};
use crate::{
    AlbumId, AlbumKind, AssetId, AssetService, AssetSummary, AssetVersionId, DomainError,
    DomainResult, FileContextView, GalleryAssetView, GalleryQuery, GalleryReadService, GallerySort,
    GenerationEventId, GenerationEventSummary, LibraryService, ReviewStatusFilter, SearchQuery,
    SearchService, VersionSummary,
};
use rusqlite::{params, Connection};
use std::path::Path;

impl SearchService for LocalLibraryService {
    fn search(
        &self,
        library_id: &crate::LibraryId,
        query: SearchQuery,
    ) -> DomainResult<Vec<AssetSummary>> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        search_assets(&connection, query)
    }
}

impl GalleryReadService for LocalLibraryService {
    fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>> {
        validate_gallery_query(&query)?;
        let connection = Self::open_library_database(library_path)?;
        let mut items = load_gallery_asset_views(&connection)?;
        let album_context = query
            .album_id
            .as_ref()
            .map(|album_id| load_album_filter_context(&connection, album_id))
            .transpose()?;

        if let Some(text) = query
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            let needle = text.to_ascii_lowercase();
            items.retain(|item| {
                item.title
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
                    || item
                        .category
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .provider
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .model_label
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .prompt
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&needle))
            });
        }

        if !query.providers.is_empty() {
            items.retain(|item| {
                item.provider
                    .as_ref()
                    .map(|provider| query.providers.iter().any(|wanted| wanted == provider))
                    .unwrap_or(false)
            });
        }

        if let Some(min_rating) = query.min_rating {
            items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
        }

        if query.review_status == ReviewStatusFilter::Pending {
            items.retain(|item| item.review_pending_count > 0);
        }

        if !query.tags.is_empty() {
            items.retain(|item| query.tags.iter().all(|tag| item.tags.contains(tag)));
        }

        if let Some(context) = &album_context {
            match context {
                AlbumFilterContext::Manual(album_id) => {
                    items.retain(|item| {
                        asset_in_album(&connection, album_id, &item.id).unwrap_or(false)
                    });
                }
                AlbumFilterContext::Smart(smart_query) => {
                    apply_smart_album_query(&mut items, smart_query)
                }
            }
        }

        let effective_sort = match &album_context {
            Some(AlbumFilterContext::Smart(smart_query)) => smart_query.sort.unwrap_or(query.sort),
            _ => query.sort,
        };

        match effective_sort {
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
            GallerySort::ProviderAsc => {
                items.sort_by(|left, right| left.provider.cmp(&right.provider))
            }
            GallerySort::AlbumOrder => {
                let Some(AlbumFilterContext::Manual(album_id)) = &album_context else {
                    return Err(DomainError::InvalidGalleryQuery {
                        message: "album_order sort requires a manual album filter".to_string(),
                    });
                };
                items.sort_by(|left, right| {
                    album_item_sort_order(&connection, album_id, &left.id)
                        .unwrap_or(i64::MAX)
                        .cmp(
                            &album_item_sort_order(&connection, album_id, &right.id)
                                .unwrap_or(i64::MAX),
                        )
                });
            }
        }

        Ok(items)
    }

    fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<crate::AssetDetailView> {
        let connection = Self::open_library_database(library_path)?;
        super::assets::ensure_asset_exists(&connection, asset_id)?;
        let asset = load_asset_detail_base(&connection, asset_id)?;
        let versions = load_asset_versions(&connection, asset_id)?;
        let current_version = current_version_id
            .map(|version_id| load_version(&connection, version_id))
            .transpose()?
            .or_else(|| versions.first().cloned());
        let event = current_version
            .as_ref()
            .and_then(|version| version.generation_event_id.as_ref())
            .map(|event_id| load_generation_event_detail(&connection, event_id))
            .transpose()?
            .or_else(|| {
                load_latest_generation_event_detail(&connection, asset_id)
                    .ok()
                    .flatten()
            });
        let lineage = current_version
            .as_ref()
            .map(|version| self.get_lineage(library_path, &version.id))
            .transpose()?
            .unwrap_or_default();
        let file = current_version
            .as_ref()
            .map(|version| load_file_context(library_path, &connection, version))
            .transpose()?;

        Ok(crate::AssetDetailView {
            id: asset.id,
            title: asset.title,
            description: asset.description,
            schema_prompt: asset.schema_prompt,
            category: asset.category,
            rating: asset.rating,
            status: asset.status,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            negative_prompt: event
                .as_ref()
                .and_then(|event| event.negative_prompt.clone()),
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.provider_model.clone()),
            parameters_json: event.as_ref().map(|event| event.parameters_json.clone()),
            tags: load_asset_tags(&connection, asset_id)?,
            albums: load_asset_albums(&connection, asset_id)?,
            review_pending_count: pending_review_count(&connection, asset_id)?,
            versions,
            lineage,
            file,
        })
    }
}

pub(super) fn validate_rating_range(
    value: u8,
    field: &str,
    gallery_query: bool,
) -> DomainResult<()> {
    if (1..=5).contains(&value) {
        return Ok(());
    }

    let message = format!("{field} must be between 1 and 5");
    if gallery_query {
        Err(DomainError::InvalidGalleryQuery { message })
    } else {
        Err(DomainError::InvalidGenerationParameters { message })
    }
}

fn search_assets(connection: &Connection, query: SearchQuery) -> DomainResult<Vec<AssetSummary>> {
    let mut assets = load_all_asset_summaries(connection)?;
    if let Some(text) = query.text {
        let needle = text.to_ascii_lowercase();
        assets.retain(|asset| asset_matches_search_text(connection, &asset.id, asset, &needle));
    }
    if let Some(provider) = query.provider {
        assets.retain(|asset| {
            load_latest_generation_event(connection, &asset.id)
                .ok()
                .flatten()
                .map(|event| event.provider == provider)
                .unwrap_or(false)
        });
    }
    if let Some(min_rating) = query.min_rating {
        assets.retain(|asset| asset.rating.unwrap_or_default() >= min_rating);
    }
    if let Some(status) = query.status {
        assets.retain(|asset| asset.status == status);
    }
    if let Some(category) = query.category {
        assets.retain(|asset| asset.category.as_deref() == Some(category.as_str()));
    }
    if !query.tags.is_empty() {
        let wanted = query.tags;
        assets.retain(|asset| asset_has_all_tags(connection, &asset.id, &wanted).unwrap_or(false));
    }
    Ok(assets)
}

fn asset_matches_search_text(
    connection: &Connection,
    asset_id: &AssetId,
    asset: &AssetSummary,
    needle: &str,
) -> bool {
    if [
        asset.title.as_deref(),
        asset.category.as_deref(),
        Some(asset.status.as_str()),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_ascii_lowercase().contains(needle))
    {
        return true;
    }

    if load_asset_tags(connection, asset_id)
        .map(|tags| {
            tags.iter()
                .any(|tag| tag.to_ascii_lowercase().contains(needle))
        })
        .unwrap_or(false)
    {
        return true;
    }

    load_latest_generation_event(connection, asset_id)
        .ok()
        .flatten()
        .map(|event| {
            [
                event.provider.as_str(),
                event.provider_model.as_str(),
                event.prompt.as_str(),
            ]
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains(needle))
        })
        .unwrap_or(false)
}

fn validate_gallery_query(query: &GalleryQuery) -> DomainResult<()> {
    if let Some(min_rating) = query.min_rating {
        validate_rating_range(min_rating, "min_rating", true)?;
    }
    if query.sort == GallerySort::AlbumOrder && query.album_id.is_none() {
        return Err(DomainError::InvalidGalleryQuery {
            message: "album_order sort requires an album filter".to_string(),
        });
    }
    Ok(())
}

fn load_gallery_asset_views(connection: &Connection) -> DomainResult<Vec<GalleryAssetView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, title, category, rating, status, created_at, updated_at
            FROM assets
            ORDER BY updated_at DESC, created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                AssetId(row.get::<_, String>(0)?),
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(database_error)?;

    let mut items = Vec::new();
    for row in rows {
        let (id, title, category, rating, status, created_at, updated_at) =
            row.map_err(database_error)?;
        let current_version = load_latest_asset_version(connection, &id)?;
        let event = current_version
            .as_ref()
            .and_then(|version| version.generation_event_id.as_ref())
            .map(|event_id| load_generation_event(connection, event_id))
            .transpose()?
            .or_else(|| load_latest_generation_event(connection, &id).ok().flatten());
        let version_count = count_asset_versions(connection, &id)?;
        let version_label = current_version
            .as_ref()
            .and_then(|version| load_version_label(connection, &version.id).ok().flatten());
        let (width, height) = current_version
            .as_ref()
            .map(|version| load_version_dimensions(connection, &version.id))
            .transpose()?
            .unwrap_or((None, None));

        items.push(GalleryAssetView {
            id: id.clone(),
            title,
            category,
            rating,
            status,
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.provider_model.clone()),
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            tags: load_asset_tags(connection, &id)?,
            review_pending_count: pending_review_count(connection, &id)?,
            current_version_id: current_version.as_ref().map(|version| version.id.clone()),
            image_path: current_version
                .as_ref()
                .map(|version| version.file_path.clone()),
            width,
            height,
            version_label,
            version_count,
            created_at,
            updated_at,
        });
    }

    Ok(items)
}

#[derive(Debug, Clone)]
struct AssetDetailBase {
    id: AssetId,
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct GenerationEventDetail {
    provider: String,
    provider_model: String,
    prompt: String,
    negative_prompt: Option<String>,
    parameters_json: String,
}

fn load_asset_detail_base(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<AssetDetailBase> {
    connection
        .query_row(
            "
            SELECT id, title, description, schema_prompt, category, rating, status, created_at, updated_at
            FROM assets
            WHERE id = ?1
            ",
            params![asset_id.0],
            |row| {
                Ok(AssetDetailBase {
                    id: AssetId(row.get(0)?),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    schema_prompt: row.get(3)?,
                    category: row.get(4)?,
                    rating: row.get::<_, Option<u8>>(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(database_error)
}

fn load_generation_event_detail(
    connection: &Connection,
    event_id: &GenerationEventId,
) -> DomainResult<GenerationEventDetail> {
    connection
        .query_row(
            "
            SELECT provider, provider_model, prompt, negative_prompt, parameters_json
            FROM generation_events
            WHERE id = ?1
            ",
            params![event_id.0],
            |row| {
                Ok(GenerationEventDetail {
                    provider: row.get(0)?,
                    provider_model: row.get(1)?,
                    prompt: row.get(2)?,
                    negative_prompt: row.get(3)?,
                    parameters_json: row.get(4)?,
                })
            },
        )
        .map_err(database_error)
}

fn load_latest_generation_event(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<GenerationEventSummary>> {
    load_latest_generation_event_id(connection, asset_id)?
        .map(|id| load_generation_event(connection, &GenerationEventId(id)))
        .transpose()
}

fn load_latest_generation_event_detail(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<GenerationEventDetail>> {
    load_latest_generation_event_id(connection, asset_id)?
        .map(|id| load_generation_event_detail(connection, &GenerationEventId(id)))
        .transpose()
}

fn load_latest_generation_event_id(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<String>> {
    connection
        .query_row(
            "
            SELECT id
            FROM generation_events
            WHERE asset_id = ?1
            ORDER BY started_at DESC
            LIMIT 1
            ",
            params![asset_id.0],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })
}

fn load_asset_versions(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<VersionSummary>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path,
                   sha256, checksum_algorithm, COALESCE(checksum, sha256), mime_type
            FROM asset_versions
            WHERE asset_id = ?1
            ORDER BY created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], version_summary_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_latest_asset_version(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<VersionSummary>> {
    let version_id = connection
        .query_row(
            "
            SELECT id
            FROM asset_versions
            WHERE asset_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            ",
            params![asset_id.0],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })?;
    version_id
        .map(|id| load_version(connection, &AssetVersionId(id)))
        .transpose()
}

fn count_asset_versions(connection: &Connection, asset_id: &AssetId) -> DomainResult<u32> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM asset_versions WHERE asset_id = ?1",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count.max(0) as u32)
}

fn load_version_label(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<Option<String>> {
    connection
        .query_row(
            "SELECT version_label FROM asset_versions WHERE id = ?1",
            params![version_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)
}

fn load_version_dimensions(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<(Option<u32>, Option<u32>)> {
    connection
        .query_row(
            "SELECT width, height FROM asset_versions WHERE id = ?1",
            params![version_id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(database_error)
}

fn load_asset_tags(connection: &Connection, asset_id: &AssetId) -> DomainResult<Vec<String>> {
    let mut statement = connection
        .prepare(
            "
            SELECT t.name
            FROM asset_tags at
            INNER JOIN tags t ON t.id = at.tag_id
            WHERE at.asset_id = ?1
            ORDER BY t.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| row.get::<_, String>(0))
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_asset_albums(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<crate::AlbumMembershipView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT a.id, a.name, a.kind
            FROM album_items ai
            INNER JOIN albums a ON a.id = ai.album_id
            WHERE ai.asset_id = ?1
            ORDER BY a.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| {
            let kind: String = row.get(2)?;
            Ok(crate::AlbumMembershipView {
                id: AlbumId(row.get(0)?),
                name: row.get(1)?,
                kind: if kind == "smart" {
                    AlbumKind::Smart
                } else {
                    AlbumKind::Manual
                },
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn pending_review_count(connection: &Connection, asset_id: &AssetId) -> DomainResult<u32> {
    let count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM metadata_suggestions
            WHERE asset_id = ?1 AND status = 'pending_review'
            ",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count.max(0) as u32)
}

fn asset_in_album(
    connection: &Connection,
    album_id: &AlbumId,
    asset_id: &AssetId,
) -> DomainResult<bool> {
    let count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM album_items
            WHERE album_id = ?1 AND asset_id = ?2
            ",
            params![album_id.0, asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count > 0)
}

enum AlbumFilterContext {
    Manual(AlbumId),
    Smart(crate::SmartAlbumQuery),
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
        .map_err(database_error)?;
    if kind == "smart" {
        let query_json = smart_query_json.ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: "smart album is missing query".to_string(),
        })?;
        Ok(AlbumFilterContext::Smart(parse_smart_query(&query_json)?))
    } else {
        Ok(AlbumFilterContext::Manual(album_id.clone()))
    }
}

fn apply_smart_album_query(items: &mut Vec<GalleryAssetView>, query: &crate::SmartAlbumQuery) {
    if let Some(text) = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        let needle = text.to_ascii_lowercase();
        items.retain(|item| gallery_item_matches_text(item, &needle));
    }
    if !query.providers.is_empty() {
        items.retain(|item| {
            item.provider
                .as_ref()
                .map(|provider| query.providers.iter().any(|wanted| wanted == provider))
                .unwrap_or(false)
        });
    }
    if let Some(min_rating) = query.min_rating {
        items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
    }
    if query.review_status == ReviewStatusFilter::Pending {
        items.retain(|item| item.review_pending_count > 0);
    }
    if !query.tags.is_empty() {
        items.retain(|item| query.tags.iter().all(|tag| item.tags.contains(tag)));
    }
    if let Some(category) = query.category.as_deref() {
        items.retain(|item| item.category.as_deref() == Some(category));
    }
    if let Some(status) = query.status.as_deref() {
        items.retain(|item| item.status == status);
    }
    if let Some(from) = query.created_at_from.as_deref() {
        items.retain(|item| item.created_at.as_str() >= from);
    }
    if let Some(to) = query.created_at_to.as_deref() {
        items.retain(|item| item.created_at.as_str() <= to);
    }
}

fn gallery_item_matches_text(item: &GalleryAssetView, needle: &str) -> bool {
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

fn album_item_sort_order(
    connection: &Connection,
    album_id: &AlbumId,
    asset_id: &AssetId,
) -> DomainResult<i64> {
    connection
        .query_row(
            "SELECT sort_order FROM album_items WHERE album_id = ?1 AND asset_id = ?2",
            params![album_id.0, asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)
}

fn load_file_context(
    library_path: &Path,
    connection: &Connection,
    version: &VersionSummary,
) -> DomainResult<FileContextView> {
    let absolute_path = library_path.join(&version.file_path);
    let size_bytes = absolute_path.metadata().ok().map(|metadata| metadata.len());
    let integrity_status = if absolute_path.is_file() {
        let actual_checksum = file_digest(&absolute_path, &version.checksum_algorithm)?;
        if actual_checksum == version.checksum {
            "verified"
        } else {
            "hash_mismatch"
        }
    } else {
        "missing"
    };
    let (width, height): (Option<u32>, Option<u32>) = connection
        .query_row(
            "SELECT width, height FROM asset_versions WHERE id = ?1",
            params![version.id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(database_error)?;

    Ok(FileContextView {
        filename: version
            .file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        relative_location: version.file_path.clone(),
        mime_type: version.mime_type.clone(),
        size_bytes,
        width,
        height,
        checksum_algorithm: version.checksum_algorithm.clone(),
        checksum: version.checksum.clone(),
        integrity_status: integrity_status.to_string(),
    })
}

fn load_all_asset_summaries(connection: &Connection) -> DomainResult<Vec<AssetSummary>> {
    let mut statement = connection
        .prepare("SELECT id, title, category, rating, status FROM assets ORDER BY created_at")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetSummary {
                id: AssetId(row.get(0)?),
                title: row.get(1)?,
                category: row.get(2)?,
                rating: row.get::<_, Option<u8>>(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn asset_has_all_tags(
    connection: &Connection,
    asset_id: &AssetId,
    tags: &[String],
) -> DomainResult<bool> {
    for tag in tags {
        let count: i64 = connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM asset_tags at
                INNER JOIN tags t ON t.id = at.tag_id
                WHERE at.asset_id = ?1 AND t.name = ?2
                ",
                params![asset_id.0, tag],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if count == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}
