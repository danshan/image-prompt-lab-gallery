use crate::{
    hash::{md5_reader, sha256_reader},
    AlbumId, AlbumKind, AlbumService, AlbumSummary, AssetId, AssetService, AssetSummary,
    AssetVersionId, CreateChildVersionRequest, CreateGenerationEventRequest, CreateLibraryRequest,
    DomainError, DomainResult, ExportLibraryRequest, ExportSummary, FileContextView,
    GalleryAssetView, GalleryQuery, GalleryReadService, GallerySort, GenerateImageRequest,
    GenerationEventId, GenerationEventSummary, GenerationOperation, ImageProvider,
    ImportAssetRequest, IntegrityIssue, IntegrityIssueKind, LibraryId, LibraryService,
    LibraryStatusView, LibrarySummary, LineageEntry, MetadataReviewService, MetadataSuggestion,
    MetadataSuggestionId, RepairIssue, RepairLibraryRequest, RepairSummary,
    ReviewMetadataSuggestionRequest, ReviewStatusFilter, SearchQuery, SearchService,
    UpdateAssetMetadataRequest, VersionSummary,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;
const CHECKSUM_MD5: &str = "MD5";
const CHECKSUM_SHA256: &str = "SHA-256";

const MANIFEST_FILE: &str = "manifest.json";
const DATABASE_FILE: &str = "library.sqlite";
const REQUIRED_DIRS: &[&str] = &[
    "originals",
    "derivatives",
    "derivatives/thumbnails",
    "derivatives/previews",
    "sidecars",
    "exports",
    "trash",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryManifest {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    pub created_at: String,
    pub app: String,
}

#[derive(Debug, Clone)]
pub struct LocalLibraryService {
    registry_path: PathBuf,
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

    fn ensure_registry(&self) -> DomainResult<Connection> {
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

    fn upsert_registry(
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

    fn read_manifest(root_path: &Path) -> DomainResult<LibraryManifest> {
        let path = Self::manifest_path(root_path);
        let content = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        serde_json::from_str(&content).map_err(serialization_error)
    }

    fn write_manifest(root_path: &Path, manifest: &LibraryManifest) -> DomainResult<()> {
        let path = Self::manifest_path(root_path);
        let content = serde_json::to_string_pretty(manifest).map_err(serialization_error)?;
        fs::write(&path, content).map_err(|error| io_error(&path, error))
    }

    fn initialize_library_database(root_path: &Path) -> DomainResult<()> {
        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)
    }

    fn summary_from_manifest(
        root_path: &Path,
        manifest: &LibraryManifest,
        hidden: bool,
    ) -> LibrarySummary {
        LibrarySummary {
            id: LibraryId(manifest.id.clone()),
            name: manifest.name.clone(),
            root_path: root_path.to_path_buf(),
            hidden,
            schema_version: manifest.schema_version,
        }
    }

    fn validate_layout(root_path: &Path) -> DomainResult<()> {
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

    fn open_library_database(root_path: &Path) -> DomainResult<Connection> {
        Self::validate_layout(root_path)?;
        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)?;
        Ok(connection)
    }

    pub fn add_tag_to_asset(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        tag: &str,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(library_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let now = timestamp_string();
        let tag_id = Uuid::new_v4().to_string();

        connection
            .execute(
                "
                INSERT INTO tags (id, name, color, created_at)
                VALUES (?1, ?2, NULL, ?3)
                ON CONFLICT(name) DO NOTHING
                ",
                params![tag_id, tag, now],
            )
            .map_err(database_error)?;
        let existing_tag_id: String = connection
            .query_row("SELECT id FROM tags WHERE name = ?1", params![tag], |row| {
                row.get(0)
            })
            .map_err(database_error)?;
        connection
            .execute(
                "
                INSERT INTO asset_tags (asset_id, tag_id, source, confirmed_at)
                VALUES (?1, ?2, 'manual', ?3)
                ON CONFLICT(asset_id, tag_id) DO UPDATE SET confirmed_at = excluded.confirmed_at
                ",
                params![asset_id.0, existing_tag_id, now],
            )
            .map_err(database_error)?;
        Ok(())
    }
}

pub struct LocalGenerationService<P> {
    provider: P,
}

impl<P> LocalGenerationService<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P> crate::GenerationService for LocalGenerationService<P>
where
    P: ImageProvider,
{
    fn generate(&self, request: GenerateImageRequest) -> DomainResult<Vec<VersionSummary>> {
        if !self
            .provider
            .supports_operation(request.parameters.operation)
        {
            return Err(DomainError::UnsupportedProviderCapability {
                provider: self.provider.name().to_string(),
                capability: operation_to_str(request.parameters.operation).to_string(),
            });
        }
        self.provider.validate_parameters(&request.parameters)?;
        let result = match request.parameters.operation {
            GenerationOperation::TextToImage => {
                self.provider.generate_from_text(&request.parameters)?
            }
            GenerationOperation::ImageToImage => {
                let input = request.input_bytes.as_deref().ok_or_else(|| {
                    DomainError::InvalidGenerationParameters {
                        message: "image-to-image generation requires input bytes".to_string(),
                    }
                })?;
                self.provider
                    .generate_from_image(&request.parameters, input)?
            }
        };

        let library_service =
            LocalLibraryService::new(std::env::temp_dir().join("imglab-unused-registry.sqlite"));
        let mut versions = Vec::new();
        let mut asset_id = None;
        let parent_version_id = request.parameters.input_version_id.clone();

        if let Some(parent_id) = &parent_version_id {
            let connection = LocalLibraryService::open_library_database(&request.library_path)?;
            let parent = load_version(&connection, parent_id)?;
            asset_id = Some(parent.asset_id);
        }

        for image in result.images {
            let temp_dir =
                std::env::temp_dir().join(format!("imglab-generated-{}", Uuid::new_v4()));
            fs::create_dir_all(&temp_dir).map_err(|error| io_error(&temp_dir, error))?;
            let extension = extension_for_mime_type(&image.mime_type);
            let temp_file = temp_dir.join(format!("output.{extension}"));
            fs::write(&temp_file, &image.bytes).map_err(|error| io_error(&temp_file, error))?;

            let version = if let (Some(asset_id), Some(parent_version_id)) =
                (asset_id.clone(), parent_version_id.clone())
            {
                let event =
                    library_service.record_generation_event(CreateGenerationEventRequest {
                        library_path: request.library_path.clone(),
                        asset_id: Some(asset_id.clone()),
                        output_version_id: None,
                        provider: request.parameters.provider.clone(),
                        provider_model: request.parameters.model.clone(),
                        operation_type: request.parameters.operation,
                        prompt: request.parameters.prompt.clone(),
                        negative_prompt: request.parameters.negative_prompt.clone(),
                        input_asset_version_id: Some(parent_version_id.clone()),
                        parameters_json: request.parameters.parameters_json.clone(),
                        raw_request_json: Some(result.raw_request_json.clone()),
                        raw_response_json: Some(result.raw_response_json.clone()),
                        status: "completed".to_string(),
                        error_code: None,
                        error_message: None,
                    })?;
                library_service.create_child_version(CreateChildVersionRequest {
                    library_path: request.library_path.clone(),
                    asset_id,
                    parent_version_id,
                    generation_event_id: Some(event.id),
                    source_path: temp_file,
                    mime_type: image.mime_type,
                    version_label: Some("generated".to_string()),
                })?
            } else {
                let (asset, mut version) = library_service.import_asset(ImportAssetRequest {
                    library_path: request.library_path.clone(),
                    source_path: temp_file,
                })?;
                let event =
                    library_service.record_generation_event(CreateGenerationEventRequest {
                        library_path: request.library_path.clone(),
                        asset_id: Some(asset.id.clone()),
                        output_version_id: Some(version.id.clone()),
                        provider: request.parameters.provider.clone(),
                        provider_model: request.parameters.model.clone(),
                        operation_type: request.parameters.operation,
                        prompt: request.parameters.prompt.clone(),
                        negative_prompt: request.parameters.negative_prompt.clone(),
                        input_asset_version_id: None,
                        parameters_json: request.parameters.parameters_json.clone(),
                        raw_request_json: Some(result.raw_request_json.clone()),
                        raw_response_json: Some(result.raw_response_json.clone()),
                        status: "completed".to_string(),
                        error_code: None,
                        error_message: None,
                    })?;
                mark_imported_version_as_generated(
                    &request.library_path,
                    &asset.id,
                    &version.id,
                    &event.id,
                )?;
                version.generation_event_id = Some(event.id);
                version
            };

            versions.push(version);
        }

        Ok(versions)
    }
}

impl LibraryService for LocalLibraryService {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary> {
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

    fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
        Self::validate_layout(root_path)?;
        let manifest = Self::read_manifest(root_path)?;

        if manifest.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(DomainError::SchemaMismatch {
                expected: CURRENT_SCHEMA_VERSION,
                found: manifest.schema_version,
            });
        }

        let database_path = Self::database_path(root_path);
        let connection = Connection::open(&database_path).map_err(database_error)?;
        migrate_library_database(&connection)?;

        let summary = Self::summary_from_manifest(root_path, &manifest, false);
        self.upsert_registry(&summary, &manifest.created_at, false)?;

        Ok(summary)
    }

    fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>> {
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
            .query_map(params![include_hidden], |row| {
                Ok(LibrarySummary {
                    id: LibraryId(row.get(0)?),
                    name: row.get(1)?,
                    root_path: PathBuf::from(row.get::<_, String>(2)?),
                    hidden: row.get::<_, i64>(3)? != 0,
                    schema_version: row.get::<_, u32>(4)?,
                })
            })
            .map_err(database_error)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()> {
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
                        "sha256": version.sha256,
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

    fn repair_library(&self, request: RepairLibraryRequest) -> DomainResult<RepairSummary> {
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

            let checksum = file_digest(&source_path, CHECKSUM_MD5)?;
            let checksum_differs = row.checksum_algorithm != CHECKSUM_MD5
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

    fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
        let connection = Self::open_library_database(root_path)?;
        let mut statement = connection
            .prepare(
                "SELECT id, file_path, checksum_algorithm, COALESCE(checksum, sha256) FROM asset_versions ORDER BY created_at",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    PathBuf::from(row.get::<_, String>(1)?),
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
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

    fn library_status(&self, root_path: &Path) -> DomainResult<LibraryStatusView> {
        Self::validate_layout(root_path)?;
        let issues = self.check_integrity(root_path)?;
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

impl AssetService for LocalLibraryService {
    fn import_asset(
        &self,
        request: ImportAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        let manifest = Self::read_manifest(&request.library_path)?;
        let connection = Self::open_library_database(&request.library_path)?;

        if !request.source_path.is_file() {
            return Err(DomainError::Io {
                path: request.source_path.display().to_string(),
                message: "source file does not exist".to_string(),
            });
        }

        let asset_id = AssetId(Uuid::new_v4().to_string());
        let version_id = AssetVersionId(Uuid::new_v4().to_string());
        let extension = normalized_extension(&request.source_path);
        let now = timestamp_string();
        let relative_path = managed_original_path(&version_id, &extension, &now);
        let destination_path = request.library_path.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let temporary_path = destination_path.with_extension(format!("{extension}.tmp"));
        fs::copy(&request.source_path, &temporary_path)
            .map_err(|error| io_error(&temporary_path, error))?;
        fs::rename(&temporary_path, &destination_path)
            .map_err(|error| io_error(&destination_path, error))?;

        let checksum = file_digest(&destination_path, CHECKSUM_MD5)?;
        let (width, height) = image_dimensions(&destination_path)?;
        let mime_type = mime_type_for_extension(&extension).to_string();

        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO assets (
                    id, library_id, media_type, title, description, category,
                    rating, status, created_at, updated_at, captured_at
                )
                VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL, ?4, ?5, ?5, NULL)
                ",
                params![asset_id.0, manifest.id, mime_type, "imported", now],
            )
            .map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO asset_versions (
                    id, asset_id, parent_version_id, generation_event_id,
                    file_path, sha256, checksum_algorithm, checksum, width, height, mime_type, version_label, created_at
                )
                VALUES (?1, ?2, NULL, NULL, ?3, ?4, ?5, ?4, ?6, ?7, ?8, ?9, ?10)
                ",
                params![
                    version_id.0,
                    asset_id.0,
                    relative_path.to_string_lossy(),
                    checksum,
                    CHECKSUM_MD5,
                    width,
                    height,
                    mime_type,
                    "import",
                    now
                ],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)?;

        Ok((
            AssetSummary {
                id: asset_id.clone(),
                title: None,
                category: None,
                rating: None,
                status: "imported".to_string(),
            },
            VersionSummary {
                id: version_id,
                asset_id,
                parent_version_id: None,
                generation_event_id: None,
                file_path: relative_path,
                sha256: checksum.clone(),
                checksum_algorithm: CHECKSUM_MD5.to_string(),
                checksum,
                mime_type,
            },
        ))
    }

    fn create_child_version(
        &self,
        request: CreateChildVersionRequest,
    ) -> DomainResult<VersionSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_asset_exists(&connection, &request.asset_id)?;
        load_version(&connection, &request.parent_version_id)?;

        if !request.source_path.is_file() {
            return Err(DomainError::Io {
                path: request.source_path.display().to_string(),
                message: "source file does not exist".to_string(),
            });
        }

        let version_id = AssetVersionId(Uuid::new_v4().to_string());
        let extension = normalized_extension(&request.source_path);
        let now = timestamp_string();
        let relative_path = managed_original_path(&version_id, &extension, &now);
        let destination_path = request.library_path.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let temporary_path = destination_path.with_extension(format!("{extension}.tmp"));
        fs::copy(&request.source_path, &temporary_path)
            .map_err(|error| io_error(&temporary_path, error))?;
        fs::rename(&temporary_path, &destination_path)
            .map_err(|error| io_error(&destination_path, error))?;

        let checksum = file_digest(&destination_path, CHECKSUM_MD5)?;
        let (width, height) = image_dimensions(&destination_path)?;

        connection
            .execute(
                "
                INSERT INTO asset_versions (
                    id, asset_id, parent_version_id, generation_event_id,
                    file_path, sha256, checksum_algorithm, checksum, width, height, mime_type, version_label, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?8, ?9, ?10, ?11, ?12)
                ",
                params![
                    version_id.0,
                    request.asset_id.0,
                    request.parent_version_id.0,
                    request.generation_event_id.as_ref().map(|id| id.0.as_str()),
                    relative_path.to_string_lossy(),
                    checksum,
                    CHECKSUM_MD5,
                    width,
                    height,
                    request.mime_type,
                    request.version_label.as_deref(),
                    now
                ],
            )
            .map_err(database_error)?;

        if let Some(event_id) = &request.generation_event_id {
            update_generation_event_output_version(&request.library_path, event_id, &version_id)?;
        }

        Ok(VersionSummary {
            id: version_id,
            asset_id: request.asset_id,
            parent_version_id: Some(request.parent_version_id),
            generation_event_id: request.generation_event_id,
            file_path: relative_path,
            sha256: checksum.clone(),
            checksum_algorithm: CHECKSUM_MD5.to_string(),
            checksum,
            mime_type: request.mime_type,
        })
    }

    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let event_id = GenerationEventId(Uuid::new_v4().to_string());
        let now = timestamp_string();
        let operation_type = operation_to_str(request.operation_type);

        connection
            .execute(
                "
                INSERT INTO generation_events (
                    id, asset_id, output_version_id, provider, provider_model, operation_type,
                    prompt, negative_prompt, input_asset_version_id, parameters_json,
                    raw_request_json, raw_response_json, status, started_at, completed_at,
                    error_code, error_message
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, ?15, ?16)
                ",
                params![
                    event_id.0,
                    request.asset_id.as_ref().map(|id| id.0.as_str()),
                    request.output_version_id.as_ref().map(|id| id.0.as_str()),
                    request.provider,
                    request.provider_model,
                    operation_type,
                    request.prompt,
                    request.negative_prompt,
                    request
                        .input_asset_version_id
                        .as_ref()
                        .map(|id| id.0.as_str()),
                    request.parameters_json,
                    request.raw_request_json,
                    request.raw_response_json,
                    request.status,
                    now,
                    request.error_code,
                    request.error_message
                ],
            )
            .map_err(database_error)?;

        Ok(GenerationEventSummary {
            id: event_id,
            asset_id: request.asset_id,
            output_version_id: request.output_version_id,
            provider: request.provider,
            provider_model: request.provider_model,
            operation_type: request.operation_type,
            prompt: request.prompt,
            parameters_json: request.parameters_json,
            status: request.status,
        })
    }

    fn get_lineage(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<Vec<LineageEntry>> {
        let connection = Self::open_library_database(library_path)?;
        let mut lineage = Vec::new();
        let mut current = Some(version_id.clone());

        while let Some(current_id) = current {
            let version = load_version(&connection, &current_id)?;
            let event = match &version.generation_event_id {
                Some(event_id) => Some(load_generation_event(&connection, event_id)?),
                None => None,
            };
            current = version.parent_version_id.clone();
            lineage.push(LineageEntry {
                version,
                generation_event: event,
            });
        }

        Ok(lineage)
    }
}

impl MetadataReviewService for LocalLibraryService {
    fn create_suggestion(
        &self,
        request: crate::CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_asset_exists(&connection, &request.asset_id)?;

        let suggestion_id = MetadataSuggestionId(Uuid::new_v4().to_string());
        let tags_json =
            serde_json::to_string(&request.suggested_tags).map_err(serialization_error)?;
        let now = timestamp_string();

        connection
            .execute(
                "
                INSERT INTO metadata_suggestions (
                    id, asset_id, source, suggested_title, suggested_description,
                    suggested_tags_json, suggested_category, confidence_json,
                    status, created_at, reviewed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending_review', ?9, NULL)
                ",
                params![
                    suggestion_id.0,
                    request.asset_id.0,
                    request.source,
                    request.suggested_title,
                    request.suggested_description,
                    tags_json,
                    request.suggested_category,
                    request.confidence_json,
                    now
                ],
            )
            .map_err(database_error)?;

        Ok(MetadataSuggestion {
            id: suggestion_id,
            asset_id: request.asset_id,
            suggested_title: request.suggested_title,
            suggested_description: request.suggested_description,
            suggested_tags: request.suggested_tags,
            suggested_category: request.suggested_category,
            confidence_json: request.confidence_json,
            status: "pending_review".to_string(),
        })
    }

    fn list_pending(
        &self,
        library_path: &Path,
        library_id: &LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        let connection = Self::open_library_database(library_path)?;
        let mut statement = connection
            .prepare(
                "
                SELECT ms.id, ms.asset_id, ms.suggested_title, ms.suggested_description,
                       ms.suggested_tags_json, ms.suggested_category, ms.confidence_json, ms.status
                FROM metadata_suggestions ms
                INNER JOIN assets a ON a.id = ms.asset_id
                WHERE a.library_id = ?1 AND ms.status = 'pending_review'
                ORDER BY ms.created_at
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![library_id.0], metadata_suggestion_from_row)
            .map_err(database_error)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let suggestion = load_suggestion(&connection, &request.suggestion_id)?;
        let now = timestamp_string();
        let transaction = connection.unchecked_transaction().map_err(database_error)?;

        transaction
            .execute(
                "
                UPDATE assets
                SET title = ?1, description = ?2, category = ?3, updated_at = ?4
                WHERE id = ?5
                ",
                params![
                    request.title,
                    request.description,
                    request.category,
                    now,
                    suggestion.asset_id.0
                ],
            )
            .map_err(database_error)?;

        for tag in request.tags {
            let tag_id = Uuid::new_v4().to_string();
            transaction
                .execute(
                    "
                    INSERT INTO tags (id, name, color, created_at)
                    VALUES (?1, ?2, NULL, ?3)
                    ON CONFLICT(name) DO NOTHING
                    ",
                    params![tag_id, tag, now],
                )
                .map_err(database_error)?;
            let existing_tag_id: String = transaction
                .query_row("SELECT id FROM tags WHERE name = ?1", params![tag], |row| {
                    row.get(0)
                })
                .map_err(database_error)?;
            transaction
                .execute(
                    "
                    INSERT INTO asset_tags (asset_id, tag_id, source, confirmed_at)
                    VALUES (?1, ?2, 'metadata_review', ?3)
                    ON CONFLICT(asset_id, tag_id) DO UPDATE SET confirmed_at = excluded.confirmed_at
                    ",
                    params![suggestion.asset_id.0, existing_tag_id, now],
                )
                .map_err(database_error)?;
        }

        transaction
            .execute(
                "UPDATE metadata_suggestions SET status = 'accepted', reviewed_at = ?1 WHERE id = ?2",
                params![now, request.suggestion_id.0],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)?;

        load_asset_summary(&connection, &suggestion.asset_id)
    }

    fn reject(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(library_path)?;
        let updated = connection
            .execute(
                "UPDATE metadata_suggestions SET status = 'rejected', reviewed_at = ?1 WHERE id = ?2",
                params![timestamp_string(), suggestion_id.0],
            )
            .map_err(database_error)?;

        if updated == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: suggestion_id.0.clone(),
            });
        }

        Ok(())
    }
}

impl AlbumService for LocalLibraryService {
    fn create_manual_album(
        &self,
        library_id: &LibraryId,
        name: &str,
    ) -> DomainResult<AlbumSummary> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        create_album(&connection, name, AlbumKind::Manual, None)
    }

    fn create_smart_album(
        &self,
        request: crate::CreateSmartAlbumRequest,
    ) -> DomainResult<AlbumSummary> {
        validate_smart_query(&request.smart_query_json)?;
        let connection = Self::open_library_database(&request.library_path)?;
        create_album(
            &connection,
            &request.name,
            AlbumKind::Smart,
            Some(request.smart_query_json),
        )
    }

    fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| {
                Self::open_library_database(&library.root_path)
                    .and_then(|connection| {
                        let count: i64 = connection
                            .query_row(
                                "SELECT COUNT(*) FROM albums WHERE id = ?1",
                                params![album_id.0],
                                |row| row.get(0),
                            )
                            .map_err(database_error)?;
                        Ok(count > 0)
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| DomainError::InvalidAssetReference {
                id: album_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let sort_order: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM album_items WHERE album_id = ?1",
                params![album_id.0],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        connection
            .execute(
                "
                INSERT INTO album_items (album_id, asset_id, sort_order, added_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(album_id, asset_id) DO NOTHING
                ",
                params![album_id.0, asset_id.0, sort_order, timestamp_string()],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn update_asset_metadata(
        &self,
        request: UpdateAssetMetadataRequest,
    ) -> DomainResult<AssetSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_asset_exists(&connection, &request.asset_id)?;
        if let Some(rating) = request.rating {
            if !(1..=5).contains(&rating) {
                return Err(DomainError::InvalidGenerationParameters {
                    message: "rating must be between 1 and 5".to_string(),
                });
            }
        }

        connection
            .execute(
                "
                UPDATE assets
                SET title = COALESCE(?1, title),
                    description = COALESCE(?2, description),
                    rating = COALESCE(?3, rating),
                    category = COALESCE(?4, category),
                    status = COALESCE(?5, status),
                    updated_at = ?6
                WHERE id = ?7
                ",
                params![
                    request.title,
                    request.description,
                    request.rating,
                    request.category,
                    request.status,
                    timestamp_string(),
                    request.asset_id.0
                ],
            )
            .map_err(database_error)?;
        load_asset_summary(&connection, &request.asset_id)
    }
}

impl SearchService for LocalLibraryService {
    fn search(
        &self,
        library_id: &LibraryId,
        query: SearchQuery,
    ) -> DomainResult<Vec<AssetSummary>> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        search_assets(&connection, query)
    }
}

impl GalleryReadService for LocalLibraryService {
    fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>> {
        validate_gallery_query(&query)?;
        let connection = Self::open_library_database(library_path)?;
        let mut items = load_gallery_asset_views(&connection)?;

        if let Some(text) = query
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            let needle = text.to_ascii_lowercase();
            items.retain(|item| {
                item.title
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
                    || item
                        .category
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .provider
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .model_label
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .prompt
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&needle))
            });
        }

        if !query.providers.is_empty() {
            items.retain(|item| {
                item.provider
                    .as_ref()
                    .map(|provider| query.providers.iter().any(|wanted| wanted == provider))
                    .unwrap_or(false)
            });
        }

        if let Some(min_rating) = query.min_rating {
            items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
        }

        if query.review_status == ReviewStatusFilter::Pending {
            items.retain(|item| item.review_pending_count > 0);
        }

        if !query.tags.is_empty() {
            items.retain(|item| query.tags.iter().all(|tag| item.tags.contains(tag)));
        }

        if let Some(album_id) = &query.album_id {
            items.retain(|item| asset_in_album(&connection, album_id, &item.id).unwrap_or(false));
        }

        match query.sort {
            GallerySort::Newest => items.sort_by(|left, right| {
                right
                    .updated_at
                    .cmp(&left.updated_at)
                    .then_with(|| right.created_at.cmp(&left.created_at))
            }),
            GallerySort::Oldest => items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.updated_at.cmp(&right.updated_at))
            }),
            GallerySort::RatingDesc => items.sort_by(|left, right| {
                right
                    .rating
                    .unwrap_or_default()
                    .cmp(&left.rating.unwrap_or_default())
                    .then_with(|| left.title.cmp(&right.title))
            }),
            GallerySort::TitleAsc => items.sort_by(|left, right| left.title.cmp(&right.title)),
            GallerySort::ProviderAsc => {
                items.sort_by(|left, right| left.provider.cmp(&right.provider))
            }
        }

        Ok(items)
    }

    fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<crate::AssetDetailView> {
        let connection = Self::open_library_database(library_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let asset = load_asset_detail_base(&connection, asset_id)?;
        let versions = load_asset_versions(&connection, asset_id)?;
        let current_version = current_version_id
            .map(|version_id| load_version(&connection, version_id))
            .transpose()?
            .or_else(|| versions.first().cloned());
        let event = current_version
            .as_ref()
            .and_then(|version| version.generation_event_id.as_ref())
            .map(|event_id| load_generation_event_detail(&connection, event_id))
            .transpose()?
            .or_else(|| {
                load_latest_generation_event_detail(&connection, asset_id)
                    .ok()
                    .flatten()
            });
        let lineage = current_version
            .as_ref()
            .map(|version| self.get_lineage(library_path, &version.id))
            .transpose()?
            .unwrap_or_default();
        let file = current_version
            .as_ref()
            .map(|version| load_file_context(library_path, &connection, version))
            .transpose()?;

        Ok(crate::AssetDetailView {
            id: asset.id,
            title: asset.title,
            description: asset.description,
            category: asset.category,
            rating: asset.rating,
            status: asset.status,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            negative_prompt: event
                .as_ref()
                .and_then(|event| event.negative_prompt.clone()),
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.provider_model.clone()),
            parameters_json: event.as_ref().map(|event| event.parameters_json.clone()),
            tags: load_asset_tags(&connection, asset_id)?,
            albums: load_asset_albums(&connection, asset_id)?,
            review_pending_count: pending_review_count(&connection, asset_id)?,
            versions,
            lineage,
            file,
        })
    }
}

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

