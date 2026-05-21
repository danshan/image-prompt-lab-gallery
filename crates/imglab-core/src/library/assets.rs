use super::{
    database_error, operation_from_str, operation_to_str, storage::timestamp_string,
    LocalLibraryService,
};
use crate::application::use_cases::assets::{CreateChildVersionUseCase, ImportAssetUseCase};
use crate::{
    version_name, AssetId, AssetService, AssetSummary, AssetVersionId, CreateChildVersionRequest,
    CreateGenerationEventRequest, DomainError, DomainResult, GenerationEventId,
    GenerationEventSummary, ImportAssetRequest, LineageEntry, PersistAssetVersionRequest,
    PersistImportedAssetRequest, VersionSummary,
};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use uuid::Uuid;

impl AssetService for LocalLibraryService {
    fn import_asset(
        &self,
        request: ImportAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        ImportAssetUseCase::new(self.clone(), self.clone()).execute(request)
    }

    fn create_child_version(
        &self,
        request: CreateChildVersionRequest,
    ) -> DomainResult<VersionSummary> {
        CreateChildVersionUseCase::new(self.clone(), self.clone()).execute(request)
    }

    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let event_id = GenerationEventId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let operation_type = operation_to_str(request.operation_type);

        connection
            .execute(
                "
                INSERT INTO generation_events (
                    id, asset_id, output_version_id, provider, provider_model, operation_type,
                    prompt, negative_prompt, input_asset_version_id, parameters_json,
                    raw_request_json, raw_response_json, status, started_at, completed_at,
                    error_code, error_message
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, ?15, ?16)
                ",
                params![
                    event_id.0,
                    request.asset_id.as_ref().map(|id| id.0.as_str()),
                    request.output_version_id.as_ref().map(|id| id.0.as_str()),
                    request.provider,
                    request.provider_model,
                    operation_type,
                    request.prompt,
                    request.negative_prompt,
                    request
                        .input_asset_version_id
                        .as_ref()
                        .map(|id| id.0.as_str()),
                    request.parameters_json,
                    request.raw_request_json,
                    request.raw_response_json,
                    request.status,
                    now,
                    request.error_code,
                    request.error_message
                ],
            )
            .map_err(database_error)?;

        Ok(GenerationEventSummary {
            id: event_id,
            asset_id: request.asset_id,
            output_version_id: request.output_version_id,
            provider: request.provider,
            provider_model: request.provider_model,
            operation_type: request.operation_type,
            prompt: request.prompt,
            parameters_json: request.parameters_json,
            status: request.status,
        })
    }

    fn get_lineage(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<Vec<LineageEntry>> {
        let connection = Self::open_library_database(library_path)?;
        let mut lineage = Vec::new();
        let mut current = Some(version_id.clone());

        while let Some(current_id) = current {
            let version = load_version(&connection, &current_id)?;
            let event = match &version.generation_event_id {
                Some(event_id) => Some(load_generation_event(&connection, event_id)?),
                None => None,
            };
            current = version.parent_version_id.clone();
            lineage.push(LineageEntry {
                version,
                generation_event: event,
            });
        }

        Ok(lineage)
    }
}

