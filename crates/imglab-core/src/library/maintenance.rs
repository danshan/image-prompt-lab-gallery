use super::{
    database_error, io_error,
    repair::{load_repair_versions, update_repaired_version},
    storage::{
        file_digest, image_dimensions, is_safe_relative_path, managed_original_path,
        managed_storage_size, normalized_extension,
    },
    LocalLibraryService, CURRENT_CHECKSUM_ALGORITHM,
};
use crate::{
    AssetVersionId, DomainResult, IntegrityIssue, IntegrityIssueKind, LibraryStatusView,
    RepairIssue, RepairLibraryRequest, RepairSummary,
};
use rusqlite::Row;
use std::{fs, path::PathBuf};

impl LocalLibraryService {
    pub(super) fn repair_managed_library(
        &self,
        request: RepairLibraryRequest,
    ) -> DomainResult<RepairSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let rows = load_repair_versions(&connection)?;
        let mut summary = RepairSummary {
            dry_run: request.dry_run,
            scanned_versions: rows.len(),
            files_moved: 0,
            paths_updated: 0,
            checksums_updated: 0,
            dimensions_updated: 0,
            issues: Vec::new(),
        };

        for row in rows {
            let extension = normalized_extension(&row.file_path);
            let canonical_path =
                managed_original_path(&row.version_id, &extension, &row.created_at);
            if !row.file_path.is_absolute() && !is_safe_relative_path(&row.file_path) {
                summary.issues.push(RepairIssue {
                    version_id: row.version_id,
                    path: row.file_path,
                    message: "recorded relative file path escapes the library root".to_string(),
                });
                continue;
            }
            let current_path = if row.file_path.is_absolute() {
                row.file_path.clone()
            } else {
                request.library_path.join(&row.file_path)
            };
            let canonical_absolute_path = request.library_path.join(&canonical_path);

            if current_path.is_absolute() && !current_path.starts_with(&request.library_path) {
                summary.issues.push(RepairIssue {
                    version_id: row.version_id,
                    path: row.file_path,
                    message: "recorded file path is outside the library root".to_string(),
                });
                continue;
            }

            let source_path = if current_path.is_file() {
                current_path
            } else if canonical_absolute_path.is_file() {
                canonical_absolute_path.clone()
            } else {
                summary.issues.push(RepairIssue {
                    version_id: row.version_id,
                    path: row.file_path,
                    message: "managed file is missing".to_string(),
                });
                continue;
            };

            let target_differs = row.file_path != canonical_path;
            let needs_move = target_differs && source_path != canonical_absolute_path;
            if needs_move && canonical_absolute_path.exists() {
                summary.issues.push(RepairIssue {
                    version_id: row.version_id,
                    path: canonical_path,
                    message: "canonical target path already exists".to_string(),
                });
                continue;
            }

            let checksum = file_digest(&source_path, CURRENT_CHECKSUM_ALGORITHM)?;
            let checksum_differs = row.checksum_algorithm != CURRENT_CHECKSUM_ALGORITHM
                || row.checksum.as_deref() != Some(checksum.as_str())
                || row.sha256 != checksum;
            let dimensions = image_dimensions(&source_path)?;
            let dimensions_differ = match dimensions {
                (Some(width), Some(height)) => {
                    row.width != Some(width) || row.height != Some(height)
                }
                _ => false,
            };

            if needs_move {
                summary.files_moved += 1;
            }
            if target_differs {
                summary.paths_updated += 1;
            }
            if checksum_differs {
                summary.checksums_updated += 1;
            }
            if dimensions_differ {
                summary.dimensions_updated += 1;
            }

            if request.dry_run
                || (!needs_move && !target_differs && !checksum_differs && !dimensions_differ)
            {
                continue;
            }

            if needs_move {
                if let Some(parent) = canonical_absolute_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
                }
                fs::rename(&source_path, &canonical_absolute_path)
                    .map_err(|error| io_error(&canonical_absolute_path, error))?;
            }

            let update_result = update_repaired_version(
                &connection,
                &row.version_id,
                &canonical_path,
                &checksum,
                dimensions,
            );

            if let Err(error) = update_result {
                if needs_move {
                    let _ = fs::rename(&canonical_absolute_path, &source_path);
                }
                return Err(error);
            }
        }

        Ok(summary)
    }

    pub(super) fn check_library_integrity(
        &self,
        root_path: &std::path::Path,
    ) -> DomainResult<Vec<IntegrityIssue>> {
        let connection = Self::open_library_database(root_path)?;
        let mut statement = connection
            .prepare(
                "SELECT id, file_path, checksum_algorithm, COALESCE(checksum, sha256) FROM asset_versions ORDER BY created_at",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], integrity_row_from_row)
            .map_err(database_error)?;

        let mut issues = Vec::new();
        for row in rows {
            let (version_id, relative_path, checksum_algorithm, expected_checksum) =
                row.map_err(database_error)?;
            let path = root_path.join(&relative_path);
            if !path.is_file() {
                issues.push(IntegrityIssue {
                    version_id: AssetVersionId(version_id),
                    path: relative_path,
                    kind: IntegrityIssueKind::MissingFile,
                    message: "managed original file is missing".to_string(),
                });
                continue;
            }

            let actual_checksum = file_digest(&path, &checksum_algorithm)?;
            if actual_checksum != expected_checksum {
                issues.push(IntegrityIssue {
                    version_id: AssetVersionId(version_id),
                    path: relative_path,
                    kind: IntegrityIssueKind::HashMismatch,
                    message: format!(
                        "expected {checksum_algorithm} {expected_checksum}, found {actual_checksum}"
                    ),
                });
            }
        }

        Ok(issues)
    }

    pub(super) fn library_maintenance_status(
        &self,
        root_path: &std::path::Path,
    ) -> DomainResult<LibraryStatusView> {
        Self::validate_layout(root_path)?;
        let issues = self.check_library_integrity(root_path)?;
        Ok(LibraryStatusView {
            storage_size_bytes: managed_storage_size(root_path)?,
            integrity_status: if issues.is_empty() {
                "healthy".to_string()
            } else {
                "issues_found".to_string()
            },
            integrity_issue_count: issues.len() as u32,
        })
    }
}

fn integrity_row_from_row(row: &Row<'_>) -> rusqlite::Result<(String, PathBuf, String, String)> {
    Ok((
        row.get::<_, String>(0)?,
        PathBuf::from(row.get::<_, String>(1)?),
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
    ))
}