fn timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

fn managed_original_path(version_id: &AssetVersionId, extension: &str, timestamp: &str) -> PathBuf {
    let millis = timestamp.parse::<u128>().unwrap_or_default();
    let days = (millis / 86_400_000) as i64;
    let (year, month, _) = civil_from_days(days);
    PathBuf::from("originals")
        .join(format!("{year:04}"))
        .join(format!("{month:02}"))
        .join(format!("{}.{}", version_id.0, extension))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

fn file_digest(path: &Path, algorithm: &str) -> DomainResult<String> {
    let file = File::open(path).map_err(|error| io_error(path, error))?;
    match algorithm {
        CHECKSUM_MD5 => md5_reader(file).map_err(|error| io_error(path, error)),
        CHECKSUM_SHA256 => sha256_reader(file).map_err(|error| io_error(path, error)),
        other => Err(DomainError::Database {
            message: format!("unsupported checksum algorithm: {other}"),
        }),
    }
}

fn image_dimensions(path: &Path) -> DomainResult<(Option<u32>, Option<u32>)> {
    let bytes = fs::read(path).map_err(|error| io_error(path, error))?;
    Ok(parse_image_dimensions(&bytes).unwrap_or((None, None)))
}

fn parse_image_dimensions(bytes: &[u8]) -> Option<(Option<u32>, Option<u32>)> {
    parse_png_dimensions(bytes)
        .or_else(|| parse_jpeg_dimensions(bytes))
        .or_else(|| parse_webp_dimensions(bytes))
        .map(|(width, height)| (Some(width), Some(height)))
}

fn parse_png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[0..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    non_zero_dimensions(width, height)
}

fn parse_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut index = 2usize;
    while index + 3 < bytes.len() {
        while index < bytes.len() && bytes[index] != 0xff {
            index += 1;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        let marker = bytes[index];
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if index + 2 > bytes.len() {
            return None;
        }
        let length = u16::from_be_bytes(bytes[index..index + 2].try_into().ok()?) as usize;
        if length < 2 || index + length > bytes.len() {
            return None;
        }
        let payload = index + 2;
        if is_jpeg_sof_marker(marker) {
            if length < 7 {
                return None;
            }
            let height =
                u16::from_be_bytes(bytes[payload + 1..payload + 3].try_into().ok()?) as u32;
            let width = u16::from_be_bytes(bytes[payload + 3..payload + 5].try_into().ok()?) as u32;
            return non_zero_dimensions(width, height);
        }
        index += length;
    }
    None
}

fn is_jpeg_sof_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn parse_webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    match &bytes[12..16] {
        b"VP8X" if bytes.len() >= 30 => {
            let width = 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]);
            let height = 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]);
            non_zero_dimensions(width, height)
        }
        b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => {
            let b1 = bytes[21] as u32;
            let b2 = bytes[22] as u32;
            let b3 = bytes[23] as u32;
            let b4 = bytes[24] as u32;
            let width = 1 + (((b2 & 0x3f) << 8) | b1);
            let height = 1 + (((b4 & 0x0f) << 10) | (b3 << 2) | ((b2 & 0xc0) >> 6));
            non_zero_dimensions(width, height)
        }
        b"VP8 " if bytes.len() >= 30 && bytes[23..26] == [0x9d, 0x01, 0x2a] => {
            let width = (u16::from_le_bytes(bytes[26..28].try_into().ok()?) & 0x3fff) as u32;
            let height = (u16::from_le_bytes(bytes[28..30].try_into().ok()?) & 0x3fff) as u32;
            non_zero_dimensions(width, height)
        }
        _ => None,
    }
}

