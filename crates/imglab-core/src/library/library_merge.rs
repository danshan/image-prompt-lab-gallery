use super::{
    database_error, extension_for_mime_type, io_error, managed_original_path, timestamp_string,
    LibraryManifest, LocalLibraryService, CURRENT_SCHEMA_VERSION, DATABASE_FILE, MANIFEST_FILE,
    REQUIRED_DIRS,
};
use crate::{
    AssetVersionId, DomainError, DomainResult, LibraryId, MergeLibraryRequest, MergeLibrarySummary,
};
use rusqlite::{params, Connection, OpenFlags};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

impl LocalLibraryService {
    pub fn dry_run_merge_library(
        &self,
        request: MergeLibraryRequest,
    ) -> DomainResult<MergeLibrarySummary> {
        let plan = MergePlan::load(&request)?;
        Ok(plan.summary)
    }

    pub fn merge_library(&self, request: MergeLibraryRequest) -> DomainResult<MergeLibrarySummary> {
        let plan = MergePlan::load(&request)?;
        let copied_files = copy_version_files(&plan)?;
        let write_result = write_merge_rows(&plan);
        if write_result.is_err() {
            for path in copied_files {
                let _ = fs::remove_file(path);
            }
        }
        write_result?;
        Ok(plan.summary)
    }
}

struct MergePlan {
    request: MergeLibraryRequest,
    target_manifest: LibraryManifest,
    summary: MergeLibrarySummary,
    assets: Vec<AssetRow>,
    versions: Vec<VersionRow>,
    generation_events: Vec<GenerationEventRow>,
    prompts: Vec<PromptDocumentRow>,
    prompt_versions: Vec<PromptVersionRow>,
    suggestions: Vec<MetadataSuggestionRow>,
    tags: Vec<TagRow>,
    asset_tags: Vec<AssetTagRow>,
    albums: Vec<AlbumRow>,
    album_items: Vec<AlbumItemRow>,
    version_sources: Vec<AssetVersionSourceRow>,
    asset_ids: BTreeMap<String, String>,
    version_ids: BTreeMap<String, String>,
    event_ids: BTreeMap<String, String>,
    prompt_ids: BTreeMap<String, String>,
    prompt_version_ids: BTreeMap<String, String>,
    suggestion_ids: BTreeMap<String, String>,
    album_ids: BTreeMap<String, String>,
    tag_ids: BTreeMap<String, String>,
    copied_paths: BTreeMap<String, PathBuf>,
}

