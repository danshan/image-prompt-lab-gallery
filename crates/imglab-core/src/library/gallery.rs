use super::{
    albums::parse_smart_query,
    assets::{load_version, version_summary_from_row},
    database_error,
    storage::file_digest,
    LocalLibraryService,
};
use crate::{
    version_name, AlbumId, AlbumKind, AlbumMembershipView, AssetId, AssetInspectorDetailView,
    AssetService, AssetSummary, AssetVersionId, CanonicalMetadataView, DomainError, DomainResult,
    FileContextView, GalleryAssetView, GalleryQuery, GalleryReadService, GallerySort,
    GenerationEventId, LibraryService, PendingSuggestionSummaryView, ReferenceSourceView,
    ReviewStatusFilter, SearchQuery, SearchService, TaskId, TaskOriginView, TaskStatus, TaskType,
    VersionSummary,
};
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
        let album_context_view = query
            .album_id
            .as_ref()
            .map(|album_id| load_album_membership_context(&connection, album_id))
            .transpose()?;

        apply_gallery_filter_spec(&mut items, GalleryFilterSpec::from_gallery_query(&query));

        if let Some(context) = &album_context {
            match context {
                AlbumFilterContext::Manual(album_id) => {
                    let album_assets = load_album_asset_ids(&connection, album_id)?;
                    items.retain(|item| album_assets.contains(&item.id.0));
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

        sort_gallery_items(&mut items, effective_sort, &album_context, &connection)?;

        if let Some(album) = album_context_view {
            for item in &mut items {
                item.album_context = Some(album.clone());
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
        let source_reference = event
            .as_ref()
            .and_then(|event| event.input_asset_version_id.as_ref())
            .map(|version_id| load_reference_source(&connection, version_id, asset_id))
            .transpose()?
            .flatten();
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
            current_version_id: current_version.as_ref().map(|version| version.id.clone()),
            current_version_number: current_version
                .as_ref()
                .map(|version| version.version_number),
            current_version_name: current_version
                .as_ref()
                .map(|version| version.version_name.clone()),
            versions,
            lineage,
            source_reference,
            file,
        })
    }

    fn get_asset_inspector_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetInspectorDetailView> {
        let detail = self.get_asset_detail(library_path, asset_id, current_version_id)?;
        let connection = Self::open_library_database(library_path)?;
        let pending_suggestions = load_pending_suggestion_summaries(&connection, asset_id)?;
        let task_origin = detail
            .versions
            .first()
            .and_then(|version| {
                load_task_origin_for_version_or_asset(&connection, &version.id, asset_id).ok()
            })
            .flatten();

        Ok(AssetInspectorDetailView {
            canonical_metadata: CanonicalMetadataView {
                title: detail.title.clone(),
                description: detail.description.clone(),
                schema_prompt: detail.schema_prompt.clone(),
                category: detail.category.clone(),
                rating: detail.rating,
                tags: detail.tags.clone(),
                status: detail.status.clone(),
            },
            asset: detail,
            pending_suggestions,
            generated_task_origin: task_origin,
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
    let mut items = load_gallery_asset_views(connection)?;
    if let Some(text) = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        let needle = text.to_ascii_lowercase();
        items.retain(|item| search_item_matches_text(item, &needle));
    }
    if let Some(provider) = query.provider {
        items.retain(|item| item.provider.as_deref() == Some(provider.as_str()));
    }
    if let Some(min_rating) = query.min_rating {
        items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
    }
    if let Some(status) = query.status {
        items.retain(|item| item.status == status);
    }
    if let Some(category) = query.category {
        items.retain(|item| item.category.as_deref() == Some(category.as_str()));
    }
    if !query.tags.is_empty() {
        let wanted = query.tags;
        items.retain(|item| wanted.iter().all(|tag| item.tags.contains(tag)));
    }
    Ok(items
        .into_iter()
        .map(|item| AssetSummary {
            id: item.id,
            title: item.title,
            category: item.category,
            rating: item.rating,
            status: item.status,
        })
        .collect())
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
    let versions = load_latest_asset_versions(connection)?;
    let events = load_gallery_events(connection)?;
    let version_counts = load_asset_version_counts(connection)?;
    let tags = load_all_asset_tags(connection)?;
    let review_counts = load_pending_review_counts(connection)?;
    let task_origins = load_task_origins(connection)?;
    let albums = load_all_asset_album_memberships(connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, title, category, rating, status, created_at, updated_at
            FROM assets
            WHERE status <> 'reference'
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
        let current_version = versions.get(&id.0);
        let event = current_version
            .and_then(|version| version.generation_event_id.as_ref())
            .and_then(|event_id| events.by_id.get(&event_id.0))
            .or_else(|| events.latest_by_asset.get(&id.0));
        let task_origin = current_version
            .and_then(|version| task_origins.by_version.get(&version.id.0))
            .or_else(|| task_origins.by_asset.get(&id.0))
            .cloned();

        items.push(GalleryAssetView {
            id: id.clone(),
            title,
            category,
            rating,
            status,
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.model_label.clone()),
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            tags: tags.get(&id.0).cloned().unwrap_or_default(),
            review_pending_count: review_counts.get(&id.0).copied().unwrap_or_default(),
            current_version_id: current_version.as_ref().map(|version| version.id.clone()),
            current_version_number: current_version.map(|version| version.version_number),
            current_version_name: current_version.map(|version| version.version_name.clone()),
            image_path: current_version
                .as_ref()
                .map(|version| version.file_path.clone()),
            width: current_version.and_then(|version| version.width),
            height: current_version.and_then(|version| version.height),
            version_label: current_version.and_then(|version| version.version_label.clone()),
            version_count: version_counts.get(&id.0).copied().unwrap_or_default(),
            task_origin,
            albums: albums.get(&id.0).cloned().unwrap_or_default(),
            album_context: None,
            created_at,
            updated_at,
        });
    }

    Ok(items)
}

#[derive(Debug, Default)]
struct TaskOrigins {
    by_asset: HashMap<String, TaskOriginView>,
    by_version: HashMap<String, TaskOriginView>,
}

fn load_task_origins(connection: &Connection) -> DomainResult<TaskOrigins> {
    let mut statement = connection
        .prepare(
            "
            SELECT task_outputs.output_type, task_outputs.target_id, tasks.id,
                   tasks.task_type, tasks.status, tasks.provider, tasks.operation_type
            FROM task_outputs
            INNER JOIN tasks ON tasks.id = task_outputs.task_id
            WHERE task_outputs.output_type IN ('asset', 'asset_version')
            ORDER BY tasks.updated_at DESC, tasks.id DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let output_type: String = row.get(0)?;
            let task_type_value: String = row.get(3)?;
            let status_value: String = row.get(4)?;
            let operation_value: Option<String> = row.get(6)?;
            let task_type = TaskType::parse(&task_type_value).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    3,
                    "task_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let status = TaskStatus::parse(&status_value).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "status".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let operation = operation_value
                .as_deref()
                .map(super::operation_from_str)
                .transpose()
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
            Ok((
                output_type,
                row.get::<_, String>(1)?,
                TaskOriginView {
                    task_id: TaskId(row.get(2)?),
                    task_type,
                    status,
                    provider: row.get(5)?,
                    operation,
                },
            ))
        })
        .map_err(database_error)?;

    let mut origins = TaskOrigins::default();
    for row in rows {
        let (output_type, target_id, origin) = row.map_err(database_error)?;
        match output_type.as_str() {
            "asset" => {
                origins.by_asset.entry(target_id).or_insert(origin);
            }
            "asset_version" => {
                origins.by_version.entry(target_id).or_insert(origin);
            }
            _ => {}
        }
    }
    Ok(origins)
}

fn load_task_origin_for_version_or_asset(
    connection: &Connection,
    version_id: &AssetVersionId,
    asset_id: &AssetId,
) -> DomainResult<Option<TaskOriginView>> {
    let origins = load_task_origins(connection)?;
    Ok(origins
        .by_version
        .get(&version_id.0)
        .or_else(|| origins.by_asset.get(&asset_id.0))
        .cloned())
}

#[derive(Debug, Clone)]
struct LatestVersionView {
    id: AssetVersionId,
    generation_event_id: Option<GenerationEventId>,
    file_path: PathBuf,
    width: Option<u32>,
    height: Option<u32>,
    version_number: u32,
    version_name: String,
    version_label: Option<String>,
}

#[derive(Debug, Clone)]
struct GalleryEventView {
    provider: String,
    model_label: String,
    prompt: String,
}

#[derive(Debug, Default)]
struct GalleryEvents {
    by_id: HashMap<String, GalleryEventView>,
    latest_by_asset: HashMap<String, GalleryEventView>,
}

fn load_latest_asset_versions(
    connection: &Connection,
) -> DomainResult<HashMap<String, LatestVersionView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT av.id, av.asset_id, av.generation_event_id, av.file_path,
                   av.width, av.height, av.version_number, av.version_label
            FROM asset_versions av
            WHERE NOT EXISTS (
                SELECT 1
                FROM asset_versions newer
                WHERE newer.asset_id = av.asset_id
                  AND (
                    newer.created_at > av.created_at
                    OR (newer.created_at = av.created_at AND newer.id > av.id)
                  )
            )
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let version_number: u32 = row.get(6)?;
            Ok((
                row.get::<_, String>(1)?,
                LatestVersionView {
                    id: AssetVersionId(row.get(0)?),
                    generation_event_id: row.get::<_, Option<String>>(2)?.map(GenerationEventId),
                    file_path: PathBuf::from(row.get::<_, String>(3)?),
                    width: row.get(4)?,
                    height: row.get(5)?,
                    version_number,
                    version_name: version_name(version_number),
                    version_label: row.get(7)?,
                },
            ))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn load_gallery_events(connection: &Connection) -> DomainResult<GalleryEvents> {
    let mut events = GalleryEvents::default();

    let mut by_id_statement = connection
        .prepare("SELECT id, provider, provider_model, prompt FROM generation_events")
        .map_err(database_error)?;
    let by_id_rows = by_id_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                GalleryEventView {
                    provider: row.get(1)?,
                    model_label: row.get(2)?,
                    prompt: row.get(3)?,
                },
            ))
        })
        .map_err(database_error)?;
    events.by_id = collect_string_map(by_id_rows)?;

    let mut latest_statement = connection
        .prepare(
            "
            SELECT ge.asset_id, ge.provider, ge.provider_model, ge.prompt
            FROM generation_events ge
            WHERE ge.asset_id IS NOT NULL
              AND NOT EXISTS (
                SELECT 1
                FROM generation_events newer
                WHERE newer.asset_id = ge.asset_id
                  AND (
                    newer.started_at > ge.started_at
                    OR (newer.started_at = ge.started_at AND newer.id > ge.id)
                  )
              )
            ",
        )
        .map_err(database_error)?;
    let latest_rows = latest_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                GalleryEventView {
                    provider: row.get(1)?,
                    model_label: row.get(2)?,
                    prompt: row.get(3)?,
                },
            ))
        })
        .map_err(database_error)?;
    events.latest_by_asset = collect_string_map(latest_rows)?;
    Ok(events)
}

