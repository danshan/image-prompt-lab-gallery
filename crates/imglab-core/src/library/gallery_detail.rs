use super::{
    assets::{ensure_asset_exists, load_version, version_summary_from_row},
    database_error,
    gallery_version_tree::{
        build_asset_version_tree, load_asset_scoped_lineage, load_promoted_source,
    },
    operation_from_str,
    storage::file_digest,
};
use crate::{
    version_name, AlbumId, AlbumKind, AssetDetailView, AssetId, AssetInspectorDetailView,
    AssetVersionId, CanonicalMetadataView, DomainResult, FileContextView, GenerationEventId,
    PendingSuggestionSummaryView, ReferenceSourceView, TaskId, TaskOriginView, TaskStatus,
    TaskType, VersionSummary,
};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub(super) fn get_asset_detail(
    library_path: &Path,
    connection: &Connection,
    asset_id: &AssetId,
    current_version_id: Option<&AssetVersionId>,
) -> DomainResult<AssetDetailView> {
    ensure_asset_exists(connection, asset_id)?;
    let asset = load_asset_detail_base(connection, asset_id)?;
    let versions = load_asset_versions(connection, asset_id)?;
    let version_tree_model = build_asset_version_tree(connection, asset_id)?;
    let current_version = current_version_id
        .map(|version_id| load_version(connection, version_id))
        .transpose()?
        .or_else(|| versions.first().cloned());
    let focused_version_tree_name = current_version
        .as_ref()
        .and_then(|version| version_tree_model.names_by_id.get(&version.id.0).cloned())
        .or_else(|| {
            current_version
                .as_ref()
                .map(|version| version.version_name.clone())
        });
    let event = current_version
        .as_ref()
        .and_then(|version| version.generation_event_id.as_ref())
        .map(|event_id| load_generation_event_detail(connection, event_id))
        .transpose()?
        .or_else(|| {
            load_latest_generation_event_detail(connection, asset_id)
                .ok()
                .flatten()
        });
    let source_reference = event
        .as_ref()
        .and_then(|event| event.input_asset_version_id.as_ref())
        .map(|version_id| load_reference_source(connection, version_id, asset_id))
        .transpose()?
        .flatten();
    let lineage = current_version
        .as_ref()
        .map(|version| load_asset_scoped_lineage(connection, version))
        .transpose()?
        .unwrap_or_default();
    let file = current_version
        .as_ref()
        .map(|version| load_file_context(library_path, connection, version))
        .transpose()?;
    let promoted_from = current_version
        .as_ref()
        .map(|version| load_promoted_source(connection, version, &version_tree_model.names_by_id))
        .transpose()?
        .flatten();

    Ok(AssetDetailView {
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
        tags: load_asset_tags(connection, asset_id)?,
        albums: load_asset_albums(connection, asset_id)?,
        review_pending_count: pending_review_count(connection, asset_id)?,
        current_version_id: current_version.as_ref().map(|version| version.id.clone()),
        current_version_number: current_version
            .as_ref()
            .map(|version| version.version_number),
        current_version_name: current_version
            .as_ref()
            .map(|version| version.version_name.clone()),
        focused_version_id: current_version.as_ref().map(|version| version.id.clone()),
        focused_version_tree_name,
        focused_version: current_version.clone(),
        versions,
        version_tree: version_tree_model.roots,
        version_tree_issues: version_tree_model.issues,
        lineage,
        source_reference,
        promoted_from,
        file,
    })
}

pub(super) fn get_asset_inspector_detail(
    library_path: &Path,
    connection: &Connection,
    asset_id: &AssetId,
    current_version_id: Option<&AssetVersionId>,
) -> DomainResult<AssetInspectorDetailView> {
    let detail = get_asset_detail(library_path, connection, asset_id, current_version_id)?;
    let pending_suggestions = load_pending_suggestion_summaries(connection, asset_id)?;
    let task_origin = detail
        .versions
        .first()
        .and_then(|version| {
            load_task_origin_for_version_or_asset(connection, &version.id, asset_id).ok()
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

fn load_task_origin_for_version_or_asset(
    connection: &Connection,
    version_id: &AssetVersionId,
    asset_id: &AssetId,
) -> DomainResult<Option<TaskOriginView>> {
    connection
        .query_row(
            "
            SELECT task_outputs.output_type, tasks.id, tasks.task_type, tasks.status,
                   tasks.provider, tasks.operation_type
            FROM task_outputs
            INNER JOIN tasks ON tasks.id = task_outputs.task_id
            WHERE (task_outputs.output_type = 'asset_version' AND task_outputs.target_id = ?1)
               OR (task_outputs.output_type = 'asset' AND task_outputs.target_id = ?2)
            ORDER BY
                CASE task_outputs.output_type
                    WHEN 'asset_version' THEN 0
                    ELSE 1
                END,
                tasks.updated_at DESC,
                tasks.id DESC
            LIMIT 1
            ",
            params![version_id.0, asset_id.0],
            |row| {
                let task_type_value: String = row.get(2)?;
                let status_value: String = row.get(3)?;
                let operation_value: Option<String> = row.get(5)?;
                let task_type = TaskType::parse(&task_type_value).ok_or_else(|| {
                    rusqlite::Error::InvalidColumnType(
                        2,
                        "task_type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let status = TaskStatus::parse(&status_value).ok_or_else(|| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "status".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let operation = operation_value
                    .as_deref()
                    .map(operation_from_str)
                    .transpose()
                    .map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                Ok(TaskOriginView {
                    task_id: TaskId(row.get(1)?),
                    task_type,
                    status,
                    provider: row.get(4)?,
                    operation,
                })
            },
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })
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
