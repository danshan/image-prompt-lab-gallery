use crate::{
    AssetId, AssetService, AssetVersionId, CreateChildVersionRequest, CreateGenerationEventRequest,
    CreateLibraryRequest, CreateMetadataSuggestionRequest, DomainError, DomainResult,
    ExportLibraryRequest, ExportSummary, GenerateImageRequest, GenerationOperation,
    GenerationParameters, GenerationRequestInput, ImageProvider, ImportAssetRequest,
    IntegrityIssue, IntegrityIssueKind, LibraryId, LibraryService, LibraryStatusView,
    LibrarySummary, MetadataReviewService, PreparedGenerationRequest, RepairIssue,
    RepairLibraryRequest, RepairSummary, VersionSummary,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod schema;
pub use schema::{migrate_library_database, CURRENT_SCHEMA_VERSION};
mod albums;
mod assets;
#[cfg(test)]
use assets::load_asset_summary;
use assets::{
    default_title_from_prompt, ensure_asset_exists, load_version,
    mark_imported_version_as_generated,
};
mod export;
use export::{load_export_versions, ExportVersionRow};
mod gallery;
mod metadata;
use metadata::attach_tag;
mod repair;
use repair::{load_repair_versions, update_repaired_version};
mod storage;
use storage::{
    extension_for_mime_type, file_digest, image_dimensions, is_safe_relative_path,
    managed_original_path, managed_storage_size, normalized_extension, timestamp_string,
};

const CHECKSUM_MD5: &str = "MD5";
const CHECKSUM_SHA256: &str = "SHA-256";
const CURRENT_CHECKSUM_ALGORITHM: &str = CHECKSUM_SHA256;

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
        attach_tag(&connection, asset_id, tag, "manual", &now)?;
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

            create_generation_metadata_suggestion(&library_service, &request, &version.asset_id)?;
            versions.push(version);
        }

        Ok(versions)
    }
}

fn create_generation_metadata_suggestion(
    library_service: &LocalLibraryService,
    request: &GenerateImageRequest,
    asset_id: &AssetId,
) -> DomainResult<()> {
    library_service.create_suggestion(CreateMetadataSuggestionRequest {
        library_path: request.library_path.clone(),
        asset_id: asset_id.clone(),
        source: format!("generation:{}", request.parameters.provider),
        suggested_title: default_title_from_prompt(&request.parameters.prompt),
        suggested_description: None,
        suggested_schema_prompt: Some(schema_prompt_draft_from_generation(&request.parameters)?),
        suggested_tags: Vec::new(),
        suggested_category: None,
        confidence_json: json!({
            "source": "generation",
            "provider": request.parameters.provider,
            "model": request.parameters.model,
            "operation": operation_to_str(request.parameters.operation),
        })
        .to_string(),
    })?;
    Ok(())
}

fn schema_prompt_draft_from_generation(parameters: &GenerationParameters) -> DomainResult<String> {
    let aspect_ratio = generation_parameter_value(&parameters.parameters_json, "aspect_ratio")
        .or_else(|| generation_parameter_value(&parameters.parameters_json, "aspectRatio"))
        .unwrap_or_else(|| "unspecified".to_string());
    let schema = json!({
        "GLOBAL_SETTINGS": {
            "aspect_ratio": aspect_ratio,
            "style": "derived from source prompt",
            "clarity": "sharp foreground, readable subject detail",
            "render_flags": ["sharp_foreground", "micro_texture", "editorial_finish"]
        },
        "ENVIRONMENT": {
            "background": "preserve the generated image environment cues",
            "lighting": "preserve the generated image lighting direction and contrast",
            "atmosphere": ["match the final image mood", "avoid unsupported scene changes"]
        },
        "CORE_ASSETS": {
            "primary_subject": parameters.prompt,
            "materials": ["infer visible materials from final image"],
            "composition": "preserve the generated composition and camera framing"
        },
        "MOTION_OR_DETAIL_SYSTEMS": [
            {
                "object": "visible detail systems",
                "state": "preserve the generated image behavior and placement"
            }
        ],
        "OUTPUT": {
            "mood": "match the accepted visual direction",
            "avoid": ["cheap e-commerce banner", "plastic CGI", "fake brand logos"]
        }
    });
    let body = serde_json::to_string_pretty(&schema).map_err(serialization_error)?;
    Ok(format!(
        "// VERSION: 0.1\n// AESTHETIC: derived from generation prompt\n{body}"
    ))
}

