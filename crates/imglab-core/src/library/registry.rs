use super::{database_error, io_error, storage::timestamp_string, LocalLibraryService};
use crate::{
    domain::library::RegistryAlias, DomainError, DomainResult, LibraryId, LibrarySummary,
    RenameLibraryAliasRequest,
};
use rusqlite::{params, Connection, Row};
use std::{fs, path::PathBuf};

impl LocalLibraryService {
    pub(super) fn ensure_registry(&self) -> DomainResult<Connection> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let connection = Connection::open(&self.registry_path).map_err(database_error)?;
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS library_registry (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    root_path TEXT NOT NULL UNIQUE,
                    hidden INTEGER NOT NULL DEFAULT 0,
                    schema_version INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    last_opened_at TEXT NOT NULL
                );
                ",
            )
            .map_err(database_error)?;

        Ok(connection)
    }

    pub(super) fn upsert_registry(
        &self,
        summary: &LibrarySummary,
        created_at: &str,
        hidden: bool,
    ) -> DomainResult<()> {
        let connection = self.ensure_registry()?;
        let now = timestamp_string();

        connection
            .execute(
                "
                INSERT INTO library_registry (
                    id, name, root_path, hidden, schema_version, created_at, last_opened_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(root_path) DO UPDATE SET
                    name = excluded.name,
                    hidden = excluded.hidden,
                    schema_version = excluded.schema_version,
                    last_opened_at = excluded.last_opened_at
                ",
                params![
                    summary.id.0,
                    summary.name,
                    summary.root_path.to_string_lossy(),
                    if hidden { 1 } else { 0 },
                    summary.schema_version,
                    created_at,
                    now
                ],
            )
            .map_err(database_error)?;

        Ok(())
    }

    pub(crate) fn registry_contains_library_id(&self, library_id: &str) -> DomainResult<bool> {
        let connection = self.ensure_registry()?;
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM library_registry WHERE id = ?1",
                params![library_id],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        Ok(count > 0)
    }

    pub(super) fn list_registry_libraries(
        &self,
        include_hidden: bool,
    ) -> DomainResult<Vec<LibrarySummary>> {
        let connection = self.ensure_registry()?;
        let mut statement = connection
            .prepare(
                "
                SELECT id, name, root_path, hidden, schema_version
                FROM library_registry
                WHERE ?1 OR hidden = 0
                ORDER BY last_opened_at DESC
                ",
            )
            .map_err(database_error)?;

        let rows = statement
            .query_map(params![include_hidden], registry_summary_from_row)
            .map_err(database_error)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    pub(super) fn hide_registry_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        let connection = self.ensure_registry()?;
        let updated = connection
            .execute(
                "UPDATE library_registry SET hidden = 1 WHERE id = ?1",
                params![library_id.0],
            )
            .map_err(database_error)?;

        if updated == 0 {
            return Err(DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            });
        }

        Ok(())
    }

    pub(super) fn rename_registry_alias(
        &self,
        request: RenameLibraryAliasRequest,
    ) -> DomainResult<LibrarySummary> {
        let library_id = request.library_id.0;
        let alias = RegistryAlias::parse(&request.alias)?;

        let connection = self.ensure_registry()?;
        let updated = connection
            .execute(
                "UPDATE library_registry SET name = ?1 WHERE id = ?2",
                params![alias.as_str(), &library_id],
            )
            .map_err(database_error)?;
        if updated == 0 {
            return Err(DomainError::LibraryNotFound { path: library_id });
        }

        let mut statement = connection
            .prepare(
                "
                SELECT id, name, root_path, hidden, schema_version
                FROM library_registry
                WHERE id = ?1
                ",
            )
            .map_err(database_error)?;
        statement
            .query_row(params![&library_id], registry_summary_from_row)
            .map_err(database_error)
    }

    pub(super) fn unregister_registry_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        let connection = self.ensure_registry()?;
        let updated = connection
            .execute(
                "DELETE FROM library_registry WHERE id = ?1",
                params![library_id.0],
            )
            .map_err(database_error)?;

        if updated == 0 {
            return Err(DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            });
        }

        Ok(())
    }

    pub(super) fn registry_counts(&self) -> DomainResult<(u32, u32)> {
        let libraries = self.list_registry_libraries(true)?;
        let registered_library_count = libraries.len() as u32;
        let missing_library_count = libraries
            .iter()
            .filter(|library| !Self::manifest_path(&library.root_path).is_file())
            .count() as u32;
        Ok((registered_library_count, missing_library_count))
    }
}

fn registry_summary_from_row(row: &Row<'_>) -> rusqlite::Result<LibrarySummary> {
    Ok(LibrarySummary {
        id: LibraryId(row.get(0)?),
        name: row.get(1)?,
        root_path: PathBuf::from(row.get::<_, String>(2)?),
        hidden: row.get::<_, i64>(3)? != 0,
        schema_version: row.get::<_, u32>(4)?,
    })
}
