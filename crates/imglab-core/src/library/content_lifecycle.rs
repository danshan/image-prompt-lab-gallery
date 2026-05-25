use super::{database_error, timestamp_string, LocalLibraryService};
use crate::{
    ArchiveAssetRequest, ArchivePromptDocumentRequest, ArchivedContentSummary, ArchivedContentType,
    DomainError, DomainResult, ListArchivedContentRequest, PermanentDeleteArchivedContentRequest,
    PermanentDeleteSummary, RestoreAssetRequest, RestorePromptDocumentRequest,
};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

impl LocalLibraryService {
    pub fn archive_asset(&self, request: ArchiveAssetRequest) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let changed = connection
            .execute(
                "UPDATE assets SET archived_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![timestamp_string(), request.asset_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: request.asset_id.0,
            });
        }
        Ok(())
    }

    pub fn restore_asset(&self, request: RestoreAssetRequest) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let changed = connection
            .execute(
                "UPDATE assets SET archived_at = NULL, updated_at = ?1 WHERE id = ?2",
                params![timestamp_string(), request.asset_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: request.asset_id.0,
            });
        }
        Ok(())
    }

    pub fn archive_prompt_document(
        &self,
        request: ArchivePromptDocumentRequest,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let now = timestamp_string();
        let changed = connection
            .execute(
                "
                UPDATE prompt_documents
                SET status = 'archived', archived_at = ?1, updated_at = ?1
                WHERE id = ?2
                ",
                params![now, request.prompt_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_prompt_reference(request.prompt_id.0));
        }
        Ok(())
    }

    pub fn restore_prompt_document(
        &self,
        request: RestorePromptDocumentRequest,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let changed = connection
            .execute(
                "
                UPDATE prompt_documents
                SET status = 'active', archived_at = NULL, updated_at = ?1
                WHERE id = ?2
                ",
                params![timestamp_string(), request.prompt_id.0],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err(invalid_prompt_reference(request.prompt_id.0));
        }
        Ok(())
    }

    pub fn list_archived_content(
        &self,
        request: ListArchivedContentRequest,
    ) -> DomainResult<Vec<ArchivedContentSummary>> {
        let connection = Self::open_library_database(&request.library_path)?;
        let mut items = Vec::new();
        if matches!(request.item_type, None | Some(ArchivedContentType::Asset)) {
            items.extend(list_archived_assets(&connection, &request.library_path)?);
        }
        if matches!(request.item_type, None | Some(ArchivedContentType::Prompt)) {
            items.extend(list_archived_prompts(&connection)?);
        }
        items.sort_by(|left, right| right.archived_at.cmp(&left.archived_at));
        Ok(items)
    }

    pub fn dry_run_permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        permanent_delete_plan(&connection, &request.library_path, &request)
    }

    pub fn permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let plan = permanent_delete_plan(&connection, &request.library_path, &request)?;
        let files = permanent_delete_files(&connection, &request.library_path, &request)?;
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        match request.item_type {
            ArchivedContentType::Asset => delete_archived_asset_rows(&transaction, &request.id)?,
            ArchivedContentType::Prompt => delete_archived_prompt_rows(&transaction, &request.id)?,
        }
        transaction.commit().map_err(database_error)?;

        let mut warnings = plan.warnings;
        for relative_path in files {
            let path = request.library_path.join(&relative_path);
            if let Err(error) = std::fs::remove_file(&path) {
                if path.exists() {
                    warnings.push(format!("failed to delete {}: {}", path.display(), error));
                }
            }
        }

        Ok(PermanentDeleteSummary { warnings, ..plan })
    }
}

fn permanent_delete_plan(
    connection: &Connection,
    library_path: &Path,
    request: &PermanentDeleteArchivedContentRequest,
) -> DomainResult<PermanentDeleteSummary> {
    match request.item_type {
        ArchivedContentType::Asset => asset_delete_plan(connection, library_path, &request.id),
        ArchivedContentType::Prompt => prompt_delete_plan(connection, &request.id),
    }
}