fn non_zero_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

fn is_safe_relative_path(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn managed_storage_size(root_path: &Path) -> DomainResult<u64> {
    REQUIRED_DIRS
        .iter()
        .filter(|relative| !relative.contains('/'))
        .try_fold(0u64, |total, relative| {
            directory_size(&root_path.join(relative)).map(|size| total + size)
        })
}

fn directory_size(path: &Path) -> DomainResult<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0u64;
    for entry in fs::read_dir(path).map_err(|error| io_error(path, error))? {
        let entry = entry.map_err(|error| io_error(path, error))?;
        let metadata = entry
            .metadata()
            .map_err(|error| io_error(&entry.path(), error))?;
        if metadata.is_dir() {
            total += directory_size(&entry.path())?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
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

fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| "bin".to_string())
}

fn mime_type_for_extension(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

fn extension_for_mime_type(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/avif" => "avif",
        _ => "png",
    }
}

fn operation_to_str(operation: GenerationOperation) -> &'static str {
    match operation {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
}

fn operation_from_str(value: &str) -> DomainResult<GenerationOperation> {
    match value {
        "text_to_image" => Ok(GenerationOperation::TextToImage),
        "image_to_image" => Ok(GenerationOperation::ImageToImage),
        _ => Err(DomainError::Database {
            message: format!("unknown generation operation: {value}"),
        }),
    }
}

fn ensure_asset_exists(connection: &Connection, asset_id: &AssetId) -> DomainResult<()> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE id = ?1",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if count == 0 {
        return Err(DomainError::InvalidAssetReference {
            id: asset_id.0.clone(),
        });
    }
    Ok(())
}

