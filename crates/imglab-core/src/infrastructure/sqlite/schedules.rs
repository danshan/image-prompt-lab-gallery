use crate::application::ports::ScheduleRepository;
use crate::library::{timestamp_string, LocalLibraryService};
use crate::{
    CreateScheduledGenerationJobRequest, CreateScheduledGenerationRunRequest, DomainError,
    DomainResult, LibraryId, LibrarySummary, ScheduleMissedRunPolicy, ScheduleOverlapPolicy,
    SchedulePromptMode, ScheduleRule, ScheduledGenerationJobId, ScheduledGenerationJobStatus,
    ScheduledGenerationJobView, ScheduledGenerationRunId, ScheduledGenerationRunOutputView,
    ScheduledGenerationRunStatus, ScheduledGenerationRunView, UpdateScheduledGenerationJobRequest,
    UpdateScheduledGenerationRunRequest, UpsertScheduledGenerationRunOutputRequest,
};
use rusqlite::{params, Connection, Row};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

impl ScheduleRepository for LocalLibraryService {
    fn create_scheduled_generation_job(
        &self,
        request: CreateScheduledGenerationJobRequest,
    ) -> DomainResult<ScheduledGenerationJobView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        ensure_manual_album(&connection, &request.target_album_id.0)?;
        let job_id = ScheduledGenerationJobId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let encoded = encode_schedule_rule(&request.schedule_rule);
        let tags_json = serde_json::to_string(&request.tags).map_err(serialization_error)?;
        connection
            .execute(
                "
                INSERT INTO scheduled_generation_jobs (
                    id, library_id, name, status, prompt_mode, fixed_prompt, negative_prompt,
                    base_prompt, dynamic_prompt, prompt_expander_provider, prompt_expander_model,
                    image_provider, image_model, parameters_json, schedule_kind, schedule_value_json,
                    timezone_id, target_album_id, tags_json, overlap_policy, missed_run_policy,
                    last_run_at, next_run_at, created_at, updated_at, paused_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                        ?16, ?17, ?18, ?19, ?20, ?21, NULL, ?22, ?23, ?23, NULL)
                ",
                params![
                    job_id.0,
                    request.library_id.0,
                    request.name,
                    ScheduledGenerationJobStatus::Active.as_str(),
                    request.prompt_mode.as_str(),
                    request.fixed_prompt,
                    request.negative_prompt,
                    request.base_prompt,
                    request.dynamic_prompt,
                    request.prompt_expander_provider,
                    request.prompt_expander_model,
                    request.image_provider,
                    request.image_model,
                    request.parameters_json,
                    encoded.kind,
                    encoded.value_json,
                    encoded.timezone_id,
                    request.target_album_id.0,
                    tags_json,
                    ScheduleOverlapPolicy::Skip.as_str(),
                    ScheduleMissedRunPolicy::NoCatchUp.as_str(),
                    request.next_run_at,
                    now,
                ],
            )
            .map_err(database_error)?;
        load_job(&connection, &job_id)
    }

    fn update_scheduled_generation_job(
        &self,
        request: UpdateScheduledGenerationJobRequest,
    ) -> DomainResult<ScheduledGenerationJobView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        ensure_manual_album(&connection, &request.target_album_id.0)?;
        let now = timestamp_string();
        let encoded = encode_schedule_rule(&request.schedule_rule);
        let tags_json = serde_json::to_string(&request.tags).map_err(serialization_error)?;
        let changed = connection
            .execute(
                "
                UPDATE scheduled_generation_jobs
                SET name = ?1, prompt_mode = ?2, fixed_prompt = ?3, negative_prompt = ?4,
                    base_prompt = ?5, dynamic_prompt = ?6, prompt_expander_provider = ?7,
                    prompt_expander_model = ?8, image_provider = ?9, image_model = ?10,
                    parameters_json = ?11, schedule_kind = ?12, schedule_value_json = ?13,
                    timezone_id = ?14, target_album_id = ?15, tags_json = ?16,
                    next_run_at = ?17, updated_at = ?18
                WHERE id = ?19
                ",
                params![
                    request.name,
                    request.prompt_mode.as_str(),
                    request.fixed_prompt,
                    request.negative_prompt,
                    request.base_prompt,
                    request.dynamic_prompt,
                    request.prompt_expander_provider,
                    request.prompt_expander_model,
                    request.image_provider,
                    request.image_model,
                    request.parameters_json,
                    encoded.kind,
                    encoded.value_json,
                    encoded.timezone_id,
                    request.target_album_id.0,
                    tags_json,
                    request.next_run_at,
                    now,
                    request.job_id.0,
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_job_reference(&request.job_id));
        }
        load_job(&connection, &request.job_id)
    }

    fn list_scheduled_generation_jobs(
        &self,
        library_path: &Path,
    ) -> DomainResult<Vec<ScheduledGenerationJobView>> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        query_jobs(
            &connection,
            "
            SELECT *
            FROM scheduled_generation_jobs
            WHERE status <> 'deleted'
            ORDER BY status ASC, next_run_at ASC, name ASC
            ",
            [],
        )
    }

    fn list_due_scheduled_generation_jobs(
        &self,
        library_path: &Path,
        now: &str,
    ) -> DomainResult<Vec<ScheduledGenerationJobView>> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let mut statement = connection
            .prepare(
                "
                SELECT *
                FROM scheduled_generation_jobs
                WHERE status = ?1 AND next_run_at <= ?2
                ORDER BY next_run_at ASC, id ASC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(
                params![ScheduledGenerationJobStatus::Active.as_str(), now],
                job_from_row,
            )
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn set_scheduled_generation_job_status(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
        status: ScheduledGenerationJobStatus,
    ) -> DomainResult<ScheduledGenerationJobView> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let now = timestamp_string();
        let paused_at = (status != ScheduledGenerationJobStatus::Active).then_some(now.clone());
        let changed = connection
            .execute(
                "
                UPDATE scheduled_generation_jobs
                SET status = ?1, updated_at = ?2, paused_at = ?3
                WHERE id = ?4
                ",
                params![status.as_str(), now, paused_at, job_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_job_reference(job_id));
        }
        load_job(&connection, job_id)
    }

    fn delete_scheduled_generation_job(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
    ) -> DomainResult<()> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let now = timestamp_string();
        let changed = connection
            .execute(
                "
                UPDATE scheduled_generation_jobs
                SET status = ?1, updated_at = ?2, paused_at = ?2
                WHERE id = ?3
                ",
                params![
                    ScheduledGenerationJobStatus::Deleted.as_str(),
                    now,
                    job_id.0
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_job_reference(job_id));
        }
        Ok(())
    }

    fn create_scheduled_generation_run(
        &self,
        request: CreateScheduledGenerationRunRequest,
    ) -> DomainResult<ScheduledGenerationRunView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let run_id = ScheduledGenerationRunId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        connection
            .execute(
                "
                INSERT INTO scheduled_generation_runs (
                    id, job_id, library_id, status, scheduled_for, started_at, completed_at,
                    skip_reason, error_code, error_message, expanded_prompt,
                    prompt_expansion_provider_metadata_json, image_task_id, output_asset_count,
                    tagged_asset_count, album_added_asset_count, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, ?6, ?6)
                ",
                params![
                    run_id.0,
                    request.job_id.0,
                    request.library_id.0,
                    ScheduledGenerationRunStatus::Pending.as_str(),
                    request.scheduled_for,
                    now,
                ],
            )
            .map_err(database_error)?;
        load_run(&connection, &run_id)
    }

    fn update_scheduled_generation_run(
        &self,
        request: UpdateScheduledGenerationRunRequest,
    ) -> DomainResult<ScheduledGenerationRunView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let now = timestamp_string();
        let changed = connection
            .execute(
                "
                UPDATE scheduled_generation_runs
                SET status = ?1, started_at = COALESCE(?2, started_at),
                    completed_at = ?3, skip_reason = ?4, error_code = ?5,
                    error_message = ?6, expanded_prompt = ?7,
                    prompt_expansion_provider_metadata_json = ?8, image_task_id = ?9,
                    updated_at = ?10
                WHERE id = ?11
                ",
                params![
                    request.status.as_str(),
                    request.started_at,
                    request.completed_at,
                    request.skip_reason,
                    request.error_code,
                    request.error_message,
                    request.expanded_prompt,
                    request.prompt_expansion_provider_metadata_json,
                    request.image_task_id.map(|id| id.0),
                    now,
                    request.run_id.0,
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_run_reference(&request.run_id));
        }
        load_run(&connection, &request.run_id)
    }

    fn list_scheduled_generation_runs(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
    ) -> DomainResult<Vec<ScheduledGenerationRunView>> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let mut statement = connection
            .prepare(
                "
                SELECT *
                FROM scheduled_generation_runs
                WHERE job_id = ?1
                ORDER BY scheduled_for DESC, id DESC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![job_id.0], run_from_row)
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn upsert_scheduled_generation_run_output(
        &self,
        request: UpsertScheduledGenerationRunOutputRequest,
    ) -> DomainResult<ScheduledGenerationRunOutputView> {
        let connection = LocalLibraryService::open_library_database(&request.library_path)?;
        let now = timestamp_string();
        let tags_json =
            serde_json::to_string(&request.tags_applied).map_err(serialization_error)?;
        connection
            .execute(
                "
                INSERT INTO scheduled_generation_run_outputs (
                    run_id, asset_id, asset_version_id, generation_event_id, album_added,
                    tags_applied_json, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                ON CONFLICT(run_id, asset_id) DO UPDATE SET
                    asset_version_id = excluded.asset_version_id,
                    generation_event_id = excluded.generation_event_id,
                    album_added = excluded.album_added,
                    tags_applied_json = excluded.tags_applied_json,
                    updated_at = excluded.updated_at
                ",
                params![
                    request.run_id.0,
                    request.asset_id.0,
                    request.asset_version_id.as_ref().map(|id| id.0.as_str()),
                    request.generation_event_id.as_ref().map(|id| id.0.as_str()),
                    request.album_added as i64,
                    tags_json,
                    now,
                ],
            )
            .map_err(database_error)?;
        connection
            .execute(
                "
                UPDATE scheduled_generation_runs
                SET output_asset_count = (
                        SELECT COUNT(*)
                        FROM scheduled_generation_run_outputs
                        WHERE run_id = ?1
                    ),
                    album_added_asset_count = (
                        SELECT COUNT(*)
                        FROM scheduled_generation_run_outputs
                        WHERE run_id = ?1 AND album_added = 1
                    ),
                    tagged_asset_count = (
                        SELECT COUNT(*)
                        FROM scheduled_generation_run_outputs
                        WHERE run_id = ?1 AND tags_applied_json != '[]'
                    ),
                    updated_at = ?2
                WHERE id = ?1
                ",
                params![request.run_id.0, now],
            )
            .map_err(database_error)?;
        load_run_output(&connection, &request.run_id, &request.asset_id.0)
    }

    fn set_library_automation_enabled(
        &self,
        library_id: &LibraryId,
        enabled: bool,
    ) -> DomainResult<LibrarySummary> {
        self.set_registry_library_automation_enabled(library_id, enabled)
    }
}

