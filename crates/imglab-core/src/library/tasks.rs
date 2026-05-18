use super::{database_error, operation_from_str, operation_to_str, storage::timestamp_string};
use crate::{
    AppendTaskAttemptRequest, AppendTaskEventRequest, AppendTaskOutputRequest,
    BatchCreateTasksRequest, CompleteTaskAttemptRequest, DomainError, DomainResult,
    ReorderQueuedTasksRequest, TaskAttempt, TaskAttemptId, TaskDetail, TaskErrorClassification,
    TaskEvent, TaskEventId, TaskId, TaskOutput, TaskOutputId, TaskOutputType, TaskService,
    TaskStatus, TaskSummary, TaskType, UpdateTaskStatusRequest,
};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::LocalLibraryService;

impl TaskService for LocalLibraryService {
    fn create_tasks(&self, request: BatchCreateTasksRequest) -> DomainResult<Vec<TaskSummary>> {
        let connection = Self::open_library_database(&request.library_path)?;
        let now = timestamp_string();
        let start_position = next_queue_position(&connection)?;
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        let mut created = Vec::new();

        for (index, input) in request.tasks.iter().enumerate() {
            let task_id = TaskId(Uuid::new_v4().to_string());
            let queue_position = start_position + index as i64;
            transaction
                .execute(
                    "
                    INSERT INTO tasks (
                        id, library_id, task_type, status, queue_position, priority,
                        provider, operation_type, concurrency_group, attempt_count,
                        max_attempts, next_retry_at, input_json, created_at, updated_at,
                        last_error_code, last_error_message, error_classification, wait_reason
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, NULL, ?11, ?12, ?13, NULL, NULL, NULL, NULL)
                    ",
                    params![
                        task_id.0,
                        request.library_id.0,
                        input.task_type.as_str(),
                        TaskStatus::Queued.as_str(),
                        queue_position,
                        input.priority,
                        input.provider,
                        input.operation.map(operation_to_str),
                        input.concurrency_group,
                        input.max_attempts,
                        input.input_json,
                        now,
                        now,
                    ],
                )
                .map_err(database_error)?;
            insert_task_event(
                &transaction,
                &task_id,
                "submitted",
                None,
                Some("{\"source\":\"task_repository\"}"),
                &now,
            )?;
            created.push(load_task_summary(&transaction, &task_id)?);
        }

        transaction.commit().map_err(database_error)?;
        Ok(created)
    }

    fn list_tasks(&self, library_path: &Path) -> DomainResult<Vec<TaskSummary>> {
        let connection = Self::open_library_database(library_path)?;
        let mut statement = connection
            .prepare(
                "
                SELECT id, library_id, task_type, status, queue_position, priority,
                       provider, operation_type, concurrency_group, attempt_count,
                       max_attempts, next_retry_at, input_json, created_at, updated_at,
                       last_error_code, last_error_message, error_classification, wait_reason
                FROM tasks
                ORDER BY priority DESC, queue_position ASC, created_at ASC
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], task_summary_from_row)
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn get_task_detail(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskDetail> {
        let connection = Self::open_library_database(library_path)?;
        Ok(TaskDetail {
            task: load_task_summary(&connection, task_id)?,
            attempts: load_task_attempts(&connection, task_id)?,
            events: load_task_events(&connection, task_id)?,
            outputs: load_task_outputs(&connection, task_id)?,
        })
    }

    fn update_task_status(&self, request: UpdateTaskStatusRequest) -> DomainResult<TaskSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_task_exists(&connection, &request.task_id)?;
        let now = timestamp_string();
        connection
            .execute(
                "
                UPDATE tasks
                SET status = ?1,
                    next_retry_at = ?2,
                    last_error_code = ?3,
                    last_error_message = ?4,
                    error_classification = ?5,
                    wait_reason = ?6,
                    updated_at = ?7
                WHERE id = ?8
                ",
                params![
                    request.status.as_str(),
                    request.next_retry_at,
                    request.last_error_code,
                    request.last_error_message,
                    request
                        .error_classification
                        .map(TaskErrorClassification::as_str),
                    request.wait_reason,
                    now,
                    request.task_id.0,
                ],
            )
            .map_err(database_error)?;
        load_task_summary(&connection, &request.task_id)
    }

