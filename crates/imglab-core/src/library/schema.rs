use super::database_error;
use crate::{DomainError, DomainResult};
use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: u32 = 6;

pub fn migrate_library_database(connection: &Connection) -> DomainResult<()> {
    let user_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(database_error)?;

    if user_version > CURRENT_SCHEMA_VERSION {
        return Err(DomainError::SchemaMismatch {
            expected: CURRENT_SCHEMA_VERSION,
            found: user_version,
        });
    }

    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                library_id TEXT NOT NULL,
                media_type TEXT NOT NULL,
                title TEXT,
                description TEXT,
                schema_prompt TEXT,
                category TEXT,
                rating INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                captured_at TEXT
            );

            CREATE TABLE IF NOT EXISTS asset_versions (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                parent_version_id TEXT,
                generation_event_id TEXT,
                file_path TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                checksum_algorithm TEXT NOT NULL DEFAULT 'SHA-256',
                checksum TEXT,
                width INTEGER,
                height INTEGER,
                mime_type TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                version_label TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(asset_id) REFERENCES assets(id)
            );

            CREATE TABLE IF NOT EXISTS generation_events (
                id TEXT PRIMARY KEY,
                asset_id TEXT,
                output_version_id TEXT,
                provider TEXT NOT NULL,
                provider_model TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                input_asset_version_id TEXT,
                parameters_json TEXT NOT NULL,
                raw_request_json TEXT,
                raw_response_json TEXT,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                error_code TEXT,
                error_message TEXT
            );

            CREATE TABLE IF NOT EXISTS metadata_suggestions (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                source TEXT NOT NULL,
                suggested_title TEXT,
                suggested_description TEXT,
                suggested_schema_prompt TEXT,
                suggested_tags_json TEXT NOT NULL,
                suggested_category TEXT,
                confidence_json TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                reviewed_at TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id)
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS asset_tags (
                asset_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                source TEXT NOT NULL,
                confirmed_at TEXT,
                PRIMARY KEY(asset_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                kind TEXT NOT NULL,
                smart_query_json TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS album_items (
                album_id TEXT NOT NULL,
                asset_id TEXT NOT NULL,
                sort_order INTEGER NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY(album_id, asset_id)
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                library_id TEXT NOT NULL,
                task_type TEXT NOT NULL,
                status TEXT NOT NULL,
                queue_position INTEGER NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                provider TEXT,
                operation_type TEXT,
                concurrency_group TEXT,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                max_attempts INTEGER NOT NULL DEFAULT 3,
                next_retry_at TEXT,
                input_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_error_code TEXT,
                last_error_message TEXT,
                error_classification TEXT,
                wait_reason TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_library_status_order
                ON tasks(library_id, status, priority DESC, queue_position ASC, created_at ASC);

            CREATE TABLE IF NOT EXISTS task_attempts (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                attempt_number INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                log_path TEXT,
                exit_code INTEGER,
                error_code TEXT,
                error_message TEXT,
                error_classification TEXT,
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            );

            CREATE INDEX IF NOT EXISTS idx_task_attempts_task
                ON task_attempts(task_id, attempt_number ASC);

            CREATE TABLE IF NOT EXISTS task_events (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                message TEXT,
                payload_json TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            );

            CREATE INDEX IF NOT EXISTS idx_task_events_task_created
                ON task_events(task_id, created_at ASC);

            CREATE TABLE IF NOT EXISTS task_outputs (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                output_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                payload_json TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(task_id, output_type, target_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            );

            CREATE INDEX IF NOT EXISTS idx_task_outputs_task
                ON task_outputs(task_id);

            CREATE INDEX IF NOT EXISTS idx_assets_library_id
                ON assets(library_id);

            CREATE INDEX IF NOT EXISTS idx_asset_versions_asset_created
                ON asset_versions(asset_id, created_at DESC, id DESC);

            CREATE INDEX IF NOT EXISTS idx_asset_versions_generation_event
                ON asset_versions(generation_event_id);

            CREATE INDEX IF NOT EXISTS idx_generation_events_asset_started
                ON generation_events(asset_id, started_at DESC, id DESC);

            CREATE INDEX IF NOT EXISTS idx_metadata_suggestions_asset_status
                ON metadata_suggestions(asset_id, status);

            CREATE INDEX IF NOT EXISTS idx_album_items_asset
                ON album_items(asset_id);

            CREATE INDEX IF NOT EXISTS idx_album_items_album_sort
                ON album_items(album_id, sort_order ASC, asset_id);

            CREATE INDEX IF NOT EXISTS idx_asset_tags_asset
                ON asset_tags(asset_id);

            CREATE INDEX IF NOT EXISTS idx_asset_tags_tag
                ON asset_tags(tag_id);

            ",
        )
        .map_err(database_error)?;

    if !column_exists(connection, "asset_versions", "checksum_algorithm")? {
        connection
            .execute(
                "ALTER TABLE asset_versions ADD COLUMN checksum_algorithm TEXT NOT NULL DEFAULT 'SHA-256'",
                [],
            )
            .map_err(database_error)?;
    }
    if !column_exists(connection, "asset_versions", "checksum")? {
        connection
            .execute("ALTER TABLE asset_versions ADD COLUMN checksum TEXT", [])
            .map_err(database_error)?;
    }
    if !column_exists(connection, "asset_versions", "version_number")? {
        connection
            .execute(
                "ALTER TABLE asset_versions ADD COLUMN version_number INTEGER",
                [],
            )
            .map_err(database_error)?;
    }
    if !column_exists(connection, "assets", "schema_prompt")? {
        connection
            .execute("ALTER TABLE assets ADD COLUMN schema_prompt TEXT", [])
            .map_err(database_error)?;
    }
    if !column_exists(
        connection,
        "metadata_suggestions",
        "suggested_schema_prompt",
    )? {
        connection
            .execute(
                "ALTER TABLE metadata_suggestions ADD COLUMN suggested_schema_prompt TEXT",
                [],
            )
            .map_err(database_error)?;
    }
    if !column_exists(connection, "albums", "sort_order")? {
        connection
            .execute(
                "ALTER TABLE albums ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .map_err(database_error)?;
        backfill_album_sort_order(connection)?;
    }
    connection
        .execute(
            "UPDATE asset_versions SET checksum = sha256 WHERE checksum IS NULL",
            [],
        )
        .map_err(database_error)?;
    backfill_asset_version_numbers(connection)?;
    connection
        .execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_asset_versions_asset_version_number
                ON asset_versions(asset_id, version_number)
            ",
            [],
        )
        .map_err(database_error)?;
    connection
        .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)
        .map_err(database_error)?;

    Ok(())
}

fn backfill_asset_version_numbers(connection: &Connection) -> DomainResult<()> {
    let asset_ids = {
        let mut statement = connection
            .prepare("SELECT DISTINCT asset_id FROM asset_versions ORDER BY asset_id")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?
    };

    for asset_id in asset_ids {
        let version_ids = {
            let mut statement = connection
                .prepare(
                    "
                    SELECT id
                    FROM asset_versions
                    WHERE asset_id = ?1
                    ORDER BY created_at ASC, id ASC
                    ",
                )
                .map_err(database_error)?;
            let rows = statement
                .query_map([asset_id.as_str()], |row| row.get::<_, String>(0))
                .map_err(database_error)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(database_error)?
        };

        for (index, version_id) in version_ids.iter().enumerate() {
            connection
                .execute(
                    "
                    UPDATE asset_versions
                    SET version_number = COALESCE(version_number, ?1)
                    WHERE id = ?2
                    ",
                    rusqlite::params![index as i64 + 1, version_id],
                )
                .map_err(database_error)?;
        }
    }

    let missing_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM asset_versions WHERE version_number IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if missing_count > 0 {
        return Err(DomainError::Database {
            message: format!("failed to backfill version_number for {missing_count} versions"),
        });
    }

    Ok(())
}

fn backfill_album_sort_order(connection: &Connection) -> DomainResult<()> {
    let album_ids = {
        let mut statement = connection
            .prepare("SELECT id FROM albums ORDER BY name, created_at, id")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?
    };

    for (index, album_id) in album_ids.iter().enumerate() {
        connection
            .execute(
                "UPDATE albums SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![index as i64 + 1, album_id],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> DomainResult<bool> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?;
    for row in rows {
        if row.map_err(database_error)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}