pub(crate) fn import_asset_with_status(
    _service: &LocalLibraryService,
    request: PersistImportedAssetRequest,
) -> DomainResult<(AssetSummary, VersionSummary)> {
    let manifest = LocalLibraryService::read_manifest(&request.library_path)?;
    let connection = LocalLibraryService::open_library_database(&request.library_path)?;

    let asset_id = AssetId(Uuid::new_v4().to_string());
    let now = timestamp_string();
    let file = request.file;

    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    transaction
        .execute(
            "
            INSERT INTO assets (
                id, library_id, media_type, title, description, schema_prompt, category,
                rating, status, created_at, updated_at, captured_at
            )
            VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL, NULL, ?4, ?5, ?5, NULL)
            ",
            params![asset_id.0, manifest.id, file.mime_type, request.status, now],
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "
            INSERT INTO asset_versions (
                id, asset_id, parent_version_id, generation_event_id,
                file_path, sha256, checksum_algorithm, checksum, width, height, mime_type, version_number, version_label, created_at
            )
            VALUES (?1, ?2, NULL, NULL, ?3, ?4, ?5, ?4, ?6, ?7, ?8, ?9, ?10, ?11)
            ",
            params![
                request.version_id.0,
                asset_id.0,
                file.file_path.to_string_lossy(),
                file.checksum,
                file.checksum_algorithm,
                file.width,
                file.height,
                file.mime_type,
                request.version_number,
                request.version_label,
                now
            ],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)?;

    Ok((
        AssetSummary {
            id: asset_id.clone(),
            title: None,
            category: None,
            rating: None,
            status: request.status,
        },
        VersionSummary {
            id: request.version_id,
            asset_id,
            parent_version_id: None,
            generation_event_id: None,
            version_number: request.version_number,
            version_name: version_name(request.version_number),
            file_path: file.file_path,
            checksum_algorithm: file.checksum_algorithm,
            checksum: file.checksum,
            mime_type: file.mime_type,
        },
    ))
}

pub(super) fn ensure_asset_exists(connection: &Connection, asset_id: &AssetId) -> DomainResult<()> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE id = ?1",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if count == 0 {
        return Err(DomainError::InvalidAssetReference {
            id: asset_id.0.clone(),
        });
    }
    Ok(())
}

pub(crate) fn load_version(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<VersionSummary> {
    connection
        .query_row(
            "
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path,
                   sha256, checksum_algorithm, COALESCE(checksum, sha256), mime_type,
                   version_number
            FROM asset_versions
            WHERE id = ?1
            ",
            params![version_id.0],
            version_summary_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidAssetReference {
                id: version_id.0.clone(),
            },
            other => database_error(other),
        })
}

pub(crate) fn list_versions_for_asset(
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
            ORDER BY version_number ASC, created_at ASC, id ASC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], version_summary_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

pub(crate) fn persist_asset_version(
    request: PersistAssetVersionRequest,
) -> DomainResult<VersionSummary> {
    let connection = LocalLibraryService::open_library_database(&request.library_path)?;
    ensure_asset_exists(&connection, &request.asset_id)?;
    if let Some(parent_version_id) = &request.parent_version_id {
        load_version(&connection, parent_version_id)?;
    }

    let now = timestamp_string();
    let file = request.file;

    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    transaction
        .execute(
            "
            INSERT INTO asset_versions (
                id, asset_id, parent_version_id, generation_event_id,
                file_path, sha256, checksum_algorithm, checksum, width, height, mime_type, version_number, version_label, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?8, ?9, ?10, ?11, ?12, ?13)
            ",
            params![
                request.version_id.0,
                request.asset_id.0,
                request.parent_version_id.as_ref().map(|id| id.0.as_str()),
                request.generation_event_id.as_ref().map(|id| id.0.as_str()),
                file.file_path.to_string_lossy(),
                file.checksum,
                file.checksum_algorithm,
                file.width,
                file.height,
                file.mime_type,
                request.version_number,
                request.version_label.as_deref(),
                now
            ],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)?;

    if let Some(event_id) = &request.generation_event_id {
        update_generation_event_output_version(
            &request.library_path,
            event_id,
            &request.version_id,
        )?;
    }

    Ok(VersionSummary {
        id: request.version_id,
        asset_id: request.asset_id,
        parent_version_id: request.parent_version_id,
        generation_event_id: request.generation_event_id,
        version_number: request.version_number,
        version_name: version_name(request.version_number),
        file_path: file.file_path,
        checksum_algorithm: file.checksum_algorithm,
        checksum: file.checksum,
        mime_type: file.mime_type,
    })
}

