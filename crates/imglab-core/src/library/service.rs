use super::{
    database_error, io_error, migrate_library_database, serialization_error,
    storage::timestamp_string, DATABASE_FILE, MANIFEST_FILE, REQUIRED_DIRS,
};
use crate::{
    domain::library::{
        ensure_schema_supported, summary_from_manifest as domain_summary_from_manifest,
        LibraryManifest,
    },
    CreateLibraryRequest, DomainError, DomainResult, LibrarySummary, CURRENT_SCHEMA_VERSION,
};
use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LocalLibraryService {
    pub(super) registry_path: PathBuf,
}

impl LocalLibraryService {
    pub fn new(registry_path: impl Into<PathBuf>) -> Self {
        Self {
            registry_path: registry_path.into(),
        }
    }

    pub fn database_path(root_path: &Path) -> PathBuf {
        root_path.join(DATABASE_FILE)
    }

    pub fn manifest_path(root_path: &Path) -> PathBuf {
        root_path.join(MANIFEST_FILE)
    }

    pub(super) fn create_managed_library(
        &self,
        request: CreateLibraryRequest,
    ) -> DomainResult<LibrarySummary> {
        fs::create_dir_all(&request.root_path)
            .map_err(|error| io_error(&request.root_path, error))?;

        for relative in REQUIRED_DIRS {
            let path = request.root_path.join(relative);
            fs::create_dir_all(&path).map_err(|error| io_error(&path, error))?;
        }

        let created_at = timestamp_string();
        let manifest = LibraryManifest {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            schema_version: CURRENT_SCHEMA_VERSION,
            created_at: created_at.clone(),
            app: "image-prompt-lab".to_string(),
        };

        Self::write_manifest(&request.root_path, &manifest)?;
        Self::initialize_library_database(&request.root_path)?;

        let summary = Self::summary_from_manifest(&request.root_path, &manifest, false);
        self.upsert_registry(&summary, &created_at, false)?;

        Ok(summary)
    }

    pub(super) fn open_managed_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
        Self::validate_layout(root_path)?;
        let manifest = Self::read_manifest(root_path)?;

        ensure_schema_supported(manifest.schema_version, CURRENT_SCHEMA_VERSION)?;

        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)?;

        let summary = Self::summary_from_manifest(root_path, &manifest, false);
        self.upsert_registry(&summary, &manifest.created_at, false)?;

        Ok(summary)
    }

    pub(super) fn read_manifest(root_path: &Path) -> DomainResult<LibraryManifest> {
        let path = Self::manifest_path(root_path);
        let content = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        serde_json::from_str(&content).map_err(serialization_error)
    }

    pub(super) fn write_manifest(root_path: &Path, manifest: &LibraryManifest) -> DomainResult<()> {
        let path = Self::manifest_path(root_path);
        let content = serde_json::to_string_pretty(manifest).map_err(serialization_error)?;
        fs::write(&path, content).map_err(|error| io_error(&path, error))
    }

    fn initialize_library_database(root_path: &Path) -> DomainResult<()> {
        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)
    }

    pub(super) fn summary_from_manifest(
        root_path: &Path,
        manifest: &LibraryManifest,
        hidden: bool,
    ) -> LibrarySummary {
        domain_summary_from_manifest(root_path, manifest, hidden)
    }

    pub(super) fn validate_layout(root_path: &Path) -> DomainResult<()> {
        if !root_path.exists() {
            return Err(DomainError::LibraryNotFound {
                path: root_path.display().to_string(),
            });
        }

        for relative in REQUIRED_DIRS {
            let path = root_path.join(relative);
            if !path.is_dir() {
                return Err(DomainError::Io {
                    path: path.display().to_string(),
                    message: "required library directory is missing".to_string(),
                });
            }
        }

        let database_path = Self::database_path(root_path);
        if !database_path.is_file() {
            return Err(DomainError::Io {
                path: database_path.display().to_string(),
                message: "library database is missing".to_string(),
            });
        }

        Ok(())
    }

    pub(crate) fn open_library_database(root_path: &Path) -> DomainResult<Connection> {
        Self::validate_layout(root_path)?;
        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)?;
        Ok(connection)
    }
}