fn load_version(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<VersionSummary> {
    connection
        .query_row(
            "
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path,
                   sha256, checksum_algorithm, COALESCE(checksum, sha256), mime_type
            FROM asset_versions
            WHERE id = ?1
            ",
            params![version_id.0],
            |row| {
                Ok(VersionSummary {
                    id: AssetVersionId(row.get(0)?),
                    asset_id: AssetId(row.get(1)?),
                    parent_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
                    generation_event_id: row.get::<_, Option<String>>(3)?.map(GenerationEventId),
                    file_path: PathBuf::from(row.get::<_, String>(4)?),
                    sha256: row.get(5)?,
                    checksum_algorithm: row.get(6)?,
                    checksum: row.get(7)?,
                    mime_type: row.get(8)?,
                })
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidAssetReference {
                id: version_id.0.clone(),
            },
            other => database_error(other),
        })
}

fn load_generation_event(
    connection: &Connection,
    event_id: &GenerationEventId,
) -> DomainResult<GenerationEventSummary> {
    connection
        .query_row(
            "
            SELECT id, asset_id, output_version_id, provider, provider_model, operation_type,
                   prompt, parameters_json, status
            FROM generation_events
            WHERE id = ?1
            ",
            params![event_id.0],
            |row| {
                let operation_value: String = row.get(5)?;
                let operation_type = operation_from_str(&operation_value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(GenerationEventSummary {
                    id: GenerationEventId(row.get(0)?),
                    asset_id: row.get::<_, Option<String>>(1)?.map(AssetId),
                    output_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
                    provider: row.get(3)?,
                    provider_model: row.get(4)?,
                    operation_type,
                    prompt: row.get(6)?,
                    parameters_json: row.get(7)?,
                    status: row.get(8)?,
                })
            },
        )
        .map_err(database_error)
}

fn metadata_suggestion_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetadataSuggestion> {
    let tags_json: String = row.get(4)?;
    let suggested_tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(MetadataSuggestion {
        id: MetadataSuggestionId(row.get(0)?),
        asset_id: AssetId(row.get(1)?),
        suggested_title: row.get(2)?,
        suggested_description: row.get(3)?,
        suggested_tags,
        suggested_category: row.get(5)?,
        confidence_json: row.get(6)?,
        status: row.get(7)?,
    })
}

fn load_suggestion(
    connection: &Connection,
    suggestion_id: &MetadataSuggestionId,
) -> DomainResult<MetadataSuggestion> {
    connection
        .query_row(
            "
            SELECT id, asset_id, suggested_title, suggested_description,
                   suggested_tags_json, suggested_category, confidence_json, status
            FROM metadata_suggestions
            WHERE id = ?1
            ",
            params![suggestion_id.0],
            metadata_suggestion_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidAssetReference {
                id: suggestion_id.0.clone(),
            },
            other => database_error(other),
        })
}

fn load_asset_summary(connection: &Connection, asset_id: &AssetId) -> DomainResult<AssetSummary> {
    connection
        .query_row(
            "SELECT id, title, category, rating, status FROM assets WHERE id = ?1",
            params![asset_id.0],
            |row| {
                Ok(AssetSummary {
                    id: AssetId(row.get(0)?),
                    title: row.get(1)?,
                    category: row.get(2)?,
                    rating: row.get::<_, Option<u8>>(3)?,
                    status: row.get(4)?,
                })
            },
        )
        .map_err(database_error)
}

fn create_album(
    connection: &Connection,
    name: &str,
    kind: AlbumKind,
    smart_query_json: Option<String>,
) -> DomainResult<AlbumSummary> {
    let album_id = AlbumId(Uuid::new_v4().to_string());
    let now = timestamp_string();
    let kind_str = match kind {
        AlbumKind::Manual => "manual",
        AlbumKind::Smart => "smart",
    };
    connection
        .execute(
            "
            INSERT INTO albums (id, name, description, kind, smart_query_json, created_at, updated_at)
            VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?5)
            ",
            params![album_id.0, name, kind_str, smart_query_json, now],
        )
        .map_err(database_error)?;
    Ok(AlbumSummary {
        id: album_id,
        name: name.to_string(),
        kind,
    })
}

fn validate_smart_query(query_json: &str) -> DomainResult<()> {
    let value: serde_json::Value = serde_json::from_str(query_json).map_err(serialization_error)?;
    let object = value
        .as_object()
        .ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: "smart query must be a JSON object".to_string(),
        })?;
    const ALLOWED: &[&str] = &[
        "tags", "rating", "provider", "date", "status", "category", "text",
    ];
    for key in object.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            return Err(DomainError::InvalidSmartAlbumQuery {
                message: format!("unsupported smart query field: {key}"),
            });
        }
    }
    Ok(())
}

fn search_assets(connection: &Connection, query: SearchQuery) -> DomainResult<Vec<AssetSummary>> {
    let mut assets = load_all_asset_summaries(connection)?;
    if let Some(text) = query.text {
        let needle = text.to_ascii_lowercase();
        assets.retain(|asset| asset_matches_search_text(connection, &asset.id, asset, &needle));
    }
    if let Some(provider) = query.provider {
        assets.retain(|asset| {
            load_latest_generation_event(connection, &asset.id)
                .ok()
                .flatten()
                .map(|event| event.provider == provider)
                .unwrap_or(false)
        });
    }
    if let Some(min_rating) = query.min_rating {
        assets.retain(|asset| asset.rating.unwrap_or_default() >= min_rating);
    }
    if let Some(status) = query.status {
        assets.retain(|asset| asset.status == status);
    }
    if let Some(category) = query.category {
        assets.retain(|asset| asset.category.as_deref() == Some(category.as_str()));
    }
    if !query.tags.is_empty() {
        let wanted = query.tags;
        assets.retain(|asset| asset_has_all_tags(connection, &asset.id, &wanted).unwrap_or(false));
    }
    Ok(assets)
}

fn asset_matches_search_text(
    connection: &Connection,
    asset_id: &AssetId,
    asset: &AssetSummary,
    needle: &str,
) -> bool {
    if [
        asset.title.as_deref(),
        asset.category.as_deref(),
        Some(asset.status.as_str()),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_ascii_lowercase().contains(needle))
    {
        return true;
    }

    if load_asset_tags(connection, asset_id)
        .map(|tags| {
            tags.iter()
                .any(|tag| tag.to_ascii_lowercase().contains(needle))
        })
        .unwrap_or(false)
    {
        return true;
    }

    load_latest_generation_event(connection, asset_id)
        .ok()
        .flatten()
        .map(|event| {
            [
                event.provider.as_str(),
                event.provider_model.as_str(),
                event.prompt.as_str(),
            ]
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains(needle))
        })
        .unwrap_or(false)
}

fn default_title_from_prompt(prompt: &str) -> Option<String> {
    let stop_words = [
        "a", "an", "and", "as", "at", "by", "for", "from", "in", "into", "of", "on", "or", "the",
        "to", "with",
    ];
    let words = prompt
        .split(|character: char| !character.is_alphanumeric())
        .filter_map(|word| {
            let word = word.trim();
            if word.is_empty() {
                return None;
            }
            let lower = word.to_ascii_lowercase();
            if stop_words.contains(&lower.as_str()) {
                return None;
            }
            Some(title_case_word(&lower))
        })
        .take(6)
        .collect::<Vec<_>>();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

fn title_case_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn validate_gallery_query(query: &GalleryQuery) -> DomainResult<()> {
    if let Some(min_rating) = query.min_rating {
        if !(1..=5).contains(&min_rating) {
            return Err(DomainError::InvalidGalleryQuery {
                message: "min_rating must be between 1 and 5".to_string(),
            });
        }
    }
    Ok(())
}

fn update_generation_event_output_version(
    library_path: &Path,
    event_id: &GenerationEventId,
    output_version_id: &AssetVersionId,
) -> DomainResult<()> {
    let connection = LocalLibraryService::open_library_database(library_path)?;
    connection
        .execute(
            "
            UPDATE generation_events
            SET output_version_id = ?1
            WHERE id = ?2
            ",
            params![output_version_id.0, event_id.0],
        )
        .map_err(database_error)?;
    Ok(())
}

fn mark_imported_version_as_generated(
    library_path: &Path,
    asset_id: &AssetId,
    version_id: &AssetVersionId,
    event_id: &GenerationEventId,
) -> DomainResult<()> {
    let connection = LocalLibraryService::open_library_database(library_path)?;
    let now = timestamp_string();
    let title = load_generation_event_detail(&connection, event_id)
        .ok()
        .and_then(|event| default_title_from_prompt(&event.prompt));
    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    transaction
        .execute(
            "
            UPDATE asset_versions
            SET generation_event_id = ?1,
                version_label = 'generated'
            WHERE id = ?2
            ",
            params![event_id.0, version_id.0],
        )
        .map_err(database_error)?;
    transaction
        .execute(
            "
            UPDATE assets
            SET title = COALESCE(title, ?1),
                status = 'generated',
                updated_at = ?2
            WHERE id = ?3
            ",
            params![title, now, asset_id.0],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)?;
    Ok(())
}

fn load_gallery_asset_views(connection: &Connection) -> DomainResult<Vec<GalleryAssetView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, title, category, rating, status, created_at, updated_at
            FROM assets
            ORDER BY updated_at DESC, created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                AssetId(row.get::<_, String>(0)?),
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(database_error)?;

    let mut items = Vec::new();
    for row in rows {
        let (id, title, category, rating, status, created_at, updated_at) =
            row.map_err(database_error)?;
        let current_version = load_latest_asset_version(connection, &id)?;
        let event = current_version
            .as_ref()
            .and_then(|version| version.generation_event_id.as_ref())
            .map(|event_id| load_generation_event(connection, event_id))
            .transpose()?
            .or_else(|| load_latest_generation_event(connection, &id).ok().flatten());
        let version_count = count_asset_versions(connection, &id)?;
        let version_label = current_version
            .as_ref()
            .and_then(|version| load_version_label(connection, &version.id).ok().flatten());
        let (width, height) = current_version
            .as_ref()
            .map(|version| load_version_dimensions(connection, &version.id))
            .transpose()?
            .unwrap_or((None, None));

        items.push(GalleryAssetView {
            id: id.clone(),
            title,
            category,
            rating,
            status,
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.provider_model.clone()),
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            tags: load_asset_tags(connection, &id)?,
            review_pending_count: pending_review_count(connection, &id)?,
            current_version_id: current_version.as_ref().map(|version| version.id.clone()),
            image_path: current_version
                .as_ref()
                .map(|version| version.file_path.clone()),
            width,
            height,
            version_label,
            version_count,
            created_at,
            updated_at,
        });
    }

    Ok(items)
}