fn load_asset_version_counts(connection: &Connection) -> DomainResult<HashMap<String, u32>> {
    let mut statement = connection
        .prepare("SELECT asset_id, COUNT(*) FROM asset_versions GROUP BY asset_id")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count.max(0) as u32))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn load_all_asset_tags(connection: &Connection) -> DomainResult<HashMap<String, Vec<String>>> {
    let mut statement = connection
        .prepare(
            "
            SELECT at.asset_id, t.name
            FROM asset_tags at
            INNER JOIN tags t ON t.id = at.tag_id
            ORDER BY at.asset_id, t.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(database_error)?;
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (asset_id, tag) = row.map_err(database_error)?;
        tags.entry(asset_id).or_default().push(tag);
    }
    Ok(tags)
}

fn load_pending_review_counts(connection: &Connection) -> DomainResult<HashMap<String, u32>> {
    let mut statement = connection
        .prepare(
            "
            SELECT asset_id, COUNT(*)
            FROM metadata_suggestions
            WHERE status = 'pending_review'
            GROUP BY asset_id
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count.max(0) as u32))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn collect_string_map<T>(
    rows: impl Iterator<Item = rusqlite::Result<(String, T)>>,
) -> DomainResult<HashMap<String, T>> {
    let mut map = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(database_error)?;
        map.insert(key, value);
    }
    Ok(map)
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
    input_asset_version_id: Option<AssetVersionId>,
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
            SELECT provider, provider_model, prompt, negative_prompt, parameters_json,
                   input_asset_version_id
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
                    input_asset_version_id: row.get::<_, Option<String>>(5)?.map(AssetVersionId),
                })
            },
        )
        .map_err(database_error)
}