impl MergePlan {
    fn load(request: &MergeLibraryRequest) -> DomainResult<Self> {
        validate_merge_layout(&request.target_library_path)?;
        validate_merge_layout(&request.source_library_path)?;
        let target_manifest = LocalLibraryService::read_manifest(&request.target_library_path)?;
        let source_manifest = LocalLibraryService::read_manifest(&request.source_library_path)?;
        ensure_supported_manifest(&target_manifest)?;
        ensure_supported_manifest(&source_manifest)?;
        if target_manifest.id == source_manifest.id
            || request.target_library_path == request.source_library_path
        {
            return Err(DomainError::InvalidLibraryBackup {
                message: "source library must be different from target library".to_string(),
            });
        }

        let source = open_read_only_database(&request.source_library_path)?;
        let target = LocalLibraryService::open_library_database(&request.target_library_path)?;
        ensure_supported_database(&source)?;
        ensure_supported_database(&target)?;

        let assets = load_assets(&source)?;
        let versions = load_versions(&source)?;
        let generation_events = load_generation_events(&source)?;
        let prompts = load_prompt_documents(&source)?;
        let prompt_versions = load_prompt_versions(&source)?;
        let suggestions = load_metadata_suggestions(&source)?;
        let tags = load_tags(&source)?;
        let asset_tags = load_asset_tags(&source)?;
        let albums = load_albums(&source)?;
        let album_items = load_album_items(&source)?;
        let version_sources = load_asset_version_sources(&source)?;
        let runtime_rows = count_runtime_rows(&source)?;
        let existing_target_tags = load_target_tags_by_name(&target)?;

        let asset_ids = id_map(assets.iter().map(|row| row.id.as_str()));
        let version_ids = id_map(versions.iter().map(|row| row.id.as_str()));
        let event_ids = id_map(generation_events.iter().map(|row| row.id.as_str()));
        let prompt_ids = id_map(prompts.iter().map(|row| row.id.as_str()));
        let prompt_version_ids = id_map(prompt_versions.iter().map(|row| row.id.as_str()));
        let suggestion_ids = id_map(suggestions.iter().map(|row| row.id.as_str()));
        let album_ids = id_map(albums.iter().map(|row| row.id.as_str()));
        let mut tag_ids = BTreeMap::new();
        for row in &tags {
            let id = existing_target_tags
                .get(&row.name)
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            tag_ids.insert(row.id.clone(), id);
        }

        let mut file_size_bytes = 0u64;
        let mut copied_paths = BTreeMap::new();
        for version in &versions {
            let source_path = request.source_library_path.join(&version.file_path);
            let metadata =
                fs::metadata(&source_path).map_err(|error| io_error(&source_path, error))?;
            if !metadata.is_file() {
                return Err(DomainError::Io {
                    path: source_path.display().to_string(),
                    message: "source version file is not a file".to_string(),
                });
            }
            file_size_bytes += metadata.len();
            let new_version_id = version_ids
                .get(&version.id)
                .expect("version id map")
                .to_string();
            let extension = extension_for_mime_type(&version.mime_type);
            copied_paths.insert(
                version.id.clone(),
                managed_original_path(
                    &AssetVersionId(new_version_id),
                    extension,
                    &timestamp_string(),
                ),
            );
        }

        let summary = MergeLibrarySummary {
            source_library_id: LibraryId(source_manifest.id.clone()),
            target_library_id: LibraryId(target_manifest.id.clone()),
            asset_count: assets.len() as u32,
            version_count: versions.len() as u32,
            prompt_count: prompts.len() as u32,
            prompt_version_count: prompt_versions.len() as u32,
            album_count: albums.len() as u32,
            tag_count: tags.len() as u32,
            generation_event_count: generation_events.len() as u32,
            metadata_suggestion_count: suggestions.len() as u32,
            skipped_runtime_row_count: runtime_rows,
            file_count: versions.len() as u32,
            file_size_bytes,
            warnings: runtime_rows
                .gt(&0)
                .then(|| format!("skipped {runtime_rows} runtime row(s)"))
                .into_iter()
                .collect(),
        };

        Ok(Self {
            request: request.clone(),
            target_manifest,
            summary,
            assets,
            versions,
            generation_events,
            prompts,
            prompt_versions,
            suggestions,
            tags,
            asset_tags,
            albums,
            album_items,
            version_sources,
            asset_ids,
            version_ids,
            event_ids,
            prompt_ids,
            prompt_version_ids,
            suggestion_ids,
            album_ids,
            tag_ids,
            copied_paths,
        })
    }
}

fn copy_version_files(plan: &MergePlan) -> DomainResult<Vec<PathBuf>> {
    let mut copied = Vec::new();
    for version in &plan.versions {
        let source = plan.request.source_library_path.join(&version.file_path);
        let target_relative = plan.copied_paths.get(&version.id).expect("copied path");
        let target = plan.request.target_library_path.join(target_relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }
        fs::copy(&source, &target).map_err(|error| io_error(&target, error))?;
        copied.push(target);
    }
    Ok(copied)
}