#[derive(Debug, Clone)]
struct AssetDetailBase {
    id: AssetId,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct GenerationEventDetail {
    provider: String,
    provider_model: String,
    prompt: String,
    negative_prompt: Option<String>,
    parameters_json: String,
}

fn load_asset_detail_base(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<AssetDetailBase> {
    connection
        .query_row(
            "
            SELECT id, title, description, category, rating, status, created_at, updated_at
            FROM assets
            WHERE id = ?1
            ",
            params![asset_id.0],
            |row| {
                Ok(AssetDetailBase {
                    id: AssetId(row.get(0)?),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    rating: row.get::<_, Option<u8>>(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(database_error)
}

fn load_generation_event_detail(
    connection: &Connection,
    event_id: &GenerationEventId,
) -> DomainResult<GenerationEventDetail> {
    connection
        .query_row(
            "
            SELECT provider, provider_model, prompt, negative_prompt, parameters_json
            FROM generation_events
            WHERE id = ?1
            ",
            params![event_id.0],
            |row| {
                Ok(GenerationEventDetail {
                    provider: row.get(0)?,
                    provider_model: row.get(1)?,
                    prompt: row.get(2)?,
                    negative_prompt: row.get(3)?,
                    parameters_json: row.get(4)?,
                })
            },
        )
        .map_err(database_error)
}

fn load_latest_generation_event(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<GenerationEventSummary>> {
    let event_id = connection
        .query_row(
            "
            SELECT id
            FROM generation_events
            WHERE asset_id = ?1
            ORDER BY started_at DESC
            LIMIT 1
            ",
            params![asset_id.0],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })?;
    event_id
        .map(|id| load_generation_event(connection, &GenerationEventId(id)))
        .transpose()
}

fn load_latest_generation_event_detail(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<GenerationEventDetail>> {
    let event_id = connection
        .query_row(
            "
            SELECT id
            FROM generation_events
            WHERE asset_id = ?1
            ORDER BY started_at DESC
            LIMIT 1
            ",
            params![asset_id.0],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })?;
    event_id
        .map(|id| load_generation_event_detail(connection, &GenerationEventId(id)))
        .transpose()
}

fn load_asset_versions(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<VersionSummary>> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path,
                   sha256, checksum_algorithm, COALESCE(checksum, sha256), mime_type
            FROM asset_versions
            WHERE asset_id = ?1
            ORDER BY created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| {
            Ok(VersionSummary {
                id: AssetVersionId(row.get(0)?),
                asset_id: AssetId(row.get(1)?),
                parent_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
                generation_event_id: row.get::<_, Option<String>>(3)?.map(GenerationEventId),
                file_path: PathBuf::from(row.get::<_, String>(4)?),
                sha256: row.get(5)?,
                checksum_algorithm: row.get(6)?,
                checksum: row.get(7)?,
                mime_type: row.get(8)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_latest_asset_version(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Option<VersionSummary>> {
    let version_id = connection
        .query_row(
            "
            SELECT id
            FROM asset_versions
            WHERE asset_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            ",
            params![asset_id.0],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })?;
    version_id
        .map(|id| load_version(connection, &AssetVersionId(id)))
        .transpose()
}

fn count_asset_versions(connection: &Connection, asset_id: &AssetId) -> DomainResult<u32> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM asset_versions WHERE asset_id = ?1",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count.max(0) as u32)
}

fn load_version_label(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<Option<String>> {
    connection
        .query_row(
            "SELECT version_label FROM asset_versions WHERE id = ?1",
            params![version_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)
}

fn load_version_dimensions(
    connection: &Connection,
    version_id: &AssetVersionId,
) -> DomainResult<(Option<u32>, Option<u32>)> {
    connection
        .query_row(
            "SELECT width, height FROM asset_versions WHERE id = ?1",
            params![version_id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(database_error)
}

fn load_asset_tags(connection: &Connection, asset_id: &AssetId) -> DomainResult<Vec<String>> {
    let mut statement = connection
        .prepare(
            "
            SELECT t.name
            FROM asset_tags at
            INNER JOIN tags t ON t.id = at.tag_id
            WHERE at.asset_id = ?1
            ORDER BY t.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| row.get::<_, String>(0))
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_asset_albums(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<crate::AlbumMembershipView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT a.id, a.name, a.kind
            FROM album_items ai
            INNER JOIN albums a ON a.id = ai.album_id
            WHERE ai.asset_id = ?1
            ORDER BY a.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![asset_id.0], |row| {
            let kind: String = row.get(2)?;
            Ok(crate::AlbumMembershipView {
                id: AlbumId(row.get(0)?),
                name: row.get(1)?,
                kind: if kind == "smart" {
                    AlbumKind::Smart
                } else {
                    AlbumKind::Manual
                },
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn pending_review_count(connection: &Connection, asset_id: &AssetId) -> DomainResult<u32> {
    let count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM metadata_suggestions
            WHERE asset_id = ?1 AND status = 'pending_review'
            ",
            params![asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count.max(0) as u32)
}

fn asset_in_album(
    connection: &Connection,
    album_id: &AlbumId,
    asset_id: &AssetId,
) -> DomainResult<bool> {
    let count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM album_items
            WHERE album_id = ?1 AND asset_id = ?2
            ",
            params![album_id.0, asset_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count > 0)
}

fn load_file_context(
    library_path: &Path,
    connection: &Connection,
    version: &VersionSummary,
) -> DomainResult<FileContextView> {
    let absolute_path = library_path.join(&version.file_path);
    let size_bytes = absolute_path.metadata().ok().map(|metadata| metadata.len());
    let integrity_status = if absolute_path.is_file() {
        let actual_checksum = file_digest(&absolute_path, &version.checksum_algorithm)?;
        if actual_checksum == version.checksum {
            "verified"
        } else {
            "hash_mismatch"
        }
    } else {
        "missing"
    };
    let (width, height): (Option<u32>, Option<u32>) = connection
        .query_row(
            "SELECT width, height FROM asset_versions WHERE id = ?1",
            params![version.id.0],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(database_error)?;

    Ok(FileContextView {
        filename: version
            .file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        relative_location: version.file_path.clone(),
        mime_type: version.mime_type.clone(),
        size_bytes,
        width,
        height,
        checksum_algorithm: version.checksum_algorithm.clone(),
        checksum: version.checksum.clone(),
        integrity_status: integrity_status.to_string(),
    })
}

fn load_all_asset_summaries(connection: &Connection) -> DomainResult<Vec<AssetSummary>> {
    let mut statement = connection
        .prepare("SELECT id, title, category, rating, status FROM assets ORDER BY created_at")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetSummary {
                id: AssetId(row.get(0)?),
                title: row.get(1)?,
                category: row.get(2)?,
                rating: row.get::<_, Option<u8>>(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn asset_has_all_tags(
    connection: &Connection,
    asset_id: &AssetId,
    tags: &[String],
) -> DomainResult<bool> {
    for tag in tags {
        let count: i64 = connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM asset_tags at
                INNER JOIN tags t ON t.id = at.tag_id
                WHERE at.asset_id = ?1 AND t.name = ?2
                ",
                params![asset_id.0, tag],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if count == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug, Clone)]
struct ExportVersionRow {
    asset_id: String,
    version_id: String,
    file_path: PathBuf,
    sha256: String,
    checksum_algorithm: String,
    checksum: String,
    mime_type: String,
}

#[derive(Debug, Clone)]
struct RepairVersionRow {
    version_id: AssetVersionId,
    file_path: PathBuf,
    sha256: String,
    checksum_algorithm: String,
    checksum: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    created_at: String,
}

fn load_repair_versions(connection: &Connection) -> DomainResult<Vec<RepairVersionRow>> {
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

fn update_repaired_version(
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
                CHECKSUM_MD5,
                dimensions.0,
                dimensions.1,
                version_id.0
            ],
        )
        .map_err(database_error)?;
    Ok(())
}

fn load_export_versions(
    connection: &Connection,
    album_id: Option<&crate::AlbumId>,
) -> DomainResult<Vec<ExportVersionRow>> {
    let sql = if album_id.is_some() {
        "
        SELECT av.asset_id, av.id, av.file_path, av.sha256,
               av.checksum_algorithm, COALESCE(av.checksum, av.sha256), av.mime_type
        FROM asset_versions av
        INNER JOIN album_items ai ON ai.asset_id = av.asset_id
        WHERE ai.album_id = ?1
        ORDER BY ai.sort_order, av.created_at
        "
    } else {
        "
        SELECT av.asset_id, av.id, av.file_path, av.sha256,
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
        sha256: row.get(3)?,
        checksum_algorithm: row.get(4)?,
        checksum: row.get(5)?,
        mime_type: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenerationService;
    use crate::MetadataReviewService;
    use crate::{
        AlbumService, GalleryReadService, GeneratedImage, GenerationResult, SearchService,
    };

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("imglab-core-{name}-{}", Uuid::new_v4()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove old test directory");
        }
        root
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes
    }

    fn move_version_to_legacy_state(
        root: &Path,
        version_id: &AssetVersionId,
        current_relative: &Path,
        legacy_relative: &Path,
    ) {
        let current = root.join(current_relative);
        let legacy = root.join(legacy_relative);
        fs::create_dir_all(legacy.parent().expect("legacy parent")).expect("create legacy parent");
        fs::rename(&current, &legacy).expect("move to legacy path");
        let sha256 = file_digest(&legacy, CHECKSUM_SHA256).expect("sha256");
        let connection =
            Connection::open(LocalLibraryService::database_path(root)).expect("open db");
        connection
            .execute(
                "
                UPDATE asset_versions
                SET file_path = ?1,
                    sha256 = ?2,
                    checksum_algorithm = 'SHA-256',
                    checksum = ?2,
                    width = NULL,
                    height = NULL
                WHERE id = ?3
                ",
                params![legacy_relative.to_string_lossy(), sha256, version_id.0],
            )
            .expect("update legacy version");
    }

    #[test]
    fn creates_managed_library_layout_and_registry() {
        let root = test_root("create-library");
        let registry = test_root("registry").join("registry.sqlite");
        let service = LocalLibraryService::new(registry);

        let summary = service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Test Library".to_string(),
            })
            .expect("create library");

        assert_eq!(summary.name, "Test Library");
        assert!(LocalLibraryService::manifest_path(&root).is_file());
        assert!(LocalLibraryService::database_path(&root).is_file());

        for relative in REQUIRED_DIRS {
            assert!(root.join(relative).is_dir(), "missing {relative}");
        }

        let libraries = service.list_libraries(false).expect("list libraries");
        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].id, summary.id);
    }

    #[test]
    fn hides_registered_library_without_deleting_files() {
        let root = test_root("hide-library");
        let registry = test_root("hide-registry").join("registry.sqlite");
        let service = LocalLibraryService::new(registry);
        let summary = service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Hidden Library".to_string(),
            })
            .expect("create library");

        service.hide_library(&summary.id).expect("hide library");

        assert!(root.exists());
        assert!(service.list_libraries(false).expect("visible").is_empty());
        assert_eq!(service.list_libraries(true).expect("all").len(), 1);
    }

    #[test]
    fn rejects_future_schema_manifest() {
        let root = test_root("future-schema");
        let registry = test_root("future-registry").join("registry.sqlite");
        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Future".to_string(),
            })
            .expect("create library");

        let mut manifest = LocalLibraryService::read_manifest(&root).expect("read manifest");
        manifest.schema_version = CURRENT_SCHEMA_VERSION + 1;
        LocalLibraryService::write_manifest(&root, &manifest).expect("write manifest");

        let error = service
            .open_library(&root)
            .expect_err("future schema should fail");
        assert!(matches!(error, DomainError::SchemaMismatch { .. }));
    }

    #[test]
    fn imports_asset_into_managed_originals() {
        let root = test_root("import-asset");
        let registry = test_root("import-registry").join("registry.sqlite");
        let source_dir = test_root("import-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"not really a png").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Import".to_string(),
            })
            .expect("create library");

        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        assert_eq!(asset.status, "imported");
        assert_eq!(version.mime_type, "image/png");
        assert!(version.generation_event_id.is_none());
        assert!(root.join(&version.file_path).is_file());
        assert_eq!(version.checksum_algorithm, CHECKSUM_MD5);
        assert_eq!(version.checksum, "a4f84feadf4cad85108478e074357b33");
        assert_eq!(version.sha256, version.checksum);
        let path_parts = version
            .file_path
            .iter()
            .map(|part| part.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(path_parts[0], "originals");
        assert_eq!(path_parts[1].len(), 4);
        assert_eq!(path_parts[2].len(), 2);
        assert!(path_parts[3].ends_with(".png"));
        assert!(Uuid::parse_str(path_parts[3].trim_end_matches(".png")).is_ok());

        let connection =
            Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
        let asset_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))
            .expect("asset count");
        let version_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM asset_versions", [], |row| row.get(0))
            .expect("version count");
        assert_eq!(asset_count, 1);
        assert_eq!(version_count, 1);
    }

    #[test]
    fn duplicate_source_filenames_get_distinct_uuid_paths() {
        let root = test_root("duplicate-filenames");
        let registry = test_root("duplicate-filenames-registry").join("registry.sqlite");
        let first_dir = test_root("duplicate-source-a");
        let second_dir = test_root("duplicate-source-b");
        fs::create_dir_all(&first_dir).expect("create first source dir");
        fs::create_dir_all(&second_dir).expect("create second source dir");
        let first = first_dir.join("input.png");
        let second = second_dir.join("input.png");
        fs::write(&first, b"first").expect("write first");
        fs::write(&second, b"second").expect("write second");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Duplicates".to_string(),
            })
            .expect("create library");

        let (_, first_version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: first,
            })
            .expect("import first");
        let (_, second_version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: second,
            })
            .expect("import second");

        assert_ne!(first_version.file_path, second_version.file_path);
        assert!(root.join(first_version.file_path).is_file());
        assert!(root.join(second_version.file_path).is_file());
    }

    #[test]
    fn imports_png_dimensions_into_file_context() {
        let root = test_root("import-dimensions");
        let registry = test_root("import-dimensions-registry").join("registry.sqlite");
        let source_dir = test_root("import-dimensions-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, png_bytes(640, 480)).expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Dimensions".to_string(),
            })
            .expect("create library");
        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&version.id))
            .expect("detail");
        let file = detail.file.expect("file");
        assert_eq!(file.width, Some(640));
        assert_eq!(file.height, Some(480));

        let gallery = service
            .query_gallery(&root, GalleryQuery::default())
            .expect("gallery");
        assert_eq!(gallery[0].width, Some(640));
        assert_eq!(gallery[0].height, Some(480));
    }

    #[test]
    fn unknown_binary_keeps_dimensions_empty() {
        let root = test_root("unknown-dimensions");
        let registry = test_root("unknown-dimensions-registry").join("registry.sqlite");
        let source_dir = test_root("unknown-dimensions-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"not really a png").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Unknown Dimensions".to_string(),
            })
            .expect("create library");
        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&version.id))
            .expect("detail");
        let file = detail.file.expect("file");
        assert_eq!(file.width, None);
        assert_eq!(file.height, None);
    }

    #[test]
    fn repair_library_dry_run_reports_legacy_path_checksum_and_dimensions() {
        let root = test_root("repair-dry-run");
        let registry = test_root("repair-dry-run-registry").join("registry.sqlite");
        let source_dir = test_root("repair-dry-run-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, png_bytes(640, 480)).expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Repair".to_string(),
            })
            .expect("create library");
        let (_, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");
        let old_relative = PathBuf::from("originals")
            .join("imported")
            .join(format!("{}.png", version.id.0));
        move_version_to_legacy_state(&root, &version.id, &version.file_path, &old_relative);

        let summary = service
            .repair_library(RepairLibraryRequest {
                library_path: root.clone(),
                dry_run: true,
            })
            .expect("repair dry run");

        assert_eq!(summary.scanned_versions, 1);
        assert_eq!(summary.files_moved, 1);
        assert_eq!(summary.paths_updated, 1);
        assert_eq!(summary.checksums_updated, 1);
        assert_eq!(summary.dimensions_updated, 1);
        assert!(summary.issues.is_empty());
        assert!(root.join(&old_relative).is_file());
        assert!(!root.join(&version.file_path).is_file());
    }

    #[test]
    fn repair_library_applies_legacy_path_checksum_and_dimensions() {
        let root = test_root("repair-apply");
        let registry = test_root("repair-apply-registry").join("registry.sqlite");
        let source_dir = test_root("repair-apply-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, png_bytes(800, 600)).expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Repair".to_string(),
            })
            .expect("create library");
        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");
        let old_relative = PathBuf::from("originals")
            .join("generated")
            .join(format!("{}.png", version.id.0));
        move_version_to_legacy_state(&root, &version.id, &version.file_path, &old_relative);

        let summary = service
            .repair_library(RepairLibraryRequest {
                library_path: root.clone(),
                dry_run: false,
            })
            .expect("repair apply");

        assert_eq!(summary.files_moved, 1);
        assert_eq!(summary.paths_updated, 1);
        assert_eq!(summary.checksums_updated, 1);
        assert_eq!(summary.dimensions_updated, 1);
        assert!(summary.issues.is_empty());
        assert!(!root.join(&old_relative).exists());
        assert!(root.join(&version.file_path).is_file());

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&version.id))
            .expect("detail");
        let file = detail.file.expect("file");
        assert_eq!(file.relative_location, version.file_path);
        assert_eq!(file.checksum_algorithm, CHECKSUM_MD5);
        assert_eq!(file.width, Some(800));
        assert_eq!(file.height, Some(600));
        assert!(service
            .check_integrity(&root)
            .expect("integrity")
            .is_empty());
    }

    #[test]
    fn repair_library_reports_missing_files_without_deleting_records() {
        let root = test_root("repair-missing");
        let registry = test_root("repair-missing-registry").join("registry.sqlite");
        let source_dir = test_root("repair-missing-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, png_bytes(16, 16)).expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Repair Missing".to_string(),
            })
            .expect("create library");
        let (_, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");
        fs::remove_file(root.join(&version.file_path)).expect("remove managed file");

        let summary = service
            .repair_library(RepairLibraryRequest {
                library_path: root.clone(),
                dry_run: false,
            })
            .expect("repair apply");

        assert_eq!(summary.scanned_versions, 1);
        assert_eq!(summary.issues.len(), 1);
        assert_eq!(summary.issues[0].version_id, version.id);

        let connection =
            Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM asset_versions", [], |row| row.get(0))
            .expect("count versions");
        assert_eq!(count, 1);
    }

    #[test]
    fn exports_imported_asset_with_sidecar() {
        let root = test_root("export-library");
        let registry = test_root("export-registry").join("registry.sqlite");
        let export_root = test_root("export-output");
        let source_dir = test_root("export-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.jpg");
        fs::write(&source, b"jpg bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Export".to_string(),
            })
            .expect("create library");
        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        let summary = service
            .export_library(ExportLibraryRequest {
                library_path: root,
                output_path: export_root.clone(),
                album_id: None,
            })
            .expect("export library");

        assert_eq!(summary.exported_files, 1);
        assert_eq!(summary.exported_sidecars, 1);
        assert!(export_root
            .join("originals")
            .join(format!("{}.jpg", version.id.0))
            .is_file());
        assert!(export_root
            .join("sidecars")
            .join(format!("{}.json", asset.id.0))
            .is_file());
    }

    #[test]
    fn reports_missing_managed_file() {
        let root = test_root("integrity-missing");
        let registry = test_root("integrity-registry").join("registry.sqlite");
        let source_dir = test_root("integrity-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.webp");
        fs::write(&source, b"webp bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Integrity".to_string(),
            })
            .expect("create library");
        let (_, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        fs::remove_file(root.join(&version.file_path)).expect("remove managed file");

        let issues = service.check_integrity(&root).expect("check integrity");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].version_id, version.id);
        assert_eq!(issues[0].kind, IntegrityIssueKind::MissingFile);
    }

    #[test]
    fn reports_md5_mismatch_for_modified_file() {
        let root = test_root("integrity-md5-mismatch");
        let registry = test_root("integrity-md5-mismatch-registry").join("registry.sqlite");
        let source_dir = test_root("integrity-md5-mismatch-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.webp");
        fs::write(&source, b"webp bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Integrity".to_string(),
            })
            .expect("create library");
        let (_, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        fs::write(root.join(&version.file_path), b"changed").expect("modify managed file");

        let issues = service.check_integrity(&root).expect("check integrity");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].version_id, version.id);
        assert_eq!(issues[0].kind, IntegrityIssueKind::HashMismatch);
        assert!(issues[0].message.contains(CHECKSUM_MD5));
    }

    #[test]
    fn legacy_sha256_versions_remain_readable() {
        let root = test_root("legacy-sha256");
        let registry = test_root("legacy-sha256-registry").join("registry.sqlite");
        let source_dir = test_root("legacy-sha256-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"legacy bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Legacy".to_string(),
            })
            .expect("create library");
        let (asset, version) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");
        let sha256 = file_digest(&root.join(&version.file_path), CHECKSUM_SHA256).expect("sha256");
        let connection =
            Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
        connection
            .execute(
                "
                UPDATE asset_versions
                SET sha256 = ?1,
                    checksum_algorithm = 'SHA-256',
                    checksum = ?1
                WHERE id = ?2
                ",
                params![sha256, version.id.0],
            )
            .expect("make legacy row");

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&version.id))
            .expect("detail");
        let file = detail.file.expect("file context");
        assert_eq!(file.checksum_algorithm, CHECKSUM_SHA256);
        assert_eq!(file.integrity_status, "verified");
        assert!(service
            .check_integrity(&root)
            .expect("integrity")
            .is_empty());
    }

    #[test]
    fn records_event_and_child_version_lineage() {
        let root = test_root("lineage");
        let registry = test_root("lineage-registry").join("registry.sqlite");
        let source_dir = test_root("lineage-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        let generated = source_dir.join("generated.png");
        fs::write(&source, b"input bytes").expect("write source");
        fs::write(&generated, b"generated bytes").expect("write generated");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Lineage".to_string(),
            })
            .expect("create library");
        let (asset, parent) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import parent");
        let event = service
            .record_generation_event(CreateGenerationEventRequest {
                library_path: root.clone(),
                asset_id: Some(asset.id.clone()),
                output_version_id: None,
                provider: "codex".to_string(),
                provider_model: "gpt-image-2".to_string(),
                operation_type: GenerationOperation::ImageToImage,
                prompt: "make a variant".to_string(),
                negative_prompt: None,
                input_asset_version_id: Some(parent.id.clone()),
                parameters_json: "{}".to_string(),
                raw_request_json: Some("{\"prompt\":\"make a variant\"}".to_string()),
                raw_response_json: Some("{\"ok\":true}".to_string()),
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })
            .expect("record event");
        let child = service
            .create_child_version(CreateChildVersionRequest {
                library_path: root.clone(),
                asset_id: asset.id,
                parent_version_id: parent.id.clone(),
                generation_event_id: Some(event.id.clone()),
                source_path: generated,
                mime_type: "image/png".to_string(),
                version_label: Some("variant".to_string()),
            })
            .expect("create child");

        let lineage = service.get_lineage(&root, &child.id).expect("lineage");
        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[0].version.id, child.id);
        assert_eq!(
            lineage[0].generation_event.as_ref().expect("event").id,
            event.id
        );
        assert_eq!(
            lineage[0]
                .generation_event
                .as_ref()
                .expect("event")
                .output_version_id
                .as_ref(),
            Some(&child.id)
        );
        assert_eq!(lineage[1].version.id, parent.id);
        assert!(lineage[1].generation_event.is_none());
    }

    #[test]
    fn generation_service_saves_fake_provider_output() {
        let root = test_root("generation-flow");
        let registry = test_root("generation-registry").join("registry.sqlite");
        let library = LocalLibraryService::new(registry);
        library
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Generation".to_string(),
            })
            .expect("create library");

        let generation = LocalGenerationService::new(crate::FakeImageProvider::success("fake"));
        let versions = generation
            .generate(GenerateImageRequest {
                library_path: root.clone(),
                input_bytes: None,
                parameters: crate::GenerationParameters {
                    library_path: Some(root.clone()),
                    provider: "fake".to_string(),
                    model: "fake-image".to_string(),
                    prompt: "make a test image".to_string(),
                    negative_prompt: None,
                    operation: GenerationOperation::TextToImage,
                    input_version_id: None,
                    parameters_json: "{}".to_string(),
                },
            })
            .expect("generate");

        assert_eq!(versions.len(), 1);
        assert!(root.join(&versions[0].file_path).is_file());
        assert!(versions[0].generation_event_id.is_some());

        let gallery = library
            .query_gallery(
                &root,
                GalleryQuery {
                    text: Some("test image".to_string()),
                    providers: vec!["fake".to_string()],
                    min_rating: None,
                    review_status: ReviewStatusFilter::Any,
                    tags: vec![],
                    album_id: None,
                    sort: GallerySort::Newest,
                },
            )
            .expect("gallery");
        assert_eq!(gallery.len(), 1);
        assert_eq!(gallery[0].id, versions[0].asset_id);
        assert_eq!(gallery[0].title.as_deref(), Some("Make Test Image"));
        assert_eq!(gallery[0].provider.as_deref(), Some("fake"));
        assert_eq!(gallery[0].model_label.as_deref(), Some("fake-image"));
        assert_eq!(gallery[0].prompt.as_deref(), Some("make a test image"));

        let detail = library
            .get_asset_detail(&root, &versions[0].asset_id, Some(&versions[0].id))
            .expect("detail");
        assert_eq!(detail.provider.as_deref(), Some("fake"));
        assert_eq!(detail.title.as_deref(), Some("Make Test Image"));
        assert_eq!(detail.model_label.as_deref(), Some("fake-image"));
        assert_eq!(detail.prompt.as_deref(), Some("make a test image"));
        assert_eq!(detail.parameters_json.as_deref(), Some("{}"));
    }

    #[derive(Debug, Clone)]
    struct PngProvider;

    impl ImageProvider for PngProvider {
        fn name(&self) -> &'static str {
            "png-provider"
        }

        fn validate_parameters(
            &self,
            _parameters: &crate::GenerationParameters,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn generate_from_text(
            &self,
            _parameters: &crate::GenerationParameters,
        ) -> DomainResult<GenerationResult> {
            Ok(GenerationResult {
                images: vec![GeneratedImage {
                    bytes: png_bytes(320, 240),
                    mime_type: "image/png".to_string(),
                    provider_metadata_json: "{}".to_string(),
                }],
                raw_request_json: "{}".to_string(),
                raw_response_json: "{}".to_string(),
            })
        }

        fn generate_from_image(
            &self,
            _parameters: &crate::GenerationParameters,
            _input: &[u8],
        ) -> DomainResult<GenerationResult> {
            self.generate_from_text(_parameters)
        }
    }

    #[test]
    fn generation_service_persists_output_dimensions() {
        let root = test_root("generation-dimensions");
        let registry = test_root("generation-dimensions-registry").join("registry.sqlite");
        let library = LocalLibraryService::new(registry);
        library
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Generation Dimensions".to_string(),
            })
            .expect("create library");

        let generation = LocalGenerationService::new(PngProvider);
        let versions = generation
            .generate(GenerateImageRequest {
                library_path: root.clone(),
                input_bytes: None,
                parameters: crate::GenerationParameters {
                    library_path: Some(root.clone()),
                    provider: "png-provider".to_string(),
                    model: "png-image".to_string(),
                    prompt: "make a png".to_string(),
                    negative_prompt: None,
                    operation: GenerationOperation::TextToImage,
                    input_version_id: None,
                    parameters_json: "{}".to_string(),
                },
            })
            .expect("generate");

        let detail = library
            .get_asset_detail(&root, &versions[0].asset_id, Some(&versions[0].id))
            .expect("detail");
        let file = detail.file.expect("file");
        assert_eq!(file.width, Some(320));
        assert_eq!(file.height, Some(240));
    }

    #[test]
    fn metadata_suggestions_are_review_first() {
        let root = test_root("metadata-review");
        let registry = test_root("metadata-registry").join("registry.sqlite");
        let source_dir = test_root("metadata-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"image bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        let library = service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Metadata".to_string(),
            })
            .expect("create library");
        let (asset, _) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import asset");

        let suggestion = service
            .create_suggestion(crate::CreateMetadataSuggestionRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                source: "fake".to_string(),
                suggested_title: Some("Suggested title".to_string()),
                suggested_description: Some("Suggested description".to_string()),
                suggested_tags: vec!["tag-a".to_string(), "tag-b".to_string()],
                suggested_category: Some("category-a".to_string()),
                confidence_json: "{}".to_string(),
            })
            .expect("create suggestion");

        let before = load_asset_summary(
            &Connection::open(LocalLibraryService::database_path(&root)).expect("open db"),
            &asset.id,
        )
        .expect("load asset");
        assert!(before.title.is_none());

        let pending = service
            .list_pending(&root, &library.id)
            .expect("list pending");
        assert_eq!(pending.len(), 1);

        let accepted = service
            .accept(ReviewMetadataSuggestionRequest {
                library_path: root.clone(),
                suggestion_id: suggestion.id,
                title: Some("Edited title".to_string()),
                description: Some("Edited description".to_string()),
                tags: vec!["tag-a".to_string()],
                category: Some("category-a".to_string()),
            })
            .expect("accept");

        assert_eq!(accepted.title.as_deref(), Some("Edited title"));
        assert!(service
            .list_pending(&root, &library.id)
            .expect("pending")
            .is_empty());
    }

    #[test]
    fn manages_manual_album_and_searches_assets() {
        let root = test_root("albums-search");
        let registry = test_root("albums-registry").join("registry.sqlite");
        let source_dir = test_root("albums-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"image bytes").expect("write source");

        let service = LocalLibraryService::new(registry);
        let library = service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Albums".to_string(),
            })
            .expect("create library");
        let (asset, _) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import");

        let album = service
            .create_manual_album(&library.id, "Favorites")
            .expect("album");
        service.add_asset(&album.id, &asset.id).expect("add asset");
        service
            .update_asset_metadata(UpdateAssetMetadataRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                title: None,
                description: None,
                rating: Some(5),
                category: Some("icons".to_string()),
                status: Some("curated".to_string()),
            })
            .expect("update metadata");

        let results = service
            .search(
                &library.id,
                SearchQuery {
                    text: None,
                    tags: vec![],
                    min_rating: Some(4),
                    provider: None,
                    status: Some("curated".to_string()),
                    category: Some("icons".to_string()),
                },
            )
            .expect("search");

        assert_eq!(results.len(), 1);

        service
            .add_tag_to_asset(&root, &asset.id, "favorite")
            .expect("tag asset");
        service
            .record_generation_event(CreateGenerationEventRequest {
                library_path: root.clone(),
                asset_id: Some(asset.id.clone()),
                output_version_id: None,
                provider: "fake".to_string(),
                provider_model: "fake-image".to_string(),
                operation_type: GenerationOperation::TextToImage,
                prompt: "tiny icon sheet".to_string(),
                negative_prompt: None,
                input_asset_version_id: None,
                parameters_json: "{}".to_string(),
                raw_request_json: None,
                raw_response_json: None,
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })
            .expect("record event");

        let prompt_results = service
            .search(
                &library.id,
                SearchQuery {
                    text: Some("icon sheet".to_string()),
                    tags: vec![],
                    min_rating: None,
                    provider: Some("fake".to_string()),
                    status: None,
                    category: None,
                },
            )
            .expect("search by prompt");
        assert_eq!(prompt_results.len(), 1);

        let tag_text_results = service
            .search(
                &library.id,
                SearchQuery {
                    text: Some("favor".to_string()),
                    tags: vec![],
                    min_rating: None,
                    provider: None,
                    status: None,
                    category: None,
                },
            )
            .expect("search by tag text");
        assert_eq!(tag_text_results.len(), 1);
    }

    #[test]
    fn gallery_query_filters_and_sorts_cards() {
        let root = test_root("gallery-query");
        let registry = test_root("gallery-registry").join("registry.sqlite");
        let source_dir = test_root("gallery-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let first_source = source_dir.join("first.png");
        let second_source = source_dir.join("second.png");
        fs::write(&first_source, b"first image").expect("write first");
        fs::write(&second_source, b"second image").expect("write second");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Gallery".to_string(),
            })
            .expect("create library");
        let (first, _) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: first_source,
            })
            .expect("import first");
        let (second, _) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: second_source,
            })
            .expect("import second");

        service
            .update_asset_metadata(UpdateAssetMetadataRequest {
                library_path: root.clone(),
                asset_id: first.id.clone(),
                title: None,
                description: None,
                rating: Some(5),
                category: Some("botanical".to_string()),
                status: Some("curated".to_string()),
            })
            .expect("update first");
        service
            .update_asset_metadata(UpdateAssetMetadataRequest {
                library_path: root.clone(),
                asset_id: second.id,
                title: None,
                description: None,
                rating: Some(2),
                category: Some("city".to_string()),
                status: Some("imported".to_string()),
            })
            .expect("update second");
        service
            .add_tag_to_asset(&root, &first.id, "neon")
            .expect("tag first");
        service
            .record_generation_event(CreateGenerationEventRequest {
                library_path: root.clone(),
                asset_id: Some(first.id.clone()),
                output_version_id: None,
                provider: "codex-cli".to_string(),
                provider_model: "codex-imagegen".to_string(),
                operation_type: GenerationOperation::TextToImage,
                prompt: "neon botanical study".to_string(),
                negative_prompt: None,
                input_asset_version_id: None,
                parameters_json: "{}".to_string(),
                raw_request_json: None,
                raw_response_json: None,
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })
            .expect("record event");
        service
            .create_suggestion(crate::CreateMetadataSuggestionRequest {
                library_path: root.clone(),
                asset_id: first.id.clone(),
                source: "fake".to_string(),
                suggested_title: Some("Neon Botanical Study".to_string()),
                suggested_description: None,
                suggested_tags: vec!["neon".to_string()],
                suggested_category: Some("botanical".to_string()),
                confidence_json: "{}".to_string(),
            })
            .expect("create suggestion");

        let results = service
            .query_gallery(
                &root,
                GalleryQuery {
                    text: Some("botanical".to_string()),
                    providers: vec!["codex-cli".to_string()],
                    min_rating: Some(4),
                    review_status: ReviewStatusFilter::Pending,
                    tags: vec!["neon".to_string()],
                    album_id: None,
                    sort: GallerySort::RatingDesc,
                },
            )
            .expect("query gallery");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, first.id);
        assert_eq!(results[0].provider.as_deref(), Some("codex-cli"));
        assert_eq!(results[0].review_pending_count, 1);
        assert_eq!(results[0].tags, vec!["neon".to_string()]);
    }

    #[test]
    fn asset_detail_aggregates_lineage_and_file_context() {
        let root = test_root("asset-detail");
        let registry = test_root("asset-detail-registry").join("registry.sqlite");
        let source_dir = test_root("asset-detail-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        let generated = source_dir.join("generated.png");
        fs::write(&source, b"input bytes").expect("write source");
        fs::write(&generated, b"generated bytes").expect("write generated");

        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Detail".to_string(),
            })
            .expect("create library");
        let (asset, parent) = service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import parent");
        service
            .add_tag_to_asset(&root, &asset.id, "study")
            .expect("tag");
        let event = service
            .record_generation_event(CreateGenerationEventRequest {
                library_path: root.clone(),
                asset_id: Some(asset.id.clone()),
                output_version_id: None,
                provider: "fake".to_string(),
                provider_model: "fake-image".to_string(),
                operation_type: GenerationOperation::ImageToImage,
                prompt: "make a variant".to_string(),
                negative_prompt: Some("blur".to_string()),
                input_asset_version_id: Some(parent.id.clone()),
                parameters_json: "{\"seed\":7}".to_string(),
                raw_request_json: None,
                raw_response_json: None,
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })
            .expect("event");
        let child = service
            .create_child_version(CreateChildVersionRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                parent_version_id: parent.id,
                generation_event_id: Some(event.id),
                source_path: generated,
                mime_type: "image/png".to_string(),
                version_label: Some("variant".to_string()),
            })
            .expect("child");

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&child.id))
            .expect("detail");

        assert_eq!(detail.id, asset.id);
        assert_eq!(detail.prompt.as_deref(), Some("make a variant"));
        assert_eq!(detail.negative_prompt.as_deref(), Some("blur"));
        assert_eq!(detail.provider.as_deref(), Some("fake"));
        assert_eq!(detail.tags, vec!["study".to_string()]);
        assert_eq!(detail.versions.len(), 2);
        assert_eq!(detail.lineage.len(), 2);
        assert_eq!(
            detail.file.as_ref().expect("file").integrity_status,
            "verified"
        );
    }

    #[derive(Debug, Clone)]
    struct TextOnlyProvider;

    impl ImageProvider for TextOnlyProvider {
        fn name(&self) -> &'static str {
            "text-only"
        }

        fn validate_parameters(
            &self,
            _parameters: &crate::GenerationParameters,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn generate_from_text(
            &self,
            _parameters: &crate::GenerationParameters,
        ) -> DomainResult<GenerationResult> {
            Ok(GenerationResult {
                images: vec![GeneratedImage {
                    bytes: b"image".to_vec(),
                    mime_type: "image/png".to_string(),
                    provider_metadata_json: "{}".to_string(),
                }],
                raw_request_json: "{}".to_string(),
                raw_response_json: "{}".to_string(),
            })
        }

        fn generate_from_image(
            &self,
            _parameters: &crate::GenerationParameters,
            _input: &[u8],
        ) -> DomainResult<GenerationResult> {
            unreachable!("capability check should reject image-to-image first")
        }
    }

    #[test]
    fn generation_service_rejects_unsupported_provider_capability() {
        let root = test_root("unsupported-capability");
        let registry = test_root("unsupported-capability-registry").join("registry.sqlite");
        let source_dir = test_root("unsupported-capability-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("input.png");
        fs::write(&source, b"input bytes").expect("write source");

        let library = LocalLibraryService::new(registry);
        library
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Capability".to_string(),
            })
            .expect("create library");
        let (_, version) = library
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import");

        let generation = LocalGenerationService::new(TextOnlyProvider);
        let error = generation
            .generate(GenerateImageRequest {
                library_path: root.clone(),
                input_bytes: Some(b"input bytes".to_vec()),
                parameters: crate::GenerationParameters {
                    library_path: Some(root),
                    provider: "text-only".to_string(),
                    model: "text-only-image".to_string(),
                    prompt: "make a variant".to_string(),
                    negative_prompt: None,
                    operation: GenerationOperation::ImageToImage,
                    input_version_id: Some(version.id),
                    parameters_json: "{}".to_string(),
                },
            })
            .expect_err("unsupported capability");

        assert!(matches!(
            error,
            DomainError::UnsupportedProviderCapability { .. }
        ));
    }

    #[test]
    fn rejects_unsupported_smart_album_field() {
        let root = test_root("smart-album");
        let registry = test_root("smart-registry").join("registry.sqlite");
        let service = LocalLibraryService::new(registry);
        service
            .create_library(CreateLibraryRequest {
                root_path: root.clone(),
                name: "Smart".to_string(),
            })
            .expect("create library");

        let error = service
            .create_smart_album(crate::CreateSmartAlbumRequest {
                library_path: root,
                name: "Bad".to_string(),
                smart_query_json: "{\"unknown\":true}".to_string(),
            })
            .expect_err("invalid smart query");

        assert!(matches!(error, DomainError::InvalidSmartAlbumQuery { .. }));
    }
}