fn load_reference_source(
    connection: &Connection,
    version_id: &AssetVersionId,
    current_asset_id: &AssetId,
) -> DomainResult<Option<ReferenceSourceView>> {
    connection
        .query_row(
            "
            SELECT a.id, a.title, a.status, av.id, av.version_number, av.file_path
            FROM asset_versions av
            INNER JOIN assets a ON a.id = av.asset_id
            WHERE av.id = ?1
            ",
            params![version_id.0],
            |row| {
                let asset_id = AssetId(row.get(0)?);
                if asset_id == *current_asset_id {
                    return Ok(None);
                }
                let version_number: u32 = row.get(4)?;
                Ok(Some(ReferenceSourceView {
                    asset_id,
                    asset_title: row.get(1)?,
                    asset_status: row.get(2)?,
                    version_id: AssetVersionId(row.get(3)?),
                    version_number,
                    version_name: version_name(version_number),
                    file_path: PathBuf::from(row.get::<_, String>(5)?),
                }))
            },
        )
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })
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
                   sha256, checksum_algorithm, COALESCE(checksum, sha256), mime_type,
                   version_number
            FROM asset_versions
            WHERE asset_id = ?1
            ORDER BY version_number DESC, created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], version_summary_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
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

fn load_all_asset_album_memberships(
    connection: &Connection,
) -> DomainResult<HashMap<String, Vec<AlbumMembershipView>>> {
    let mut statement = connection
        .prepare(
            "
            SELECT ai.asset_id, a.id, a.name, a.kind
            FROM album_items ai
            INNER JOIN albums a ON a.id = ai.album_id
            ORDER BY ai.asset_id, a.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let kind: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                AlbumMembershipView {
                    id: AlbumId(row.get(1)?),
                    name: row.get(2)?,
                    kind: if kind == "smart" {
                        AlbumKind::Smart
                    } else {
                        AlbumKind::Manual
                    },
                },
            ))
        })
        .map_err(database_error)?;

    let mut albums: HashMap<String, Vec<AlbumMembershipView>> = HashMap::new();
    for row in rows {
        let (asset_id, album) = row.map_err(database_error)?;
        albums.entry(asset_id).or_default().push(album);
    }
    Ok(albums)
}

