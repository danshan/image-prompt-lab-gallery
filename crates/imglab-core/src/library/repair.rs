use super::{database_error, CURRENT_CHECKSUM_ALGORITHM};
use crate::{AssetVersionId, DomainResult};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct RepairVersionRow {
    pub(super) version_id: AssetVersionId,
    pub(super) file_path: PathBuf,
    pub(super) sha256: String,
    pub(super) checksum_algorithm: String,
    pub(super) checksum: Option<String>,
    pub(super) width: Option<u32>,
    pub(super) height: Option<u32>,
    pub(super) created_at: String,
}

pub(super) fn load_repair_versions(connection: &Connection) -> DomainResult<Vec<RepairVersionRow>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, file_path, sha256, checksum_algorithm, checksum, width, height, created_at
            FROM asset_versions
            ORDER BY created_at
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(RepairVersionRow {
                version_id: AssetVersionId(row.get(0)?),
                file_path: PathBuf::from(row.get::<_, String>(1)?),
                sha256: row.get(2)?,
                checksum_algorithm: row.get(3)?,
                checksum: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

pub(super) fn update_repaired_version(
    connection: &Connection,
    version_id: &AssetVersionId,
    file_path: &Path,
    checksum: &str,
    dimensions: (Option<u32>, Option<u32>),
) -> DomainResult<()> {
    connection
        .execute(
            "
            UPDATE asset_versions
            SET file_path = ?1,
                sha256 = ?2,
                checksum_algorithm = ?3,
                checksum = ?2,
                width = COALESCE(?4, width),
                height = COALESCE(?5, height)
            WHERE id = ?6
            ",
            params![
                file_path.to_string_lossy(),
                checksum,
                CURRENT_CHECKSUM_ALGORITHM,
                dimensions.0,
                dimensions.1,
                version_id.0
            ],
        )
        .map_err(database_error)?;
    Ok(())
}
