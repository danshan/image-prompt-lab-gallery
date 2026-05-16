use crate::{
    hash::sha256_reader, AlbumId, AlbumKind, AlbumService, AlbumSummary, AssetId, AssetService,
    AssetSummary, AssetVersionId, CreateChildVersionRequest, CreateGenerationEventRequest,
    CreateLibraryRequest, DomainError, DomainResult, ExportLibraryRequest, ExportSummary,
    GenerateImageRequest, GenerationEventId, GenerationEventSummary, GenerationOperation,
    ImageProvider, ImportAssetRequest, IntegrityIssue, IntegrityIssueKind, LibraryId,
    LibraryService, LibrarySummary, LineageEntry, MetadataReviewService, MetadataSuggestion,
    MetadataSuggestionId, ReviewMetadataSuggestionRequest, SearchQuery, SearchService,
    UpdateAssetMetadataRequest, VersionSummary,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

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

            let event_asset_id = asset_id.clone();
            let event = library_service.record_generation_event(CreateGenerationEventRequest {
                library_path: request.library_path.clone(),
                asset_id: event_asset_id.clone(),
                output_version_id: None,
                provider: request.parameters.provider.clone(),
                provider_model: request.parameters.model.clone(),
                operation_type: request.parameters.operation,
                prompt: request.parameters.prompt.clone(),
                negative_prompt: request.parameters.negative_prompt.clone(),
                input_asset_version_id: parent_version_id.clone(),
                parameters_json: request.parameters.parameters_json.clone(),
                raw_request_json: Some(result.raw_request_json.clone()),
                raw_response_json: Some(result.raw_response_json.clone()),
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })?;

            let version = if let (Some(asset_id), Some(parent_version_id)) =
                (event_asset_id, parent_version_id.clone())
            {
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
                let (_, version) = library_service.import_asset(ImportAssetRequest {
                    library_path: request.library_path.clone(),
                    source_path: temp_file,
                })?;
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

    fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
        let connection = Self::open_library_database(root_path)?;
        let mut statement = connection
            .prepare("SELECT id, file_path, sha256 FROM asset_versions ORDER BY created_at")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    PathBuf::from(row.get::<_, String>(1)?),
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(database_error)?;

        let mut issues = Vec::new();
        for row in rows {
            let (version_id, relative_path, expected_sha256) = row.map_err(database_error)?;
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

            let actual_sha256 =
                sha256_reader(File::open(&path).map_err(|error| io_error(&path, error))?)
                    .map_err(|error| io_error(&path, error))?;
            if actual_sha256 != expected_sha256 {
                issues.push(IntegrityIssue {
                    version_id: AssetVersionId(version_id),
                    path: relative_path,
                    kind: IntegrityIssueKind::HashMismatch,
                    message: format!("expected sha256 {expected_sha256}, found {actual_sha256}"),
                });
            }
        }

        Ok(issues)
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
        let relative_path = PathBuf::from("originals")
            .join("imported")
            .join(format!("{}.{}", version_id.0, extension));
        let destination_path = request.library_path.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let temporary_path = destination_path.with_extension(format!("{extension}.tmp"));
        fs::copy(&request.source_path, &temporary_path)
            .map_err(|error| io_error(&temporary_path, error))?;
        fs::rename(&temporary_path, &destination_path)
            .map_err(|error| io_error(&destination_path, error))?;

        let sha256 = sha256_reader(
            File::open(&destination_path).map_err(|error| io_error(&destination_path, error))?,
        )
        .map_err(|error| io_error(&destination_path, error))?;
        let mime_type = mime_type_for_extension(&extension).to_string();
        let now = timestamp_string();

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
                    file_path, sha256, width, height, mime_type, version_label, created_at
                )
                VALUES (?1, ?2, NULL, NULL, ?3, ?4, NULL, NULL, ?5, ?6, ?7)
                ",
                params![
                    version_id.0,
                    asset_id.0,
                    relative_path.to_string_lossy(),
                    sha256,
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
                sha256,
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
        let relative_path = PathBuf::from("originals")
            .join("generated")
            .join(format!("{}.{}", version_id.0, extension));
        let destination_path = request.library_path.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let temporary_path = destination_path.with_extension(format!("{extension}.tmp"));
        fs::copy(&request.source_path, &temporary_path)
            .map_err(|error| io_error(&temporary_path, error))?;
        fs::rename(&temporary_path, &destination_path)
            .map_err(|error| io_error(&destination_path, error))?;

        let sha256 = sha256_reader(
            File::open(&destination_path).map_err(|error| io_error(&destination_path, error))?,
        )
        .map_err(|error| io_error(&destination_path, error))?;
        let now = timestamp_string();

        connection
            .execute(
                "
                INSERT INTO asset_versions (
                    id, asset_id, parent_version_id, generation_event_id,
                    file_path, sha256, width, height, mime_type, version_label, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, ?8, ?9)
                ",
                params![
                    version_id.0,
                    request.asset_id.0,
                    request.parent_version_id.0,
                    request.generation_event_id.as_ref().map(|id| id.0.as_str()),
                    relative_path.to_string_lossy(),
                    sha256,
                    request.mime_type,
                    request.version_label.as_deref(),
                    now
                ],
            )
            .map_err(database_error)?;

        Ok(VersionSummary {
            id: version_id,
            asset_id: request.asset_id,
            parent_version_id: Some(request.parent_version_id),
            generation_event_id: request.generation_event_id,
            file_path: relative_path,
            sha256,
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
                SET rating = COALESCE(?1, rating),
                    category = COALESCE(?2, category),
                    status = COALESCE(?3, status),
                    updated_at = ?4
                WHERE id = ?5
                ",
                params![
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

            PRAGMA user_version = 1;
            ",
        )
        .map_err(database_error)?;

    Ok(())
}

fn timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
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
            SELECT id, asset_id, parent_version_id, generation_event_id, file_path, sha256, mime_type
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
                    mime_type: row.get(6)?,
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
        assets.retain(|asset| {
            asset
                .title
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains(&needle)
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
    mime_type: String,
}

fn load_export_versions(
    connection: &Connection,
    album_id: Option<&crate::AlbumId>,
) -> DomainResult<Vec<ExportVersionRow>> {
    let sql = if album_id.is_some() {
        "
        SELECT av.asset_id, av.id, av.file_path, av.sha256, av.mime_type
        FROM asset_versions av
        INNER JOIN album_items ai ON ai.asset_id = av.asset_id
        WHERE ai.album_id = ?1
        ORDER BY ai.sort_order, av.created_at
        "
    } else {
        "
        SELECT av.asset_id, av.id, av.file_path, av.sha256, av.mime_type
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
        mime_type: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenerationService;
    use crate::MetadataReviewService;
    use crate::{AlbumService, SearchService};

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("imglab-core-{name}-{}", Uuid::new_v4()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove old test directory");
        }
        root
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
        assert_eq!(
            version.sha256,
            "e90137d39de304eefbbe788bc535c7e82f27abbf8069505fbbd8a9dcdc4f2024"
        );

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
                library_path: root,
                asset_id: asset.id,
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
