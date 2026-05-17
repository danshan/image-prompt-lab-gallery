use super::database_error;
use crate::{AlbumId, DomainResult};
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(super) struct ExportVersionRow {
    pub(super) asset_id: String,
    pub(super) version_id: String,
    pub(super) file_path: PathBuf,
    pub(super) checksum_algorithm: String,
    pub(super) checksum: String,
    pub(super) mime_type: String,
}

pub(super) fn load_export_versions(
    connection: &Connection,
    album_id: Option<&AlbumId>,
) -> DomainResult<Vec<ExportVersionRow>> {
    let sql = if album_id.is_some() {
        "
        SELECT av.asset_id, av.id, av.file_path,
               av.checksum_algorithm, COALESCE(av.checksum, av.sha256), av.mime_type
        FROM asset_versions av
        INNER JOIN album_items ai ON ai.asset_id = av.asset_id
        WHERE ai.album_id = ?1
        ORDER BY ai.sort_order, av.created_at
        "
    } else {
        "
        SELECT av.asset_id, av.id, av.file_path,
               av.checksum_algorithm, COALESCE(av.checksum, av.sha256), av.mime_type
        FROM asset_versions av
        ORDER BY av.created_at
        "
    };

    let mut statement = connection.prepare(sql).map_err(database_error)?;
    let rows = if let Some(album_id) = album_id {
        statement
            .query_map(params![album_id.0], export_version_from_row)
            .map_err(database_error)?
    } else {
        statement
            .query_map([], export_version_from_row)
            .map_err(database_error)?
    };

    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn export_version_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportVersionRow> {
    Ok(ExportVersionRow {
        asset_id: row.get(0)?,
        version_id: row.get(1)?,
        file_path: PathBuf::from(row.get::<_, String>(2)?),
        checksum_algorithm: row.get(3)?,
        checksum: row.get(4)?,
        mime_type: row.get(5)?,
    })
}