pub(super) fn version_summary_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<VersionSummary> {
    let version_number: u32 = row.get(9)?;
    Ok(VersionSummary {
        id: AssetVersionId(row.get(0)?),
        asset_id: AssetId(row.get(1)?),
        parent_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
        generation_event_id: row.get::<_, Option<String>>(3)?.map(GenerationEventId),
        file_path: PathBuf::from(row.get::<_, String>(4)?),
        checksum_algorithm: row.get(6)?,
        checksum: row.get(7)?,
        mime_type: row.get(8)?,
        version_number,
        version_name: version_name(version_number),
    })
}

pub(super) fn load_generation_event(
    connection: &Connection,
    event_id: &GenerationEventId,
) -> DomainResult<GenerationEventSummary> {
    connection
        .query_row(
            "
            SELECT id, asset_id, output_version_id, provider, provider_model, operation_type,
                   prompt, parameters_json, status
            FROM generation_events
            WHERE id = ?1
            ",
            params![event_id.0],
            |row| {
                let operation_value: String = row.get(5)?;
                let operation_type = operation_from_str(&operation_value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(GenerationEventSummary {
                    id: GenerationEventId(row.get(0)?),
                    asset_id: row.get::<_, Option<String>>(1)?.map(AssetId),
                    output_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
                    provider: row.get(3)?,
                    provider_model: row.get(4)?,
                    operation_type,
                    prompt: row.get(6)?,
                    parameters_json: row.get(7)?,
                    status: row.get(8)?,
                })
            },
        )
        .map_err(database_error)
}

pub(super) fn load_asset_summary(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<AssetSummary> {
    connection
        .query_row(
            "SELECT id, title, category, rating, status FROM assets WHERE id = ?1",
            params![asset_id.0],
            |row| {
                Ok(AssetSummary {
                    id: AssetId(row.get(0)?),
                    title: row.get(1)?,
                    category: row.get(2)?,
                    rating: row.get::<_, Option<u8>>(3)?,
                    status: row.get(4)?,
                })
            },
        )
        .map_err(database_error)
}

pub(crate) fn mark_imported_version_as_generated(
    library_path: &Path,
    asset_id: &AssetId,
    version_id: &AssetVersionId,
    event_id: &GenerationEventId,
) -> DomainResult<()> {
    let connection = LocalLibraryService::open_library_database(library_path)?;
    let now = timestamp_string();
    let title = load_generation_event(&connection, event_id)
        .ok()
        .and_then(|event| default_title_from_prompt(&event.prompt));
    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    transaction
        .execute(
            "
            UPDATE asset_versions
            SET generation_event_id = ?1,
                version_label = 'generated'
            WHERE id = ?2
            ",
            params![event_id.0, version_id.0],
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "
            UPDATE assets
            SET title = COALESCE(title, ?1),
                status = 'generated',
                updated_at = ?2
            WHERE id = ?3
            ",
            params![title, now, asset_id.0],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)?;
    Ok(())
}

fn update_generation_event_output_version(
    library_path: &Path,
    event_id: &GenerationEventId,
    output_version_id: &AssetVersionId,
) -> DomainResult<()> {
    let connection = LocalLibraryService::open_library_database(library_path)?;
    connection
        .execute(
            "
            UPDATE generation_events
            SET output_version_id = ?1
            WHERE id = ?2
            ",
            params![output_version_id.0, event_id.0],
        )
        .map_err(database_error)?;
    Ok(())
}

pub(super) fn default_title_from_prompt(prompt: &str) -> Option<String> {
    let stop_words = [
        "a", "an", "and", "as", "at", "by", "for", "from", "in", "into", "of", "on", "or", "the",
        "to", "with",
    ];
    let words = prompt
        .split(|character: char| !character.is_alphanumeric())
        .filter_map(|word| {
            let word = word.trim();
            if word.is_empty() {
                return None;
            }
            let lower = word.to_ascii_lowercase();
            if stop_words.contains(&lower.as_str()) {
                return None;
            }
            Some(title_case_word(&lower))
        })
        .take(6)
        .collect::<Vec<_>>();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

fn title_case_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