fn asset_delete_plan(
    connection: &Connection,
    library_path: &Path,
    asset_id: &str,
) -> DomainResult<PermanentDeleteSummary> {
    ensure_asset_archived(connection, asset_id)?;
    let files = asset_version_files(connection, asset_id)?;
    let file_size_bytes = files
        .iter()
        .filter_map(|relative| std::fs::metadata(library_path.join(relative)).ok())
        .map(|metadata| metadata.len())
        .sum();
    let version_count = files.len() as u32;
    let generation_count = count_rows(
        connection,
        "SELECT COUNT(*) FROM generation_events WHERE asset_id = ?1 OR output_version_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1)",
        asset_id,
    )?;
    let suggestion_count = count_rows(
        connection,
        "SELECT COUNT(*) FROM metadata_suggestions WHERE asset_id = ?1",
        asset_id,
    )?;
    let album_count = count_rows(
        connection,
        "SELECT COUNT(*) FROM album_items WHERE asset_id = ?1",
        asset_id,
    )?;
    Ok(PermanentDeleteSummary {
        item_id: asset_id.to_string(),
        item_type: ArchivedContentType::Asset,
        sqlite_row_count: 1 + version_count + generation_count + suggestion_count + album_count,
        file_count: files.len() as u32,
        file_size_bytes,
        warnings: Vec::new(),
    })
}

fn prompt_delete_plan(
    connection: &Connection,
    prompt_id: &str,
) -> DomainResult<PermanentDeleteSummary> {
    ensure_prompt_archived(connection, prompt_id)?;
    let version_count = count_rows(
        connection,
        "SELECT COUNT(*) FROM prompt_versions WHERE prompt_id = ?1",
        prompt_id,
    )?;
    let event_count = count_rows(
        connection,
        "SELECT COUNT(*) FROM generation_events WHERE prompt_version_id IN (SELECT id FROM prompt_versions WHERE prompt_id = ?1)",
        prompt_id,
    )?;
    Ok(PermanentDeleteSummary {
        item_id: prompt_id.to_string(),
        item_type: ArchivedContentType::Prompt,
        sqlite_row_count: 1 + version_count + event_count,
        file_count: 0,
        file_size_bytes: 0,
        warnings: Vec::new(),
    })
}

fn permanent_delete_files(
    connection: &Connection,
    _library_path: &Path,
    request: &PermanentDeleteArchivedContentRequest,
) -> DomainResult<Vec<PathBuf>> {
    match request.item_type {
        ArchivedContentType::Asset => asset_version_files(connection, &request.id),
        ArchivedContentType::Prompt => Ok(Vec::new()),
    }
}

fn ensure_asset_archived(connection: &Connection, asset_id: &str) -> DomainResult<()> {
    let archived_at: Option<String> = connection
        .query_row(
            "SELECT archived_at FROM assets WHERE id = ?1",
            params![asset_id],
            |row| row.get(0),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidAssetReference {
                id: asset_id.to_string(),
            },
            other => database_error(other),
        })?;
    if archived_at.is_none() {
        return Err(DomainError::InvalidGenerationParameters {
            message: "permanent delete requires archived content".to_string(),
        });
    }
    Ok(())
}

fn ensure_prompt_archived(connection: &Connection, prompt_id: &str) -> DomainResult<()> {
    let archived_at: Option<String> = connection
        .query_row(
            "SELECT archived_at FROM prompt_documents WHERE id = ?1 AND status = 'archived'",
            params![prompt_id],
            |row| row.get(0),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => invalid_prompt_reference(prompt_id.to_string()),
            other => database_error(other),
        })?;
    if archived_at.is_none() {
        return Err(DomainError::InvalidGenerationParameters {
            message: "permanent delete requires archived content".to_string(),
        });
    }
    Ok(())
}

