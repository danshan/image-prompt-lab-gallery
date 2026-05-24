use crate::application::ports::PromptRepository;
use crate::domain::library::LibraryManifest;
use crate::domain::prompt::{next_prompt_version_number, prompt_version_name, PromptDocumentKind};
use crate::library::{timestamp_string, LocalLibraryService};
use crate::{
    CreatePromptDocumentRequest, DomainError, DomainResult, ListPromptDocumentsRequest,
    ListPromptOutputHistoryRequest, ListPromptVersionsRequest, LoadPromptVersionRequest,
    PromptDocumentView, PromptId, PromptOutputHistoryItem, PromptVersionId, PromptVersionView,
    SaveGenerationPromptAsPromptRequest, SavePromptVersionRequest, TaskId, TaskOutputType,
    UpdatePromptDraftRequest,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use std::fs;
use std::path::Path;
use uuid::Uuid;

impl PromptRepository for LocalLibraryService {
    fn create_prompt_document(
        &self,
        request: CreatePromptDocumentRequest,
    ) -> DomainResult<PromptDocumentView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let manifest = read_library_manifest(&request.library_path)?;
        let prompt_json = validate_prompt_json(
            &request.variables_schema_json,
            &request.default_values_json,
            &request.parameter_preset_json,
        )?;
        let prompt_id = PromptId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let kind = infer_prompt_kind(&prompt_json.variables_schema);

        connection
            .execute(
                "
                INSERT INTO prompt_documents (
                    id, library_id, name, kind, status, draft_body,
                    draft_negative_prompt, draft_style_prompt, draft_variables_schema_json,
                    draft_default_values_json, draft_parameter_preset_json, notes,
                    created_at, updated_at, archived_at
                )
                VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12, NULL)
                ",
                params![
                    prompt_id.0,
                    manifest.id,
                    request.name,
                    kind.as_str(),
                    request.draft_body,
                    request.draft_negative_prompt,
                    request.draft_style_prompt,
                    request.variables_schema_json,
                    request.default_values_json,
                    request.parameter_preset_json,
                    request.notes,
                    now,
                ],
            )
            .map_err(database_error)?;

        load_prompt_document(&connection, &prompt_id)
    }

    fn update_prompt_draft(
        &self,
        request: UpdatePromptDraftRequest,
    ) -> DomainResult<PromptDocumentView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let prompt_id = PromptId(request.prompt_id);
        let prompt_json = validate_prompt_json(
            &request.variables_schema_json,
            &request.default_values_json,
            &request.parameter_preset_json,
        )?;
        let kind = infer_prompt_kind(&prompt_json.variables_schema);
        let now = timestamp_string();
        let changed = connection
            .execute(
                "
                UPDATE prompt_documents
                SET name = ?1,
                    kind = ?2,
                    draft_body = ?3,
                    draft_negative_prompt = ?4,
                    draft_style_prompt = ?5,
                    draft_variables_schema_json = ?6,
                    draft_default_values_json = ?7,
                    draft_parameter_preset_json = ?8,
                    notes = ?9,
                    updated_at = ?10
                WHERE id = ?11
                ",
                params![
                    request.name,
                    kind.as_str(),
                    request.draft_body,
                    request.draft_negative_prompt,
                    request.draft_style_prompt,
                    request.variables_schema_json,
                    request.default_values_json,
                    request.parameter_preset_json,
                    request.notes,
                    now,
                    prompt_id.0,
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_prompt_reference(&prompt_id));
        }

        load_prompt_document(&connection, &prompt_id)
    }

    fn save_prompt_version(
        &self,
        request: SavePromptVersionRequest,
    ) -> DomainResult<PromptVersionView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let prompt_id = PromptId(request.prompt_id);
        ensure_prompt_document_exists(&connection, &prompt_id)?;
        let current_max = connection
            .query_row(
                "SELECT MAX(version_number) FROM prompt_versions WHERE prompt_id = ?1",
                params![prompt_id.0],
                |row| row.get::<_, Option<u32>>(0),
            )
            .map_err(database_error)?;
        let version_number = next_prompt_version_number(current_max);
        let version_id = PromptVersionId(Uuid::new_v4().to_string());
        let now = timestamp_string();

        let changed = connection
            .execute(
                "
                INSERT INTO prompt_versions (
                    id, prompt_id, version_number, body, negative_prompt, style_prompt,
                    variables_schema_json, default_values_json, parameter_preset_json,
                    notes, created_at
                )
                SELECT ?1, id, ?2, draft_body, draft_negative_prompt, draft_style_prompt,
                       draft_variables_schema_json, draft_default_values_json,
                       draft_parameter_preset_json, notes, ?3
                FROM prompt_documents
                WHERE id = ?4
                ",
                params![version_id.0, version_number, now, prompt_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_prompt_reference(&prompt_id));
        }

        load_prompt_version(&connection, &version_id)
    }

    fn list_prompt_documents(
        &self,
        request: ListPromptDocumentsRequest,
    ) -> DomainResult<Vec<PromptDocumentView>> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let manifest = read_library_manifest(&request.library_path)?;
        let query_pattern = request
            .query
            .as_ref()
            .map(|query| format!("%{}%", query.trim()))
            .filter(|query| query != "%%");
        let mut statement = connection
            .prepare(
                "
                SELECT
                    pd.id, pd.name, pd.kind, pd.status, pd.draft_body,
                    pd.draft_negative_prompt, pd.draft_style_prompt,
                    pd.draft_variables_schema_json, pd.draft_default_values_json,
                    pd.draft_parameter_preset_json, pd.notes, latest.id,
                    latest.version_number, pd.created_at, pd.updated_at, pd.archived_at
                FROM prompt_documents pd
                LEFT JOIN prompt_versions latest
                    ON latest.prompt_id = pd.id
                    AND latest.version_number = (
                        SELECT MAX(version_number)
                        FROM prompt_versions
                        WHERE prompt_id = pd.id
                    )
                WHERE pd.library_id = ?1
                  AND (?2 OR pd.status = 'active')
                  AND (
                    ?3 IS NULL
                    OR pd.name LIKE ?3
                    OR pd.draft_body LIKE ?3
                    OR pd.notes LIKE ?3
                  )
                ORDER BY pd.updated_at DESC, pd.id DESC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(
                params![manifest.id, request.include_archived, query_pattern],
                prompt_document_from_row,
            )
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn list_prompt_versions(
        &self,
        request: ListPromptVersionsRequest,
    ) -> DomainResult<Vec<PromptVersionView>> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let prompt_id = PromptId(request.prompt_id);
        ensure_prompt_document_exists(&connection, &prompt_id)?;
        let mut statement = connection
            .prepare(
                "
                SELECT id, prompt_id, version_number, body, negative_prompt, style_prompt,
                       variables_schema_json, default_values_json, parameter_preset_json,
                       notes, created_at
                FROM prompt_versions
                WHERE prompt_id = ?1
                ORDER BY version_number DESC, id DESC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![prompt_id.0], prompt_version_from_row)
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn load_prompt_version(
        &self,
        request: LoadPromptVersionRequest,
    ) -> DomainResult<PromptVersionView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let version_id = PromptVersionId(request.prompt_version_id);
        load_prompt_version(&connection, &version_id)
    }

    fn list_prompt_output_history(
        &self,
        request: ListPromptOutputHistoryRequest,
    ) -> DomainResult<Vec<PromptOutputHistoryItem>> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let version_id = PromptVersionId(request.prompt_version_id);
        let mut statement = connection
            .prepare(
                "
                SELECT
                    ge.id, ge.asset_id, ge.output_version_id, task_outputs.task_id,
                    ge.provider, ge.provider_model, ge.status, ge.prompt, ge.started_at
                FROM generation_events ge
                LEFT JOIN task_outputs
                    ON task_outputs.output_type = ?2
                    AND task_outputs.target_id = ge.id
                WHERE ge.prompt_version_id = ?1
                ORDER BY ge.started_at DESC, ge.id DESC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(
                params![version_id.0, TaskOutputType::GenerationEvent.as_str()],
                prompt_output_history_from_row,
            )
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn save_generation_prompt_as_prompt(
        &self,
        request: SaveGenerationPromptAsPromptRequest,
    ) -> DomainResult<PromptVersionView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let manifest = read_library_manifest(&request.library_path)?;
        let snapshot = load_generation_prompt_snapshot(&connection, &request.generation_event_id)?;
        let prompt_id = PromptId(Uuid::new_v4().to_string());
        let version_id = PromptVersionId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let notes = Some(format!(
            "Saved from generation event {}",
            request.generation_event_id
        ));

        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO prompt_documents (
                    id, library_id, name, kind, status, draft_body,
                    draft_negative_prompt, draft_style_prompt, draft_variables_schema_json,
                    draft_default_values_json, draft_parameter_preset_json, notes,
                    created_at, updated_at, archived_at
                )
                VALUES (?1, ?2, ?3, 'image_generation', 'active', ?4, ?5, NULL, '[]', '{}', ?6, ?7, ?8, ?8, NULL)
                ",
                params![
                    prompt_id.0,
                    manifest.id,
                    request.name,
                    snapshot.prompt,
                    snapshot.negative_prompt,
                    snapshot.parameters_json,
                    notes,
                    now,
                ],
            )
            .map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO prompt_versions (
                    id, prompt_id, version_number, body, negative_prompt, style_prompt,
                    variables_schema_json, default_values_json, parameter_preset_json,
                    notes, created_at
                )
                VALUES (?1, ?2, 1, ?3, ?4, NULL, '[]', '{}', ?5, ?6, ?7)
                ",
                params![
                    version_id.0,
                    prompt_id.0,
                    snapshot.prompt,
                    snapshot.negative_prompt,
                    snapshot.parameters_json,
                    notes,
                    now,
                ],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)?;

        load_prompt_version(&connection, &version_id)
    }
}