    fn append_task_event(&self, request: AppendTaskEventRequest) -> DomainResult<TaskEvent> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_task_exists(&connection, &request.task_id)?;
        let now = timestamp_string();
        let event_id = insert_task_event(
            &connection,
            &request.task_id,
            &request.event_type,
            request.message.as_deref(),
            request.payload_json.as_deref(),
            &now,
        )?;
        load_task_event(&connection, &event_id)
    }

    fn append_task_attempt(&self, request: AppendTaskAttemptRequest) -> DomainResult<TaskAttempt> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_task_exists(&connection, &request.task_id)?;
        let attempt_number = next_attempt_number(&connection, &request.task_id)?;
        let attempt_id = TaskAttemptId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO task_attempts (
                    id, task_id, attempt_number, status, started_at, completed_at,
                    log_path, exit_code, error_code, error_message, error_classification
                )
                VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, NULL, NULL, NULL, NULL)
                ",
                params![
                    attempt_id.0,
                    request.task_id.0,
                    attempt_number,
                    request.status,
                    now,
                    request
                        .log_path
                        .map(|path| path.to_string_lossy().to_string()),
                ],
            )
            .map_err(database_error)?;
        transaction
            .execute(
                "UPDATE tasks SET attempt_count = ?1, updated_at = ?2 WHERE id = ?3",
                params![attempt_number, now, request.task_id.0],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)?;
        load_task_attempt(&connection, &attempt_id)
    }

    fn complete_task_attempt(
        &self,
        request: CompleteTaskAttemptRequest,
    ) -> DomainResult<TaskAttempt> {
        let connection = Self::open_library_database(&request.library_path)?;
        let now = timestamp_string();
        let changed = connection
            .execute(
                "
                UPDATE task_attempts
                SET status = ?1,
                    completed_at = ?2,
                    exit_code = ?3,
                    error_code = ?4,
                    error_message = ?5,
                    error_classification = ?6
                WHERE id = ?7
                ",
                params![
                    request.status,
                    now,
                    request.exit_code,
                    request.error_code,
                    request.error_message,
                    request
                        .error_classification
                        .map(TaskErrorClassification::as_str),
                    request.attempt_id.0,
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(DomainError::InvalidTaskReference {
                id: request.attempt_id.0,
            });
        }
        load_task_attempt(&connection, &request.attempt_id)
    }

    fn append_task_output(&self, request: AppendTaskOutputRequest) -> DomainResult<TaskOutput> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_task_exists(&connection, &request.task_id)?;
        let now = timestamp_string();
        let output_id = TaskOutputId(Uuid::new_v4().to_string());
        connection
            .execute(
                "
                INSERT INTO task_outputs (
                    id, task_id, output_type, target_id, payload_json, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(task_id, output_type, target_id) DO NOTHING
                ",
                params![
                    output_id.0,
                    request.task_id.0,
                    request.output_type.as_str(),
                    request.target_id,
                    request.payload_json,
                    now,
                ],
            )
            .map_err(database_error)?;
        load_task_output_by_target(
            &connection,
            &request.task_id,
            request.output_type,
            &request.target_id,
        )
    }

    fn has_task_output(
        &self,
        library_path: &Path,
        task_id: &TaskId,
        output_type: TaskOutputType,
        target_id: &str,
    ) -> DomainResult<bool> {
        let connection = Self::open_library_database(library_path)?;
        let count: i64 = connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM task_outputs
                WHERE task_id = ?1 AND output_type = ?2 AND target_id = ?3
                ",
                params![task_id.0, output_type.as_str(), target_id],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        Ok(count > 0)
    }

    fn reorder_queued_tasks(&self, request: ReorderQueuedTasksRequest) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        let now = timestamp_string();
        for (index, task_id) in request.task_ids.iter().enumerate() {
            let changed = transaction
                .execute(
                    "
                    UPDATE tasks
                    SET queue_position = ?1, updated_at = ?2
                    WHERE id = ?3 AND status = ?4
                    ",
                    params![
                        index as i64 + 1,
                        now,
                        task_id.0,
                        TaskStatus::Queued.as_str(),
                    ],
                )
                .map_err(database_error)?;
            if changed == 0 {
                return Err(DomainError::InvalidTaskReference {
                    id: task_id.0.clone(),
                });
            }
        }
        transaction.commit().map_err(database_error)
    }

    fn retry_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary> {
        let connection = Self::open_library_database(library_path)?;
        ensure_task_exists(&connection, task_id)?;
        let now = timestamp_string();
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "
                UPDATE tasks
                SET status = ?1,
                    next_retry_at = NULL,
                    last_error_code = NULL,
                    last_error_message = NULL,
                    error_classification = NULL,
                    wait_reason = NULL,
                    updated_at = ?2
                WHERE id = ?3
                ",
                params![TaskStatus::Queued.as_str(), now, task_id.0],
            )
            .map_err(database_error)?;
        insert_task_event(
            &transaction,
            task_id,
            "manual_retry",
            Some("Task queued for manual retry"),
            None,
            &now,
        )?;
        transaction.commit().map_err(database_error)?;
        load_task_summary(&connection, task_id)
    }

    fn duplicate_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary> {
        let connection = Self::open_library_database(library_path)?;
        let source = load_task_summary(&connection, task_id)?;
        let duplicated_id = TaskId(Uuid::new_v4().to_string());
        let queue_position = next_queue_position(&connection)?;
        let now = timestamp_string();
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO tasks (
                    id, library_id, task_type, status, queue_position, priority,
                    provider, operation_type, concurrency_group, attempt_count,
                    max_attempts, next_retry_at, input_json, created_at, updated_at,
                    last_error_code, last_error_message, error_classification, wait_reason
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, NULL, ?11, ?12, ?13, NULL, NULL, NULL, NULL)
                ",
                params![
                    duplicated_id.0,
                    source.library_id.0,
                    source.task_type.as_str(),
                    TaskStatus::Queued.as_str(),
                    queue_position,
                    source.priority,
                    source.provider,
                    source.operation.map(operation_to_str),
                    source.concurrency_group,
                    source.max_attempts,
                    source.input_json,
                    now,
                    now,
                ],
            )
            .map_err(database_error)?;
        insert_task_event(
            &transaction,
            &duplicated_id,
            "duplicated",
            Some("Task duplicated from existing task"),
            Some(&format!("{{\"source_task_id\":\"{}\"}}", task_id.0)),
            &now,
        )?;
        transaction.commit().map_err(database_error)?;
        load_task_summary(&connection, &duplicated_id)
    }
}

