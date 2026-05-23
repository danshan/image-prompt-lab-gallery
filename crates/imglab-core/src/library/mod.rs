use crate::{
    AssetId, CreateLibraryRequest, DiagnosticsOverviewView, DomainError, DomainResult,
    ExportLibraryBackupRequest, ExportLibraryRequest, ExportSummary, ImportLibraryBackupRequest,
    IntegrityIssue, LibraryBackupSummary, LibraryId, LibraryService, LibraryStatusView,
    LibrarySummary, RenameLibraryAliasRequest, RepairLibraryRequest, RepairSummary,
    StudioOverviewView,
};
#[cfg(test)]
use crate::{
    AssetVersionId, CreateChildVersionRequest, CreateGenerationEventRequest,
    CreateMetadataSuggestionRequest, GenerationOperation, IntegrityIssueKind,
};
#[cfg(test)]
use rusqlite::params;
#[cfg(test)]
use rusqlite::Connection;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

mod schema;
pub use schema::{migrate_library_database, CURRENT_SCHEMA_VERSION};
mod albums;
mod assets;
pub(crate) use assets::{
    import_asset_with_status, list_versions_for_asset, load_version,
    mark_imported_version_as_generated, persist_asset_version, promote_version_as_asset,
};
mod backup;
use assets::ensure_asset_exists;
#[cfg(test)]
use assets::load_asset_summary;
mod export;
use export::{load_export_versions, ExportVersionRow};
mod diagnostics;
mod generation;
pub(super) use crate::domain::generation::{operation_from_str, operation_to_str};
pub use generation::{normalize_provider_name, prepare_generation_request, LocalGenerationService};
mod gallery;
mod maintenance;
mod metadata;
use metadata::attach_tag;
mod registry;
mod repair;
mod service;
pub use crate::domain::library::LibraryManifest;
pub use service::LocalLibraryService;
mod storage;
pub(crate) use storage::{
    extension_for_mime_type, file_digest, image_dimensions, managed_original_path,
    mime_type_for_extension, normalized_extension, timestamp_string,
};
mod tasks;

const CHECKSUM_MD5: &str = "MD5";
const CHECKSUM_SHA256: &str = "SHA-256";
pub(crate) const CURRENT_CHECKSUM_ALGORITHM: &str = CHECKSUM_SHA256;

pub(super) const MANIFEST_FILE: &str = "manifest.json";
pub(super) const DATABASE_FILE: &str = "library.sqlite";
pub(super) const REQUIRED_DIRS: &[&str] = &[
    "originals",
    "derivatives",
    "derivatives/thumbnails",
    "derivatives/previews",
    "sidecars",
    "exports",
    "trash",
];

impl LocalLibraryService {
    pub fn add_tag_to_asset(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        tag: &str,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(library_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let now = timestamp_string();
        attach_tag(&connection, asset_id, tag, "manual", &now)?;
        Ok(())
    }
}

impl LibraryService for LocalLibraryService {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary> {
        self.create_managed_library(request)
    }

    fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
        self.open_managed_library(root_path)
    }

    fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>> {
        self.list_registry_libraries(include_hidden)
    }

    fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        self.hide_registry_library(library_id)
    }

    fn rename_library_alias(
        &self,
        request: RenameLibraryAliasRequest,
    ) -> DomainResult<LibrarySummary> {
        self.rename_registry_alias(request)
    }

    fn unregister_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        self.unregister_registry_library(library_id)
    }

    fn export_library(&self, request: ExportLibraryRequest) -> DomainResult<ExportSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        fs::create_dir_all(request.output_path.join("originals"))
            .map_err(|error| io_error(&request.output_path, error))?;
        fs::create_dir_all(request.output_path.join("sidecars"))
            .map_err(|error| io_error(&request.output_path, error))?;

        let versions = load_export_versions(&connection, request.album_id.as_ref())?;
        let mut sidecars: BTreeMap<String, Vec<ExportVersionRow>> = BTreeMap::new();
        let mut exported_files = 0;

        for version in versions {
            let source = request.library_path.join(&version.file_path);
            if !source.is_file() {
                return Err(DomainError::FileIntegrityMismatch {
                    version_id: version.version_id.clone(),
                    message: "source file is missing during export".to_string(),
                });
            }

            let extension = normalized_extension(&source);
            let destination = request
                .output_path
                .join("originals")
                .join(format!("{}.{}", version.version_id, extension));
            fs::copy(&source, &destination).map_err(|error| io_error(&destination, error))?;
            exported_files += 1;

            sidecars
                .entry(version.asset_id.clone())
                .or_default()
                .push(version);
        }

        let mut exported_sidecars = 0;
        for (asset_id, versions) in sidecars {
            let sidecar = json!({
                "asset_id": asset_id,
                "versions": versions.iter().map(|version| {
                    json!({
                        "id": version.version_id,
                        "file_path": version.file_path.to_string_lossy(),
                        "checksum_algorithm": version.checksum_algorithm,
                        "checksum": version.checksum,
                        "mime_type": version.mime_type,
                    })
                }).collect::<Vec<_>>()
            });
            let path = request
                .output_path
                .join("sidecars")
                .join(format!("{asset_id}.json"));
            let content = serde_json::to_string_pretty(&sidecar).map_err(serialization_error)?;
            fs::write(&path, content).map_err(|error| io_error(&path, error))?;
            exported_sidecars += 1;
        }

        Ok(ExportSummary {
            exported_files,
            exported_sidecars,
        })
    }

    fn export_library_backup_zip(&self, request: ExportLibraryBackupRequest) -> DomainResult<()> {
        backup::export_backup_zip(&request.library_path, &request.output_zip_path)
    }

    fn import_library_backup_zip(
        &self,
        request: ImportLibraryBackupRequest,
    ) -> DomainResult<LibraryBackupSummary> {
        let (manifest, cloned) = backup::import_backup_zip(
            &request.zip_path,
            &request.destination_path,
            |library_id| self.registry_contains_library_id(library_id),
        )?;
        let summary = Self::summary_from_manifest(&request.destination_path, &manifest, false);
        self.upsert_registry(&summary, &manifest.created_at, false)?;
        Ok(LibraryBackupSummary {
            library: summary,
            cloned,
        })
    }

    fn repair_library(&self, request: RepairLibraryRequest) -> DomainResult<RepairSummary> {
        self.repair_managed_library(request)
    }

    fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
        self.check_library_integrity(root_path)
    }

    fn library_status(&self, root_path: &Path) -> DomainResult<LibraryStatusView> {
        self.library_maintenance_status(root_path)
    }

    fn studio_overview(&self, root_path: &Path) -> DomainResult<StudioOverviewView> {
        self.studio_diagnostics_overview(root_path)
    }

    fn diagnostics_overview(&self, root_path: &Path) -> DomainResult<DiagnosticsOverviewView> {
        self.library_diagnostics_overview(root_path)
    }
}

fn io_error(path: &Path, error: std::io::Error) -> DomainError {
    DomainError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
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

#[cfg(test)]
mod tests;