struct EncodedScheduleRule {
    kind: &'static str,
    value_json: String,
    timezone_id: String,
}

fn encode_schedule_rule(rule: &ScheduleRule) -> EncodedScheduleRule {
    match rule {
        ScheduleRule::IntervalMinutes(minutes) => EncodedScheduleRule {
            kind: "interval_minutes",
            value_json: serde_json::json!({ "minutes": minutes }).to_string(),
            timezone_id: "UTC".to_string(),
        },
        ScheduleRule::IntervalHours(hours) => EncodedScheduleRule {
            kind: "interval_hours",
            value_json: serde_json::json!({ "hours": hours }).to_string(),
            timezone_id: "UTC".to_string(),
        },
        ScheduleRule::DailyTime {
            timezone_id,
            local_time_hh_mm,
        } => EncodedScheduleRule {
            kind: "daily_time",
            value_json: serde_json::json!({ "localTime": local_time_hh_mm }).to_string(),
            timezone_id: timezone_id.clone(),
        },
    }
}

fn decode_schedule_rule(
    kind: &str,
    value_json: &str,
    timezone_id: &str,
) -> rusqlite::Result<ScheduleRule> {
    let value: Value = serde_json::from_str(value_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })?;
    match kind {
        "interval_minutes" => Ok(ScheduleRule::IntervalMinutes(
            value["minutes"].as_u64().unwrap_or_default() as u32,
        )),
        "interval_hours" => Ok(ScheduleRule::IntervalHours(
            value["hours"].as_u64().unwrap_or_default() as u32,
        )),
        "daily_time" => Ok(ScheduleRule::DailyTime {
            timezone_id: timezone_id.to_string(),
            local_time_hh_mm: value["localTime"].as_str().unwrap_or_default().to_string(),
        }),
        _ => Err(rusqlite::Error::InvalidColumnType(
            0,
            "schedule_kind".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}

fn query_jobs<P: rusqlite::Params>(
    connection: &Connection,
    sql: &str,
    params: P,
) -> DomainResult<Vec<ScheduledGenerationJobView>> {
    let mut statement = connection.prepare(sql).map_err(database_error)?;
    let rows = statement
        .query_map(params, job_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_job(
    connection: &Connection,
    job_id: &ScheduledGenerationJobId,
) -> DomainResult<ScheduledGenerationJobView> {
    connection
        .query_row(
            "SELECT * FROM scheduled_generation_jobs WHERE id = ?1",
            params![job_id.0],
            job_from_row,
        )
        .map_err(database_error)
}

fn job_from_row(row: &Row<'_>) -> rusqlite::Result<ScheduledGenerationJobView> {
    let tags_json: String = row.get("tags_json")?;
    let tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let schedule_kind: String = row.get("schedule_kind")?;
    let schedule_value_json: String = row.get("schedule_value_json")?;
    let timezone_id: String = row.get("timezone_id")?;
    Ok(ScheduledGenerationJobView {
        id: ScheduledGenerationJobId(row.get("id")?),
        library_id: LibraryId(row.get("library_id")?),
        name: row.get("name")?,
        status: parse_job_status(row.get::<_, String>("status")?)?,
        prompt_mode: parse_prompt_mode(row.get::<_, String>("prompt_mode")?)?,
        fixed_prompt: row.get("fixed_prompt")?,
        negative_prompt: row.get("negative_prompt")?,
        base_prompt: row.get("base_prompt")?,
        dynamic_prompt: row.get("dynamic_prompt")?,
        prompt_expander_provider: row.get("prompt_expander_provider")?,
        prompt_expander_model: row.get("prompt_expander_model")?,
        image_provider: row.get("image_provider")?,
        image_model: row.get("image_model")?,
        parameters_json: row.get("parameters_json")?,
        schedule_rule: decode_schedule_rule(&schedule_kind, &schedule_value_json, &timezone_id)?,
        target_album_id: crate::AlbumId(row.get("target_album_id")?),
        tags,
        overlap_policy: ScheduleOverlapPolicy::Skip,
        missed_run_policy: ScheduleMissedRunPolicy::NoCatchUp,
        last_run_at: row.get("last_run_at")?,
        next_run_at: row.get("next_run_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        paused_at: row.get("paused_at")?,
    })
}

fn load_run(
    connection: &Connection,
    run_id: &ScheduledGenerationRunId,
) -> DomainResult<ScheduledGenerationRunView> {
    connection
        .query_row(
            "SELECT * FROM scheduled_generation_runs WHERE id = ?1",
            params![run_id.0],
            run_from_row,
        )
        .map_err(database_error)
}

fn run_from_row(row: &Row<'_>) -> rusqlite::Result<ScheduledGenerationRunView> {
    Ok(ScheduledGenerationRunView {
        id: ScheduledGenerationRunId(row.get("id")?),
        job_id: ScheduledGenerationJobId(row.get("job_id")?),
        library_id: LibraryId(row.get("library_id")?),
        status: parse_run_status(row.get::<_, String>("status")?)?,
        scheduled_for: row.get("scheduled_for")?,
        started_at: row.get("started_at")?,
        completed_at: row.get("completed_at")?,
        skip_reason: row.get("skip_reason")?,
        error_code: row.get("error_code")?,
        error_message: row.get("error_message")?,
        expanded_prompt: row.get("expanded_prompt")?,
        prompt_expansion_provider_metadata_json: row
            .get("prompt_expansion_provider_metadata_json")?,
        image_task_id: row
            .get::<_, Option<String>>("image_task_id")?
            .map(crate::TaskId),
        output_asset_count: row.get::<_, i64>("output_asset_count")? as u32,
        tagged_asset_count: row.get::<_, i64>("tagged_asset_count")? as u32,
        album_added_asset_count: row.get::<_, i64>("album_added_asset_count")? as u32,
    })
}

fn load_run_output(
    connection: &Connection,
    run_id: &ScheduledGenerationRunId,
    asset_id: &str,
) -> DomainResult<ScheduledGenerationRunOutputView> {
    connection
        .query_row(
            "
            SELECT run_id, asset_id, asset_version_id, generation_event_id, album_added,
                   tags_applied_json, created_at, updated_at
            FROM scheduled_generation_run_outputs
            WHERE run_id = ?1 AND asset_id = ?2
            ",
            params![run_id.0, asset_id],
            |row| {
                let tags_json: String = row.get("tags_applied_json")?;
                let tags_applied = serde_json::from_str(&tags_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(ScheduledGenerationRunOutputView {
                    run_id: ScheduledGenerationRunId(row.get("run_id")?),
                    asset_id: crate::AssetId(row.get("asset_id")?),
                    asset_version_id: row
                        .get::<_, Option<String>>("asset_version_id")?
                        .map(crate::AssetVersionId),
                    generation_event_id: row
                        .get::<_, Option<String>>("generation_event_id")?
                        .map(crate::GenerationEventId),
                    album_added: row.get::<_, i64>("album_added")? != 0,
                    tags_applied,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                })
            },
        )
        .map_err(database_error)
}

fn ensure_manual_album(connection: &Connection, album_id: &str) -> DomainResult<()> {
    let kind: String = connection
        .query_row(
            "SELECT kind FROM albums WHERE id = ?1",
            params![album_id],
            |row| row.get(0),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidGenerationParameters {
                message: format!("scheduled generation target album does not exist: {album_id}"),
            },
            other => database_error(other),
        })?;
    if kind == "manual" {
        Ok(())
    } else {
        Err(DomainError::InvalidGenerationParameters {
            message: format!("scheduled generation target album must be manual: {album_id}"),
        })
    }
}

fn parse_job_status(value: String) -> rusqlite::Result<ScheduledGenerationJobStatus> {
    ScheduledGenerationJobStatus::parse(&value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(0, "status".to_string(), rusqlite::types::Type::Text)
    })
}

fn parse_run_status(value: String) -> rusqlite::Result<ScheduledGenerationRunStatus> {
    ScheduledGenerationRunStatus::parse(&value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(0, "status".to_string(), rusqlite::types::Type::Text)
    })
}

fn parse_prompt_mode(value: String) -> rusqlite::Result<SchedulePromptMode> {
    SchedulePromptMode::parse(&value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(
            0,
            "prompt_mode".to_string(),
            rusqlite::types::Type::Text,
        )
    })
}

fn invalid_job_reference(job_id: &ScheduledGenerationJobId) -> DomainError {
    DomainError::InvalidGenerationParameters {
        message: format!("invalid scheduled generation job reference: {}", job_id.0),
    }
}

fn invalid_run_reference(run_id: &ScheduledGenerationRunId) -> DomainError {
    DomainError::InvalidGenerationParameters {
        message: format!("invalid scheduled generation run reference: {}", run_id.0),
    }
}

fn database_error(error: rusqlite::Error) -> DomainError {
    DomainError::Database {
        message: error.to_string(),
    }
}

fn serialization_error(error: serde_json::Error) -> DomainError {
    DomainError::Serialization {
        message: error.to_string(),
    }
}