fn load_album_membership_context(
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

fn load_pending_suggestion_summaries(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<PendingSuggestionSummaryView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, suggested_title, suggested_tags_json, suggested_category, created_at
            FROM metadata_suggestions
            WHERE asset_id = ?1 AND status = 'pending_review'
            ORDER BY created_at DESC, id
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| {
            let tags_json: String = row.get(3)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            Ok(PendingSuggestionSummaryView {
                id: crate::MetadataSuggestionId(row.get(0)?),
                asset_id: AssetId(row.get(1)?),
                title: row.get(2)?,
                tag_count: tags.len() as u32,
                category: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
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

enum AlbumFilterContext {
    Manual(AlbumId),
    Smart(crate::SmartAlbumQuery),
}

struct GalleryFilterSpec<'a> {
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
    fn from_gallery_query(query: &'a GalleryQuery) -> Self {
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
    apply_gallery_filter_spec(items, GalleryFilterSpec::from_smart_album_query(query));
}

fn apply_gallery_filter_spec(items: &mut Vec<GalleryAssetView>, spec: GalleryFilterSpec<'_>) {
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

fn sort_gallery_items(
    items: &mut [GalleryAssetView],
    sort: GallerySort,
    album_context: &Option<AlbumFilterContext>,
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

fn search_item_matches_text(item: &GalleryAssetView, needle: &str) -> bool {
    gallery_item_matches_text(item, needle) || item.status.to_ascii_lowercase().contains(needle)
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
    collect_string_map(rows)
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