fn generation_parameter_value(parameters_json: &str, key: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(parameters_json).ok()?;
    value.get(key).and_then(|value| match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

pub fn prepare_generation_request(
    input: GenerationRequestInput,
) -> DomainResult<PreparedGenerationRequest> {
    let provider = normalize_provider_name(&input.provider)?;
    let operation =
        infer_generation_operation(input.input_file.as_ref(), input.input_version_id.as_ref());
    let input_bytes = load_generation_input_bytes(
        &input.library_path,
        input.input_file.as_ref(),
        input.input_version_id.as_ref(),
    )?;
    let parameters = GenerationParameters {
        library_path: Some(input.library_path.clone()),
        provider: provider.clone(),
        model: default_generation_model_label(&provider).to_string(),
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        operation,
        input_version_id: input.input_version_id,
        parameters_json: input.parameters_json.unwrap_or_else(|| "{}".to_string()),
    };

    Ok(PreparedGenerationRequest {
        provider,
        request: GenerateImageRequest {
            library_path: input.library_path,
            parameters,
            input_bytes,
        },
    })
}

pub fn normalize_provider_name(provider: &str) -> DomainResult<String> {
    match provider {
        "codex" | "codex-cli" => Ok("codex-cli".to_string()),
        "fake" => Ok("fake".to_string()),
        other => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported provider: {other}"),
        }),
    }
}

fn default_generation_model_label(provider: &str) -> &'static str {
    match provider {
        "fake" => "fake-image",
        _ => "imagegen-skill",
    }
}

fn infer_generation_operation(
    input_file: Option<&PathBuf>,
    input_version_id: Option<&AssetVersionId>,
) -> GenerationOperation {
    if input_file.is_some() || input_version_id.is_some() {
        GenerationOperation::ImageToImage
    } else {
        GenerationOperation::TextToImage
    }
}