fn next_queue_position(connection: &Connection) -> DomainResult<i64> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(queue_position), 0) + 1 FROM tasks",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)
}

fn next_attempt_number(connection: &Connection, task_id: &TaskId) -> DomainResult<u32> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(attempt_number), 0) + 1 FROM task_attempts WHERE task_id = ?1",
            params![task_id.0],
            |row| row.get::<_, u32>(0),
        )
        .map_err(database_error)
}

fn ensure_task_exists(connection: &Connection, task_id: &TaskId) -> DomainResult<()> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE id = ?1",
            params![task_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if count == 0 {
        return Err(DomainError::InvalidTaskReference {
            id: task_id.0.clone(),
        });
    }
    Ok(())
}

fn insert_task_event(
    connection: &Connection,
    task_id: &TaskId,
    event_type: &str,
    message: Option<&str>,
    payload_json: Option<&str>,
    created_at: &str,
) -> DomainResult<TaskEventId> {
    let event_id = TaskEventId(Uuid::new_v4().to_string());
    connection
        .execute(
            "
            INSERT INTO task_events (id, task_id, event_type, message, payload_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                event_id.0,
                task_id.0,
                event_type,
                message,
                payload_json,
                created_at,
            ],
        )
        .map_err(database_error)?;
    Ok(event_id)
}

fn load_task_summary(connection: &Connection, task_id: &TaskId) -> DomainResult<TaskSummary> {
    connection
        .query_row(
            "
            SELECT id, library_id, task_type, status, queue_position, priority,
                   provider, operation_type, concurrency_group, attempt_count,
                   max_attempts, next_retry_at, input_json, created_at, updated_at,
                   last_error_code, last_error_message, error_classification, wait_reason
            FROM tasks
            WHERE id = ?1
            ",
            params![task_id.0],
            task_summary_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidTaskReference {
                id: task_id.0.clone(),
            },
            other => database_error(other),
        })
}

