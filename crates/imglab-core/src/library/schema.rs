use super::database_error;
use crate::{DomainError, DomainResult};
use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: u32 = 4;

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
    connection
        .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)
        .map_err(database_error)?;

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
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)?
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
