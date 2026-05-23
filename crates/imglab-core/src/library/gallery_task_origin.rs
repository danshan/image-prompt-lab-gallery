use super::{database_error, operation_from_str};
use crate::{DomainResult, TaskId, TaskOriginView, TaskStatus, TaskType};
use rusqlite::Connection;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(super) struct TaskOrigins {
    pub(super) by_asset: HashMap<String, TaskOriginView>,
    pub(super) by_version: HashMap<String, TaskOriginView>,
}

pub(super) fn load_task_origins(connection: &Connection) -> DomainResult<TaskOrigins> {
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
                .map(operation_from_str)
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