fn load_generation_input_bytes(
    library_path: &Path,
    input_file: Option<&PathBuf>,
    input_version_id: Option<&AssetVersionId>,
) -> DomainResult<Option<Vec<u8>>> {
    if let Some(path) = input_file {
        return fs::read(path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    if let Some(version_id) = input_version_id {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let version = load_version(&connection, version_id)?;
        let path = library_path.join(version.file_path);
        return fs::read(&path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenerationService;
    use crate::MetadataReviewService;
    use crate::{
        AlbumKind, AlbumService, GalleryQuery, GalleryReadService, GallerySort, GeneratedImage,
        GenerationResult, ReviewMetadataSuggestionRequest, ReviewStatusFilter, SearchQuery,
        SearchService, UpdateAssetMetadataRequest,
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
        let md5 = file_digest(&legacy, CHECKSUM_MD5).expect("md5");
        let connection =
            Connection::open(LocalLibraryService::database_path(root)).expect("open db");
        connection
            .execute(
                "
                UPDATE asset_versions
                SET file_path = ?1,
                    sha256 = ?2,
                    checksum_algorithm = 'MD5',
                    checksum = ?2,
                    width = NULL,
                    height = NULL
                WHERE id = ?3
                ",
                params![legacy_relative.to_string_lossy(), md5, version_id.0],
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
        assert_eq!(version.checksum_algorithm, CHECKSUM_SHA256);
        assert_eq!(
            version.checksum,
            "e90137d39de304eefbbe788bc535c7e82f27abbf8069505fbbd8a9dcdc4f2024"
        );
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
        assert_eq!(file.checksum_algorithm, CHECKSUM_SHA256);
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
    fn reports_sha256_mismatch_for_modified_file() {
        let root = test_root("integrity-sha256-mismatch");
        let registry = test_root("integrity-sha256-mismatch-registry").join("registry.sqlite");
        let source_dir = test_root("integrity-sha256-mismatch-source");
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
        assert!(issues[0].message.contains(CHECKSUM_SHA256));
    }

    #[test]
    fn legacy_md5_versions_remain_readable_before_repair() {
        let root = test_root("legacy-md5");
        let registry = test_root("legacy-md5-registry").join("registry.sqlite");
        let source_dir = test_root("legacy-md5-source");
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
        let md5 = file_digest(&root.join(&version.file_path), CHECKSUM_MD5).expect("md5");
        let connection =
            Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
        connection
            .execute(
                "
                UPDATE asset_versions
                SET sha256 = ?1,
                    checksum_algorithm = 'MD5',
                    checksum = ?1
                WHERE id = ?2
                ",
                params![md5, version.id.0],
            )
            .expect("make legacy row");

        let detail = service
            .get_asset_detail(&root, &asset.id, Some(&version.id))
            .expect("detail");
        let file = detail.file.expect("file context");
        assert_eq!(file.checksum_algorithm, CHECKSUM_MD5);
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
        let library_summary = library
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
        assert_eq!(gallery[0].review_pending_count, 1);

        let suggestions = library
            .list_pending(&root, &library_summary.id)
            .expect("pending suggestions");
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].asset_id, versions[0].asset_id);
        assert_eq!(
            suggestions[0].suggested_title.as_deref(),
            Some("Make Test Image")
        );
        assert!(suggestions[0]
            .suggested_schema_prompt
            .as_deref()
            .unwrap_or_default()
            .contains("\"primary_subject\": \"make a test image\""));

        let detail = library
            .get_asset_detail(&root, &versions[0].asset_id, Some(&versions[0].id))
            .expect("detail");
        assert_eq!(detail.provider.as_deref(), Some("fake"));
        assert_eq!(detail.title.as_deref(), Some("Make Test Image"));
        assert_eq!(detail.model_label.as_deref(), Some("fake-image"));
        assert_eq!(detail.prompt.as_deref(), Some("make a test image"));
        assert_eq!(detail.parameters_json.as_deref(), Some("{}"));
        assert_eq!(detail.review_pending_count, 1);
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
        service
            .update_asset_metadata(UpdateAssetMetadataRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                title: None,
                description: None,
                schema_prompt: None,
                rating: None,
                category: Some("category-a".to_string()),
                status: None,
            })
            .expect("seed category");

        let suggestion = service
            .create_suggestion(crate::CreateMetadataSuggestionRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                source: "fake".to_string(),
                suggested_title: Some("Suggested title".to_string()),
                suggested_description: Some("Suggested description".to_string()),
                suggested_schema_prompt: Some("{\"OUTPUT\":{\"mood\":\"edited\"}}".to_string()),
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
                schema_prompt: Some("{\"OUTPUT\":{\"mood\":\"edited\"}}".to_string()),
                tags: vec!["tag-a".to_string()],
                category: Some("category-a".to_string()),
            })
            .expect("accept");

        assert_eq!(accepted.title.as_deref(), Some("Edited title"));
        let detail = service
            .get_asset_detail(&root, &asset.id, None)
            .expect("detail");
        assert_eq!(detail.description.as_deref(), Some("Edited description"));
        assert_eq!(
            detail.schema_prompt.as_deref(),
            Some("{\"OUTPUT\":{\"mood\":\"edited\"}}")
        );
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

        assert!(service
            .list_albums(&library.id)
            .expect("list empty albums")
            .is_empty());

        let album = service
            .create_manual_album(&library.id, "Favorites")
            .expect("album");
        let albums = service.list_albums(&library.id).expect("list albums");
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].id, album.id);
        assert_eq!(albums[0].name, "Favorites");
        assert_eq!(albums[0].kind, AlbumKind::Manual);
        assert_eq!(albums[0].item_count, 0);

        service.add_asset(&album.id, &asset.id).expect("add asset");
        service
            .add_asset(&album.id, &asset.id)
            .expect("duplicate add is no-op");
        let albums = service.list_albums(&library.id).expect("list albums");
        assert_eq!(albums[0].item_count, 1);

        let album_results = service
            .query_gallery(
                &root,
                GalleryQuery {
                    album_id: Some(album.id.clone()),
                    ..GalleryQuery::default()
                },
            )
            .expect("query album gallery");
        assert_eq!(album_results.len(), 1);
        assert_eq!(album_results[0].id, asset.id);

        service
            .update_asset_metadata(UpdateAssetMetadataRequest {
                library_path: root.clone(),
                asset_id: asset.id.clone(),
                title: None,
                description: None,
                schema_prompt: None,
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
                schema_prompt: None,
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
                schema_prompt: None,
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
                suggested_schema_prompt: None,
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