struct GenerationPromptSnapshot {
    prompt: String,
    negative_prompt: Option<String>,
    parameters_json: String,
}

fn load_generation_prompt_snapshot(
    connection: &Connection,
    generation_event_id: &str,
) -> DomainResult<GenerationPromptSnapshot> {
    connection
        .query_row(
            "
            SELECT prompt, negative_prompt, parameters_json
            FROM generation_events
            WHERE id = ?1
            ",
            params![generation_event_id],
            |row| {
                Ok(GenerationPromptSnapshot {
                    prompt: row.get(0)?,
                    negative_prompt: row.get(1)?,
                    parameters_json: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| DomainError::InvalidGenerationParameters {
            message: format!("invalid generation event reference: {generation_event_id}"),
        })
}

fn load_prompt_document(
    connection: &Connection,
    prompt_id: &PromptId,
) -> DomainResult<PromptDocumentView> {
    connection
        .query_row(
            "
            SELECT
                pd.id, pd.name, pd.kind, pd.status, pd.draft_body,
                pd.draft_negative_prompt, pd.draft_style_prompt,
                pd.draft_variables_schema_json, pd.draft_default_values_json,
                pd.draft_parameter_preset_json, pd.notes, latest.id,
                latest.version_number, pd.created_at, pd.updated_at, pd.archived_at
            FROM prompt_documents pd
            LEFT JOIN prompt_versions latest
                ON latest.prompt_id = pd.id
                AND latest.version_number = (
                    SELECT MAX(version_number)
                    FROM prompt_versions
                    WHERE prompt_id = pd.id
                )
            WHERE pd.id = ?1
            ",
            params![prompt_id.0],
            prompt_document_from_row,
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| invalid_prompt_reference(prompt_id))
}

fn load_prompt_version(
    connection: &Connection,
    version_id: &PromptVersionId,
) -> DomainResult<PromptVersionView> {
    connection
        .query_row(
            "
            SELECT id, prompt_id, version_number, body, negative_prompt, style_prompt,
                   variables_schema_json, default_values_json, parameter_preset_json,
                   notes, created_at
            FROM prompt_versions
            WHERE id = ?1
            ",
            params![version_id.0],
            prompt_version_from_row,
        )
        .map_err(database_error)
}

fn prompt_document_from_row(row: &Row<'_>) -> rusqlite::Result<PromptDocumentView> {
    let latest_version_id = row.get::<_, Option<String>>(11)?.map(PromptVersionId);
    let latest_version_number = row.get::<_, Option<u32>>(12)?;
    Ok(PromptDocumentView {
        id: PromptId(row.get(0)?),
        name: row.get(1)?,
        kind: row.get(2)?,
        status: row.get(3)?,
        draft_body: row.get(4)?,
        draft_negative_prompt: row.get(5)?,
        draft_style_prompt: row.get(6)?,
        variables_schema_json: row.get(7)?,
        default_values_json: row.get(8)?,
        parameter_preset_json: row.get(9)?,
        notes: row.get(10)?,
        latest_version_id,
        latest_version_number,
        latest_version_name: latest_version_number.map(prompt_version_name),
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        archived_at: row.get(15)?,
    })
}

fn prompt_version_from_row(row: &Row<'_>) -> rusqlite::Result<PromptVersionView> {
    let version_number = row.get::<_, u32>(2)?;
    Ok(PromptVersionView {
        id: PromptVersionId(row.get(0)?),
        prompt_id: PromptId(row.get(1)?),
        version_number,
        version_name: prompt_version_name(version_number),
        body: row.get(3)?,
        negative_prompt: row.get(4)?,
        style_prompt: row.get(5)?,
        variables_schema_json: row.get(6)?,
        default_values_json: row.get(7)?,
        parameter_preset_json: row.get(8)?,
        notes: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn prompt_output_history_from_row(row: &Row<'_>) -> rusqlite::Result<PromptOutputHistoryItem> {
    Ok(PromptOutputHistoryItem {
        generation_event_id: crate::GenerationEventId(row.get(0)?),
        asset_id: row.get::<_, Option<String>>(1)?.map(crate::AssetId),
        output_version_id: row.get::<_, Option<String>>(2)?.map(crate::AssetVersionId),
        task_id: row.get::<_, Option<String>>(3)?.map(TaskId),
        provider: row.get(4)?,
        provider_model: row.get(5)?,
        status: row.get(6)?,
        prompt_snapshot: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn ensure_prompt_document_exists(
    connection: &Connection,
    prompt_id: &PromptId,
) -> DomainResult<()> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM prompt_documents WHERE id = ?1)",
            params![prompt_id.0],
            |row| row.get::<_, bool>(0),
        )
        .map_err(database_error)?;
    if exists {
        Ok(())
    } else {
        Err(invalid_prompt_reference(prompt_id))
    }
}

struct ValidatedPromptJson {
    variables_schema: Value,
}

fn validate_prompt_json(
    variables_schema_json: &str,
    default_values_json: &str,
    parameter_preset_json: &str,
) -> DomainResult<ValidatedPromptJson> {
    let variables_schema = parse_json_field("variables_schema_json", variables_schema_json)?;
    parse_json_field("default_values_json", default_values_json)?;
    parse_json_field("parameter_preset_json", parameter_preset_json)?;
    Ok(ValidatedPromptJson { variables_schema })
}

fn parse_json_field(field: &str, value: &str) -> DomainResult<Value> {
    serde_json::from_str(value).map_err(|error| DomainError::InvalidGenerationParameters {
        message: format!("{field} must be valid JSON: {error}"),
    })
}

fn infer_prompt_kind(variables_schema: &Value) -> PromptDocumentKind {
    match variables_schema {
        Value::Array(values) if !values.is_empty() => PromptDocumentKind::Template,
        Value::Object(values) if !values.is_empty() => PromptDocumentKind::Template,
        _ => PromptDocumentKind::Draft,
    }
}

fn read_library_manifest(root_path: &Path) -> DomainResult<LibraryManifest> {
    let path = LocalLibraryService::manifest_path(root_path);
    let content = fs::read_to_string(&path).map_err(|error| DomainError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&content).map_err(|error| DomainError::Serialization {
        message: error.to_string(),
    })
}

fn invalid_prompt_reference(prompt_id: &PromptId) -> DomainError {
    DomainError::InvalidGenerationParameters {
        message: format!("invalid prompt reference: {}", prompt_id.0),
    }
}

fn database_error(error: rusqlite::Error) -> DomainError {
    DomainError::Database {
        message: error.to_string(),
    }
}