fn task_summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskSummary> {
    let task_type_value: String = row.get(2)?;
    let status_value: String = row.get(3)?;
    let operation_value: Option<String> = row.get(7)?;
    let error_classification_value: Option<String> = row.get(17)?;
    Ok(TaskSummary {
        id: TaskId(row.get(0)?),
        library_id: crate::LibraryId(row.get(1)?),
        task_type: TaskType::from_str(&task_type_value).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                2,
                "task_type".to_string(),
                rusqlite::types::Type::Text,
            )
        })?,
        status: TaskStatus::from_str(&status_value).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(3, "status".to_string(), rusqlite::types::Type::Text)
        })?,
        queue_position: row.get(4)?,
        priority: row.get(5)?,
        provider: row.get(6)?,
        operation: operation_value
            .as_deref()
            .map(operation_from_str)
            .transpose()
            .map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    7,
                    "operation_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?,
        concurrency_group: row.get(8)?,
        attempt_count: row.get(9)?,
        max_attempts: row.get(10)?,
        next_retry_at: row.get(11)?,
        input_json: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        last_error_code: row.get(15)?,
        last_error_message: row.get(16)?,
        error_classification: optional_error_classification(error_classification_value, 17)?,
        wait_reason: row.get(18)?,
    })
}

fn load_task_attempts(connection: &Connection, task_id: &TaskId) -> DomainResult<Vec<TaskAttempt>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, task_id, attempt_number, status, started_at, completed_at, log_path,
                   exit_code, error_code, error_message, error_classification
            FROM task_attempts
            WHERE task_id = ?1
            ORDER BY attempt_number ASC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![task_id.0], task_attempt_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_task_attempt(
    connection: &Connection,
    attempt_id: &TaskAttemptId,
) -> DomainResult<TaskAttempt> {
    connection
        .query_row(
            "
            SELECT id, task_id, attempt_number, status, started_at, completed_at, log_path,
                   exit_code, error_code, error_message, error_classification
            FROM task_attempts
            WHERE id = ?1
            ",
            params![attempt_id.0],
            task_attempt_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidTaskReference {
                id: attempt_id.0.clone(),
            },
            other => database_error(other),
        })
}

fn task_attempt_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskAttempt> {
    let error_classification_value: Option<String> = row.get(10)?;
    Ok(TaskAttempt {
        id: TaskAttemptId(row.get(0)?),
        task_id: TaskId(row.get(1)?),
        attempt_number: row.get(2)?,
        status: row.get(3)?,
        started_at: row.get(4)?,
        completed_at: row.get(5)?,
        log_path: row.get::<_, Option<String>>(6)?.map(PathBuf::from),
        exit_code: row.get(7)?,
        error_code: row.get(8)?,
        error_message: row.get(9)?,
        error_classification: optional_error_classification(error_classification_value, 10)?,
    })
}

fn load_task_events(connection: &Connection, task_id: &TaskId) -> DomainResult<Vec<TaskEvent>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, task_id, event_type, message, payload_json, created_at
            FROM task_events
            WHERE task_id = ?1
            ORDER BY created_at ASC, id ASC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![task_id.0], task_event_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_task_event(connection: &Connection, event_id: &TaskEventId) -> DomainResult<TaskEvent> {
    connection
        .query_row(
            "
            SELECT id, task_id, event_type, message, payload_json, created_at
            FROM task_events
            WHERE id = ?1
            ",
            params![event_id.0],
            task_event_from_row,
        )
        .map_err(database_error)
}