fn write_merge_rows(plan: &MergePlan) -> DomainResult<()> {
    let connection = LocalLibraryService::open_library_database(&plan.request.target_library_path)?;
    let transaction = connection.unchecked_transaction().map_err(database_error)?;

    for asset in &plan.assets {
        transaction
            .execute(
                "
                INSERT INTO assets (
                    id, library_id, media_type, title, description, schema_prompt, category,
                    rating, status, created_at, updated_at, captured_at, archived_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ",
                params![
                    mapped(&plan.asset_ids, &asset.id),
                    plan.target_manifest.id,
                    asset.media_type,
                    asset.title,
                    asset.description,
                    asset.schema_prompt,
                    asset.category,
                    asset.rating,
                    asset.status,
                    asset.created_at,
                    asset.updated_at,
                    asset.captured_at,
                    asset.archived_at,
                ],
            )
            .map_err(database_error)?;
    }

    for tag in &plan.tags {
        transaction
            .execute(
                "INSERT OR IGNORE INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    mapped(&plan.tag_ids, &tag.id),
                    tag.name,
                    tag.color,
                    tag.created_at
                ],
            )
            .map_err(database_error)?;
    }

    for prompt in &plan.prompts {
        transaction
            .execute(
                "
                INSERT INTO prompt_documents (
                    id, library_id, name, kind, status, draft_body, draft_negative_prompt,
                    draft_style_prompt, draft_variables_schema_json, draft_default_values_json,
                    draft_parameter_preset_json, notes, created_at, updated_at, archived_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                ",
                params![
                    mapped(&plan.prompt_ids, &prompt.id),
                    plan.target_manifest.id,
                    prompt.name,
                    prompt.kind,
                    prompt.status,
                    prompt.draft_body,
                    prompt.draft_negative_prompt,
                    prompt.draft_style_prompt,
                    prompt.draft_variables_schema_json,
                    prompt.draft_default_values_json,
                    prompt.draft_parameter_preset_json,
                    prompt.notes,
                    prompt.created_at,
                    prompt.updated_at,
                    prompt.archived_at,
                ],
            )
            .map_err(database_error)?;
    }

    for prompt_version in &plan.prompt_versions {
        transaction
            .execute(
                "
                INSERT INTO prompt_versions (
                    id, prompt_id, version_number, body, negative_prompt, style_prompt,
                    variables_schema_json, default_values_json, parameter_preset_json, notes,
                    created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ",
                params![
                    mapped(&plan.prompt_version_ids, &prompt_version.id),
                    mapped(&plan.prompt_ids, &prompt_version.prompt_id),
                    prompt_version.version_number,
                    prompt_version.body,
                    prompt_version.negative_prompt,
                    prompt_version.style_prompt,
                    prompt_version.variables_schema_json,
                    prompt_version.default_values_json,
                    prompt_version.parameter_preset_json,
                    prompt_version.notes,
                    prompt_version.created_at,
                ],
            )
            .map_err(database_error)?;
    }

    for album in &plan.albums {
        transaction
            .execute(
                "
                INSERT INTO albums (
                    id, name, description, kind, smart_query_json, sort_order, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ",
                params![
                    mapped(&plan.album_ids, &album.id),
                    album.name,
                    album.description,
                    album.kind,
                    album.smart_query_json,
                    album.sort_order,
                    album.created_at,
                    album.updated_at,
                ],
            )
            .map_err(database_error)?;
    }

    for version in &plan.versions {
        transaction
            .execute(
                "
                INSERT INTO asset_versions (
                    id, asset_id, parent_version_id, generation_event_id, file_path, sha256,
                    checksum_algorithm, checksum, width, height, mime_type, version_number,
                    version_label, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                ",
                params![
                    mapped(&plan.version_ids, &version.id),
                    mapped(&plan.asset_ids, &version.asset_id),
                    optional_mapped(&plan.version_ids, version.parent_version_id.as_deref()),
                    optional_mapped(&plan.event_ids, version.generation_event_id.as_deref()),
                    plan.copied_paths
                        .get(&version.id)
                        .expect("copied path")
                        .to_string_lossy(),
                    version.sha256,
                    version.checksum_algorithm,
                    version.checksum,
                    version.width,
                    version.height,
                    version.mime_type,
                    version.version_number,
                    version.version_label,
                    version.created_at,
                ],
            )
            .map_err(database_error)?;
    }

    for event in &plan.generation_events {
        transaction
            .execute(
                "
                INSERT INTO generation_events (
                    id, asset_id, output_version_id, provider, provider_model, operation_type,
                    prompt, negative_prompt, input_asset_version_id, prompt_version_id,
                    parameters_json, raw_request_json, raw_response_json, status, started_at,
                    completed_at, error_code, error_message
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                ",
                params![
                    mapped(&plan.event_ids, &event.id),
                    optional_mapped(&plan.asset_ids, event.asset_id.as_deref()),
                    optional_mapped(&plan.version_ids, event.output_version_id.as_deref()),
                    event.provider,
                    event.provider_model,
                    event.operation_type,
                    event.prompt,
                    event.negative_prompt,
                    optional_mapped(&plan.version_ids, event.input_asset_version_id.as_deref()),
                    optional_mapped(&plan.prompt_version_ids, event.prompt_version_id.as_deref()),
                    event.parameters_json,
                    event.raw_request_json,
                    event.raw_response_json,
                    event.status,
                    event.started_at,
                    event.completed_at,
                    event.error_code,
                    event.error_message,
                ],
            )
            .map_err(database_error)?;
    }

    for asset_tag in &plan.asset_tags {
        transaction
            .execute(
                "
                INSERT OR IGNORE INTO asset_tags (asset_id, tag_id, source, confirmed_at)
                VALUES (?1, ?2, ?3, ?4)
                ",
                params![
                    mapped(&plan.asset_ids, &asset_tag.asset_id),
                    mapped(&plan.tag_ids, &asset_tag.tag_id),
                    asset_tag.source,
                    asset_tag.confirmed_at,
                ],
            )
            .map_err(database_error)?;
    }

    for item in &plan.album_items {
        transaction
            .execute(
                "
                INSERT OR IGNORE INTO album_items (album_id, asset_id, sort_order, added_at)
                VALUES (?1, ?2, ?3, ?4)
                ",
                params![
                    mapped(&plan.album_ids, &item.album_id),
                    mapped(&plan.asset_ids, &item.asset_id),
                    item.sort_order,
                    item.added_at,
                ],
            )
            .map_err(database_error)?;
    }

    for suggestion in &plan.suggestions {
        transaction
            .execute(
                "
                INSERT INTO metadata_suggestions (
                    id, asset_id, source, suggested_title, suggested_description,
                    suggested_schema_prompt, suggested_tags_json, suggested_category,
                    confidence_json, status, created_at, reviewed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ",
                params![
                    mapped(&plan.suggestion_ids, &suggestion.id),
                    mapped(&plan.asset_ids, &suggestion.asset_id),
                    suggestion.source,
                    suggestion.suggested_title,
                    suggestion.suggested_description,
                    suggestion.suggested_schema_prompt,
                    suggestion.suggested_tags_json,
                    suggestion.suggested_category,
                    suggestion.confidence_json,
                    suggestion.status,
                    suggestion.created_at,
                    suggestion.reviewed_at,
                ],
            )
            .map_err(database_error)?;
    }

    for source in &plan.version_sources {
        transaction
            .execute(
                "
                INSERT INTO asset_version_sources (
                    id, target_version_id, source_asset_id, source_version_id, source_kind, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    Uuid::new_v4().to_string(),
                    mapped(&plan.version_ids, &source.target_version_id),
                    mapped(&plan.asset_ids, &source.source_asset_id),
                    mapped(&plan.version_ids, &source.source_version_id),
                    source.source_kind,
                    source.created_at,
                ],
            )
            .map_err(database_error)?;
    }

    transaction.commit().map_err(database_error)
}

fn validate_merge_layout(root_path: &Path) -> DomainResult<()> {
    for required in REQUIRED_DIRS {
        if !root_path.join(required).is_dir() {
            return Err(DomainError::InvalidLibraryBackup {
                message: format!("required directory is missing: {required}"),
            });
        }
    }
    if !root_path.join(MANIFEST_FILE).is_file() {
        return Err(DomainError::InvalidLibraryBackup {
            message: "manifest.json is missing".to_string(),
        });
    }
    if !root_path.join(DATABASE_FILE).is_file() {
        return Err(DomainError::InvalidLibraryBackup {
            message: "library.sqlite is missing".to_string(),
        });
    }
    Ok(())
}

fn ensure_supported_manifest(manifest: &LibraryManifest) -> DomainResult<()> {
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(DomainError::SchemaMismatch {
            expected: CURRENT_SCHEMA_VERSION,
            found: manifest.schema_version,
        });
    }
    Ok(())
}

fn ensure_supported_database(connection: &Connection) -> DomainResult<()> {
    let user_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(database_error)?;
    if user_version > CURRENT_SCHEMA_VERSION {
        return Err(DomainError::SchemaMismatch {
            expected: CURRENT_SCHEMA_VERSION,
            found: user_version,
        });
    }
    Ok(())
}

fn open_read_only_database(root_path: &Path) -> DomainResult<Connection> {
    Connection::open_with_flags(
        LocalLibraryService::database_path(root_path),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(database_error)
}

fn id_map<'a>(ids: impl Iterator<Item = &'a str>) -> BTreeMap<String, String> {
    ids.map(|id| (id.to_string(), Uuid::new_v4().to_string()))
        .collect()
}

fn mapped<'a>(map: &'a BTreeMap<String, String>, id: &str) -> &'a str {
    map.get(id).map(String::as_str).expect("mapped id")
}

fn optional_mapped(map: &BTreeMap<String, String>, id: Option<&str>) -> Option<String> {
    id.map(|value| mapped(map, value).to_string())
}

fn load_target_tags_by_name(connection: &Connection) -> DomainResult<BTreeMap<String, String>> {
    let mut statement = connection
        .prepare("SELECT name, id FROM tags")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(database_error)?;
    rows.collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(database_error)
}

fn count_table(connection: &Connection, table: &str) -> DomainResult<u32> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let count: i64 = connection
        .query_row(&sql, [], |row| row.get(0))
        .map_err(database_error)?;
    Ok(count as u32)
}

fn count_runtime_rows(connection: &Connection) -> DomainResult<u32> {
    Ok(count_table(connection, "tasks")?
        + count_table(connection, "task_attempts")?
        + count_table(connection, "task_events")?
        + count_table(connection, "task_outputs")?
        + count_table(connection, "scheduled_generation_jobs")?
        + count_table(connection, "scheduled_generation_runs")?
        + count_table(connection, "scheduled_generation_run_outputs")?)
}

#[derive(Debug)]
struct AssetRow {
    id: String,
    media_type: String,
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    created_at: String,
    updated_at: String,
    captured_at: Option<String>,
    archived_at: Option<String>,
}

fn load_assets(connection: &Connection) -> DomainResult<Vec<AssetRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, media_type, title, description, schema_prompt, category, rating, status,
                   created_at, updated_at, captured_at, archived_at
            FROM assets
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetRow {
                id: row.get(0)?,
                media_type: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                schema_prompt: row.get(4)?,
                category: row.get(5)?,
                rating: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                captured_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct VersionRow {
    id: String,
    asset_id: String,
    parent_version_id: Option<String>,
    generation_event_id: Option<String>,
    file_path: PathBuf,
    sha256: String,
    checksum_algorithm: String,
    checksum: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    mime_type: String,
    version_number: u32,
    version_label: Option<String>,
    created_at: String,
}

fn load_versions(connection: &Connection) -> DomainResult<Vec<VersionRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path, sha256,
                   checksum_algorithm, checksum, width, height, mime_type, version_number,
                   version_label, created_at
            FROM asset_versions
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(VersionRow {
                id: row.get(0)?,
                asset_id: row.get(1)?,
                parent_version_id: row.get(2)?,
                generation_event_id: row.get(3)?,
                file_path: PathBuf::from(row.get::<_, String>(4)?),
                sha256: row.get(5)?,
                checksum_algorithm: row.get(6)?,
                checksum: row.get(7)?,
                width: row.get(8)?,
                height: row.get(9)?,
                mime_type: row.get(10)?,
                version_number: row.get(11)?,
                version_label: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct GenerationEventRow {
    id: String,
    asset_id: Option<String>,
    output_version_id: Option<String>,
    provider: String,
    provider_model: String,
    operation_type: String,
    prompt: String,
    negative_prompt: Option<String>,
    input_asset_version_id: Option<String>,
    prompt_version_id: Option<String>,
    parameters_json: String,
    raw_request_json: Option<String>,
    raw_response_json: Option<String>,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
}

fn load_generation_events(connection: &Connection) -> DomainResult<Vec<GenerationEventRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, output_version_id, provider, provider_model, operation_type,
                   prompt, negative_prompt, input_asset_version_id, prompt_version_id,
                   parameters_json, raw_request_json, raw_response_json, status, started_at,
                   completed_at, error_code, error_message
            FROM generation_events
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(GenerationEventRow {
                id: row.get(0)?,
                asset_id: row.get(1)?,
                output_version_id: row.get(2)?,
                provider: row.get(3)?,
                provider_model: row.get(4)?,
                operation_type: row.get(5)?,
                prompt: row.get(6)?,
                negative_prompt: row.get(7)?,
                input_asset_version_id: row.get(8)?,
                prompt_version_id: row.get(9)?,
                parameters_json: row.get(10)?,
                raw_request_json: row.get(11)?,
                raw_response_json: row.get(12)?,
                status: row.get(13)?,
                started_at: row.get(14)?,
                completed_at: row.get(15)?,
                error_code: row.get(16)?,
                error_message: row.get(17)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct PromptDocumentRow {
    id: String,
    name: String,
    kind: String,
    status: String,
    draft_body: String,
    draft_negative_prompt: Option<String>,
    draft_style_prompt: Option<String>,
    draft_variables_schema_json: String,
    draft_default_values_json: String,
    draft_parameter_preset_json: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

fn load_prompt_documents(connection: &Connection) -> DomainResult<Vec<PromptDocumentRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, name, kind, status, draft_body, draft_negative_prompt, draft_style_prompt,
                   draft_variables_schema_json, draft_default_values_json,
                   draft_parameter_preset_json, notes, created_at, updated_at, archived_at
            FROM prompt_documents
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(PromptDocumentRow {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                status: row.get(3)?,
                draft_body: row.get(4)?,
                draft_negative_prompt: row.get(5)?,
                draft_style_prompt: row.get(6)?,
                draft_variables_schema_json: row.get(7)?,
                draft_default_values_json: row.get(8)?,
                draft_parameter_preset_json: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                archived_at: row.get(13)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct PromptVersionRow {
    id: String,
    prompt_id: String,
    version_number: u32,
    body: String,
    negative_prompt: Option<String>,
    style_prompt: Option<String>,
    variables_schema_json: String,
    default_values_json: String,
    parameter_preset_json: String,
    notes: Option<String>,
    created_at: String,
}

fn load_prompt_versions(connection: &Connection) -> DomainResult<Vec<PromptVersionRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, prompt_id, version_number, body, negative_prompt, style_prompt,
                   variables_schema_json, default_values_json, parameter_preset_json, notes,
                   created_at
            FROM prompt_versions
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(PromptVersionRow {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                version_number: row.get(2)?,
                body: row.get(3)?,
                negative_prompt: row.get(4)?,
                style_prompt: row.get(5)?,
                variables_schema_json: row.get(6)?,
                default_values_json: row.get(7)?,
                parameter_preset_json: row.get(8)?,
                notes: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct MetadataSuggestionRow {
    id: String,
    asset_id: String,
    source: String,
    suggested_title: Option<String>,
    suggested_description: Option<String>,
    suggested_schema_prompt: Option<String>,
    suggested_tags_json: String,
    suggested_category: Option<String>,
    confidence_json: String,
    status: String,
    created_at: String,
    reviewed_at: Option<String>,
}

fn load_metadata_suggestions(connection: &Connection) -> DomainResult<Vec<MetadataSuggestionRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, source, suggested_title, suggested_description,
                   suggested_schema_prompt, suggested_tags_json, suggested_category,
                   confidence_json, status, created_at, reviewed_at
            FROM metadata_suggestions
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(MetadataSuggestionRow {
                id: row.get(0)?,
                asset_id: row.get(1)?,
                source: row.get(2)?,
                suggested_title: row.get(3)?,
                suggested_description: row.get(4)?,
                suggested_schema_prompt: row.get(5)?,
                suggested_tags_json: row.get(6)?,
                suggested_category: row.get(7)?,
                confidence_json: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                reviewed_at: row.get(11)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct TagRow {
    id: String,
    name: String,
    color: Option<String>,
    created_at: String,
}

fn load_tags(connection: &Connection) -> DomainResult<Vec<TagRow>> {
    let mut statement = connection
        .prepare("SELECT id, name, color, created_at FROM tags")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct AssetTagRow {
    asset_id: String,
    tag_id: String,
    source: String,
    confirmed_at: Option<String>,
}

fn load_asset_tags(connection: &Connection) -> DomainResult<Vec<AssetTagRow>> {
    let mut statement = connection
        .prepare("SELECT asset_id, tag_id, source, confirmed_at FROM asset_tags")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetTagRow {
                asset_id: row.get(0)?,
                tag_id: row.get(1)?,
                source: row.get(2)?,
                confirmed_at: row.get(3)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct AlbumRow {
    id: String,
    name: String,
    description: Option<String>,
    kind: String,
    smart_query_json: Option<String>,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}

fn load_albums(connection: &Connection) -> DomainResult<Vec<AlbumRow>> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, description, kind, smart_query_json, sort_order, created_at, updated_at FROM albums",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AlbumRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                kind: row.get(3)?,
                smart_query_json: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct AlbumItemRow {
    album_id: String,
    asset_id: String,
    sort_order: i64,
    added_at: String,
}

fn load_album_items(connection: &Connection) -> DomainResult<Vec<AlbumItemRow>> {
    let mut statement = connection
        .prepare("SELECT album_id, asset_id, sort_order, added_at FROM album_items")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AlbumItemRow {
                album_id: row.get(0)?,
                asset_id: row.get(1)?,
                sort_order: row.get(2)?,
                added_at: row.get(3)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

#[derive(Debug)]
struct AssetVersionSourceRow {
    target_version_id: String,
    source_asset_id: String,
    source_version_id: String,
    source_kind: String,
    created_at: String,
}

fn load_asset_version_sources(connection: &Connection) -> DomainResult<Vec<AssetVersionSourceRow>> {
    let mut statement = connection
        .prepare(
            "SELECT target_version_id, source_asset_id, source_version_id, source_kind, created_at FROM asset_version_sources",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetVersionSourceRow {
                target_version_id: row.get(0)?,
                source_asset_id: row.get(1)?,
                source_version_id: row.get(2)?,
                source_kind: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}