fn delete_archived_asset_rows(connection: &Connection, asset_id: &str) -> DomainResult<()> {
    connection
        .execute(
            "DELETE FROM task_outputs WHERE target_id = ?1 OR target_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1) OR target_id IN (SELECT id FROM generation_events WHERE asset_id = ?1)",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM scheduled_generation_run_outputs WHERE asset_id = ?1 OR asset_version_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1) OR generation_event_id IN (SELECT id FROM generation_events WHERE asset_id = ?1)",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM album_items WHERE asset_id = ?1",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM asset_tags WHERE asset_id = ?1",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM metadata_suggestions WHERE asset_id = ?1",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM asset_version_sources WHERE target_version_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1) OR source_asset_id = ?1 OR source_version_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1)",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM generation_events WHERE asset_id = ?1 OR output_version_id IN (SELECT id FROM asset_versions WHERE asset_id = ?1)",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM asset_versions WHERE asset_id = ?1",
            params![asset_id],
        )
        .map_err(database_error)?;
    connection
        .execute("DELETE FROM assets WHERE id = ?1", params![asset_id])
        .map_err(database_error)?;
    Ok(())
}

fn delete_archived_prompt_rows(connection: &Connection, prompt_id: &str) -> DomainResult<()> {
    connection
        .execute(
            "UPDATE generation_events SET prompt_version_id = NULL WHERE prompt_version_id IN (SELECT id FROM prompt_versions WHERE prompt_id = ?1)",
            params![prompt_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM prompt_versions WHERE prompt_id = ?1",
            params![prompt_id],
        )
        .map_err(database_error)?;
    connection
        .execute(
            "DELETE FROM prompt_documents WHERE id = ?1",
            params![prompt_id],
        )
        .map_err(database_error)?;
    Ok(())
}

fn count_rows(connection: &Connection, sql: &str, id: &str) -> DomainResult<u32> {
    let count: i64 = connection
        .query_row(sql, params![id], |row| row.get(0))
        .map_err(database_error)?;
    Ok(count as u32)
}

fn list_archived_assets(
    connection: &Connection,
    library_path: &Path,
) -> DomainResult<Vec<ArchivedContentSummary>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, COALESCE(title, id), archived_at
            FROM assets
            WHERE archived_at IS NOT NULL
            ORDER BY archived_at DESC, id DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(database_error)?;

    let mut summaries = Vec::new();
    for row in rows {
        let (id, title, archived_at) = row.map_err(database_error)?;
        let files = asset_version_files(connection, &id)?;
        let file_size_bytes = files
            .iter()
            .filter_map(|relative| std::fs::metadata(library_path.join(relative)).ok())
            .map(|metadata| metadata.len())
            .sum();
        summaries.push(ArchivedContentSummary {
            id,
            item_type: ArchivedContentType::Asset,
            title,
            archived_at,
            dependency_summary: format!("{} version(s)", files.len()),
            file_count: files.len() as u32,
            file_size_bytes,
        });
    }
    Ok(summaries)
}

fn asset_version_files(connection: &Connection, asset_id: &str) -> DomainResult<Vec<PathBuf>> {
    let mut statement = connection
        .prepare("SELECT file_path FROM asset_versions WHERE asset_id = ?1")
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id], |row| {
            Ok(PathBuf::from(row.get::<_, String>(0)?))
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn list_archived_prompts(connection: &Connection) -> DomainResult<Vec<ArchivedContentSummary>> {
    let mut statement = connection
        .prepare(
            "
            SELECT pd.id, pd.name, pd.archived_at, COUNT(pv.id)
            FROM prompt_documents pd
            LEFT JOIN prompt_versions pv ON pv.prompt_id = pd.id
            WHERE pd.status = 'archived' AND pd.archived_at IS NOT NULL
            GROUP BY pd.id, pd.name, pd.archived_at
            ORDER BY pd.archived_at DESC, pd.id DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let version_count = row.get::<_, u32>(3)?;
            Ok(ArchivedContentSummary {
                id: row.get(0)?,
                item_type: ArchivedContentType::Prompt,
                title: row.get(1)?,
                archived_at: row.get(2)?,
                dependency_summary: format!("{version_count} version(s)"),
                file_count: 0,
                file_size_bytes: 0,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn invalid_prompt_reference(id: String) -> DomainError {
    DomainError::InvalidGenerationParameters {
        message: format!("invalid prompt reference: {id}"),
    }
}