fn task_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskEvent> {
    Ok(TaskEvent {
        id: TaskEventId(row.get(0)?),
        task_id: TaskId(row.get(1)?),
        event_type: row.get(2)?,
        message: row.get(3)?,
        payload_json: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn load_task_outputs(connection: &Connection, task_id: &TaskId) -> DomainResult<Vec<TaskOutput>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, task_id, output_type, target_id, payload_json, created_at
            FROM task_outputs
            WHERE task_id = ?1
            ORDER BY created_at ASC, id ASC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![task_id.0], task_output_from_row)
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_task_output_by_target(
    connection: &Connection,
    task_id: &TaskId,
    output_type: TaskOutputType,
    target_id: &str,
) -> DomainResult<TaskOutput> {
    connection
        .query_row(
            "
            SELECT id, task_id, output_type, target_id, payload_json, created_at
            FROM task_outputs
            WHERE task_id = ?1 AND output_type = ?2 AND target_id = ?3
            ",
            params![task_id.0, output_type.as_str(), target_id],
            task_output_from_row,
        )
        .map_err(database_error)
}

fn task_output_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskOutput> {
    let output_type_value: String = row.get(2)?;
    Ok(TaskOutput {
        id: TaskOutputId(row.get(0)?),
        task_id: TaskId(row.get(1)?),
        output_type: TaskOutputType::from_str(&output_type_value).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                2,
                "output_type".to_string(),
                rusqlite::types::Type::Text,
            )
        })?,
        target_id: row.get(3)?,
        payload_json: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn optional_error_classification(
    value: Option<String>,
    index: usize,
) -> rusqlite::Result<Option<TaskErrorClassification>> {
    value
        .as_deref()
        .map(|classification| {
            TaskErrorClassification::from_str(classification).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    index,
                    "error_classification".to_string(),
                    rusqlite::types::Type::Text,
                )
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CreateLibraryRequest, GenerationOperation, LibraryService};
    use std::fs;

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("imglab-task-{name}-{}", Uuid::new_v4()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove existing root");
        }
        root
    }

    fn create_library(name: &str) -> (LocalLibraryService, PathBuf, crate::LibraryId) {
        let root = test_root(name);
        let registry = test_root(&format!("{name}-registry")).join("registry.sqlite");
        let service = LocalLibraryService::new(registry);
        let library = service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: name.to_string(),
            })
            .expect("create library");
        (service, root, library.id)
    }

    #[test]
    fn task_repository_creates_and_loads_detail() {
        let (service, root, library_id) = create_library("repo-detail");
        let tasks = service
            .create_tasks(crate::BatchCreateTasksRequest {
                library_path: root.clone(),
                library_id,
                tasks: vec![crate::CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 10,
                    concurrency_group: Some("fake".to_string()),
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"line one\\nline two\"}".to_string(),
                }],
            })
            .expect("create tasks");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Queued);
        assert_eq!(tasks[0].queue_position, 1);

        let event = service
            .append_task_event(crate::AppendTaskEventRequest {
                library_path: root.clone(),
                task_id: tasks[0].id.clone(),
                event_type: "scheduled".to_string(),
                message: Some("ready".to_string()),
                payload_json: None,
            })
            .expect("append event");
        assert_eq!(event.event_type, "scheduled");

        let attempt = service
            .append_task_attempt(crate::AppendTaskAttemptRequest {
                library_path: root.clone(),
                task_id: tasks[0].id.clone(),
                status: "running".to_string(),
                log_path: Some(PathBuf::from("/tmp/imglab-task.log")),
            })
            .expect("append attempt");
        assert_eq!(attempt.attempt_number, 1);

        let attempt = service
            .complete_task_attempt(crate::CompleteTaskAttemptRequest {
                library_path: root.clone(),
                attempt_id: attempt.id,
                status: "completed".to_string(),
                exit_code: Some(0),
                error_code: None,
                error_message: None,
                error_classification: None,
            })
            .expect("complete attempt");
        assert_eq!(attempt.status, "completed");

        let output = service
            .append_task_output(crate::AppendTaskOutputRequest {
                library_path: root.clone(),
                task_id: tasks[0].id.clone(),
                output_type: TaskOutputType::AssetVersion,
                target_id: "version-1".to_string(),
                payload_json: Some("{\"kind\":\"test\"}".to_string()),
            })
            .expect("append output");
        assert_eq!(output.target_id, "version-1");

        let detail = service
            .get_task_detail(&root, &tasks[0].id)
            .expect("task detail");
        assert_eq!(detail.task.attempt_count, 1);
        assert_eq!(detail.events.len(), 2);
        assert_eq!(detail.attempts.len(), 1);
        assert_eq!(detail.outputs.len(), 1);
    }

    #[test]
    fn task_outputs_are_idempotent() {
        let (service, root, library_id) = create_library("repo-outputs");
        let task = service
            .create_tasks(crate::BatchCreateTasksRequest {
                library_path: root.clone(),
                library_id,
                tasks: vec![crate::CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 3,
                    input_json: "{}".to_string(),
                }],
            })
            .expect("create task")
            .remove(0);

        let first = service
            .append_task_output(crate::AppendTaskOutputRequest {
                library_path: root.clone(),
                task_id: task.id.clone(),
                output_type: TaskOutputType::GenerationEvent,
                target_id: "event-1".to_string(),
                payload_json: None,
            })
            .expect("append first");
        let second = service
            .append_task_output(crate::AppendTaskOutputRequest {
                library_path: root.clone(),
                task_id: task.id.clone(),
                output_type: TaskOutputType::GenerationEvent,
                target_id: "event-1".to_string(),
                payload_json: None,
            })
            .expect("append second");

        assert_eq!(first.id, second.id);
        assert!(service
            .has_task_output(&root, &task.id, TaskOutputType::GenerationEvent, "event-1")
            .expect("has output"));
        assert_eq!(
            service
                .get_task_detail(&root, &task.id)
                .expect("detail")
                .outputs
                .len(),
            1
        );
    }

    #[test]
    fn reorder_rejects_non_queued_tasks() {
        let (service, root, library_id) = create_library("repo-reorder");
        let tasks = service
            .create_tasks(crate::BatchCreateTasksRequest {
                library_path: root.clone(),
                library_id,
                tasks: vec![
                    crate::CreateTaskInput {
                        task_type: TaskType::ImageGeneration,
                        provider: Some("fake".to_string()),
                        operation: Some(GenerationOperation::TextToImage),
                        priority: 0,
                        concurrency_group: None,
                        max_attempts: 3,
                        input_json: "{\"n\":1}".to_string(),
                    },
                    crate::CreateTaskInput {
                        task_type: TaskType::ImageGeneration,
                        provider: Some("fake".to_string()),
                        operation: Some(GenerationOperation::TextToImage),
                        priority: 0,
                        concurrency_group: None,
                        max_attempts: 3,
                        input_json: "{\"n\":2}".to_string(),
                    },
                ],
            })
            .expect("create tasks");

        service
            .reorder_queued_tasks(crate::ReorderQueuedTasksRequest {
                library_path: root.clone(),
                task_ids: vec![tasks[1].id.clone(), tasks[0].id.clone()],
            })
            .expect("reorder queued");
        let ordered = service.list_tasks(&root).expect("list tasks");
        assert_eq!(ordered[0].id, tasks[1].id);

        service
            .update_task_status(crate::UpdateTaskStatusRequest {
                library_path: root.clone(),
                task_id: tasks[0].id.clone(),
                status: TaskStatus::Running,
                next_retry_at: None,
                last_error_code: None,
                last_error_message: None,
                error_classification: None,
                wait_reason: None,
            })
            .expect("mark running");
        let error = service
            .reorder_queued_tasks(crate::ReorderQueuedTasksRequest {
                library_path: root,
                task_ids: vec![tasks[0].id.clone()],
            })
            .expect_err("running task cannot be reordered");
        assert!(matches!(error, DomainError::InvalidTaskReference { .. }));
    }

    #[test]
    fn retry_and_duplicate_preserve_task_history_boundaries() {
        let (service, root, library_id) = create_library("repo-retry-duplicate");
        let task = service
            .create_tasks(crate::BatchCreateTasksRequest {
                library_path: root.clone(),
                library_id,
                tasks: vec![crate::CreateTaskInput {
                    task_type: TaskType::MetadataFieldGeneration,
                    provider: Some("codex-cli".to_string()),
                    operation: None,
                    priority: 5,
                    concurrency_group: Some("codex-cli".to_string()),
                    max_attempts: 3,
                    input_json: "{\"field\":\"title\"}".to_string(),
                }],
            })
            .expect("create task")
            .remove(0);
        service
            .append_task_attempt(crate::AppendTaskAttemptRequest {
                library_path: root.clone(),
                task_id: task.id.clone(),
                status: "failed".to_string(),
                log_path: None,
            })
            .expect("append attempt");
        service
            .update_task_status(crate::UpdateTaskStatusRequest {
                library_path: root.clone(),
                task_id: task.id.clone(),
                status: TaskStatus::FailedRetryable,
                next_retry_at: Some("2026-01-01T00:00:30Z".to_string()),
                last_error_code: Some("Timeout".to_string()),
                last_error_message: Some("temporary timeout".to_string()),
                error_classification: Some(TaskErrorClassification::Transient),
                wait_reason: Some("Waiting until retry backoff expires".to_string()),
            })
            .expect("mark failed");

        let retried = service.retry_task(&root, &task.id).expect("retry task");
        assert_eq!(retried.id, task.id);
        assert_eq!(retried.status, TaskStatus::Queued);
        assert_eq!(retried.attempt_count, 1);
        assert!(retried.last_error_code.is_none());

        let duplicated = service.duplicate_task(&root, &task.id).expect("duplicate");
        assert_ne!(duplicated.id, task.id);
        assert_eq!(duplicated.status, TaskStatus::Queued);
        assert_eq!(duplicated.attempt_count, 0);
        assert_eq!(duplicated.input_json, task.input_json);
        assert_eq!(duplicated.priority, task.priority);
    }
}
