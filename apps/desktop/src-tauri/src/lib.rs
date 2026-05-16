use imglab_core::{
    AlbumService, AssetId, AssetService, CreateLibraryRequest, CreateMetadataSuggestionRequest,
    DomainError, ExportLibraryRequest, GenerateImageRequest, GenerationOperation,
    GenerationParameters, GenerationService, ImageProvider, ImportAssetRequest, LibraryId,
    LibraryService, LocalGenerationService, LocalLibraryService, MetadataReviewService,
    MetadataSuggestionId, ReviewMetadataSuggestionRequest, SearchQuery, SearchService,
    UpdateAssetMetadataRequest,
};
use imglab_provider_codex::CodexCliImageProvider;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Default)]
struct DesktopState {
    generation_jobs: Arc<Mutex<HashMap<String, GenerationJobState>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandError {
    code: String,
    message: String,
    recoverable: bool,
}

impl From<DomainError> for CommandError {
    fn from(error: DomainError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
            recoverable: error.recoverable(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryView {
    id: String,
    name: String,
    root_path: PathBuf,
    hidden: bool,
    schema_version: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetView {
    id: String,
    title: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GalleryItemView {
    id: String,
    title: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    image_path: Option<PathBuf>,
    version_id: Option<String>,
    mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionView {
    id: String,
    asset_id: String,
    parent_version_id: Option<String>,
    generation_event_id: Option<String>,
    file_path: PathBuf,
    sha256: String,
    mime_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AlbumView {
    id: String,
    name: String,
    kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuggestionView {
    id: String,
    asset_id: String,
    title: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationJobView {
    id: String,
    provider: String,
    prompt: String,
    status: String,
    log_path: PathBuf,
    error: Option<String>,
    versions: Vec<VersionView>,
}

#[derive(Debug, Clone)]
struct GenerationJobState {
    id: String,
    provider: String,
    prompt: String,
    status: String,
    log_path: PathBuf,
    error: Option<String>,
    versions: Vec<VersionView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateLibraryInput {
    root_path: PathBuf,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportAssetInput {
    library_path: PathBuf,
    source_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportLibraryInput {
    library_path: PathBuf,
    output_path: PathBuf,
    album_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchInput {
    library_path: PathBuf,
    text: Option<String>,
    tags: Vec<String>,
    min_rating: Option<u8>,
    provider: Option<String>,
    status: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GalleryItemsInput {
    library_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateImageInput {
    library_path: PathBuf,
    provider: String,
    prompt: String,
    negative_prompt: Option<String>,
    input_file: Option<PathBuf>,
    input_version_id: Option<String>,
    parameters_json: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMetadataInput {
    library_path: PathBuf,
    asset_id: String,
    rating: Option<u8>,
    category: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddTagInput {
    library_path: PathBuf,
    asset_id: String,
    tag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAlbumInput {
    library_path: PathBuf,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddAlbumAssetInput {
    album_id: String,
    asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSuggestionInput {
    library_path: PathBuf,
    asset_id: String,
    title: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
    confidence_json: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewSuggestionInput {
    library_path: PathBuf,
    suggestion_id: String,
    title: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
}

#[tauri::command]
fn health() -> &'static str {
    "ok"
}

#[tauri::command]
fn create_library(input: CreateLibraryInput) -> Result<LibraryView, CommandError> {
    service()
        .create_library(CreateLibraryRequest {
            root_path: input.root_path,
            name: input.name,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn list_libraries(include_hidden: bool) -> Result<Vec<LibraryView>, CommandError> {
    service()
        .list_libraries(include_hidden)
        .map(|libraries| libraries.into_iter().map(library_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn open_library(root_path: PathBuf) -> Result<LibraryView, CommandError> {
    service()
        .open_library(&root_path)
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn hide_library(library_id: String) -> Result<(), CommandError> {
    service()
        .hide_library(&LibraryId(library_id))
        .map_err(Into::into)
}

#[tauri::command]
fn import_asset(input: ImportAssetInput) -> Result<(AssetView, VersionView), CommandError> {
    service()
        .import_asset(ImportAssetRequest {
            library_path: input.library_path,
            source_path: input.source_path,
        })
        .map(|(asset, version)| (asset_view(asset), version_view(version)))
        .map_err(Into::into)
}

#[tauri::command]
fn export_library(input: ExportLibraryInput) -> Result<serde_json::Value, CommandError> {
    service()
        .export_library(ExportLibraryRequest {
            library_path: input.library_path,
            output_path: input.output_path,
            album_id: input.album_id.map(imglab_core::AlbumId),
        })
        .map(|summary| {
            serde_json::json!({
                "exportedFiles": summary.exported_files,
                "exportedSidecars": summary.exported_sidecars
            })
        })
        .map_err(Into::into)
}

#[tauri::command]
fn search_assets(input: SearchInput) -> Result<Vec<AssetView>, CommandError> {
    let service = service();
    let library = service.open_library(&input.library_path)?;
    service
        .search(
            &library.id,
            SearchQuery {
                text: input.text,
                tags: input.tags,
                min_rating: input.min_rating,
                provider: input.provider,
                status: input.status,
                category: input.category,
            },
        )
        .map(|assets| assets.into_iter().map(asset_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn gallery_items(input: GalleryItemsInput) -> Result<Vec<GalleryItemView>, CommandError> {
    let library_path = input.library_path;
    let database_path = LocalLibraryService::database_path(&library_path);
    let connection = Connection::open(&database_path).map_err(|error| CommandError {
        code: "Database".to_string(),
        message: error.to_string(),
        recoverable: true,
    })?;
    let mut statement = connection
        .prepare(
            "
            SELECT a.id, a.title, a.category, a.rating, a.status,
                   av.id, av.file_path, av.mime_type
            FROM assets a
            LEFT JOIN asset_versions av
              ON av.id = (
                SELECT latest.id
                FROM asset_versions latest
                WHERE latest.asset_id = a.id
                ORDER BY latest.created_at DESC
                LIMIT 1
              )
            ORDER BY a.created_at DESC
            ",
        )
        .map_err(|error| CommandError {
            code: "Database".to_string(),
            message: error.to_string(),
            recoverable: true,
        })?;
    let rows = statement
        .query_map([], |row| {
            let relative_path: Option<String> = row.get(6)?;
            Ok(GalleryItemView {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                rating: row.get::<_, Option<u8>>(3)?,
                status: row.get(4)?,
                version_id: row.get(5)?,
                image_path: relative_path.map(|path| library_path.join(path)),
                mime_type: row.get(7)?,
            })
        })
        .map_err(|error| CommandError {
            code: "Database".to_string(),
            message: error.to_string(),
            recoverable: true,
        })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| CommandError {
            code: "Database".to_string(),
            message: error.to_string(),
            recoverable: true,
        })
}

#[tauri::command]
fn generate_image(input: GenerateImageInput) -> Result<Vec<VersionView>, CommandError> {
    execute_generation(input, None)
}

#[tauri::command]
fn start_generation(
    input: GenerateImageInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<GenerationJobView, CommandError> {
    let job_id = generation_job_id();
    let log_path = std::env::temp_dir().join(format!("imglab-codex-cli-{job_id}.log"));
    let job = GenerationJobState {
        id: job_id.clone(),
        provider: input.provider.clone(),
        prompt: input.prompt.clone(),
        status: "running".to_string(),
        log_path: log_path.clone(),
        error: None,
        versions: Vec::new(),
    };
    let jobs = state.generation_jobs.clone();
    {
        let mut guard = jobs.lock().map_err(|_| CommandError {
            code: "ConcurrentWriteConflict".to_string(),
            message: "generation job state lock poisoned".to_string(),
            recoverable: false,
        })?;
        guard.insert(job_id.clone(), job.clone());
    }

    std::thread::spawn(move || {
        let result = execute_generation(input, Some(log_path.clone()));
        if let Ok(mut guard) = jobs.lock() {
            if let Some(job) = guard.get_mut(&job_id) {
                match result {
                    Ok(versions) => {
                        job.status = "completed".to_string();
                        job.versions = versions;
                    }
                    Err(error) => {
                        job.status = "failed".to_string();
                        job.error = Some(error.message);
                    }
                }
            }
        }
    });

    Ok(generation_job_view(job))
}

#[tauri::command]
fn get_generation_job(
    job_id: String,
    state: tauri::State<'_, DesktopState>,
) -> Result<GenerationJobView, CommandError> {
    let guard = state.generation_jobs.lock().map_err(|_| CommandError {
        code: "ConcurrentWriteConflict".to_string(),
        message: "generation job state lock poisoned".to_string(),
        recoverable: false,
    })?;
    guard
        .get(&job_id)
        .cloned()
        .map(generation_job_view)
        .ok_or_else(|| CommandError {
            code: "InvalidGenerationJob".to_string(),
            message: format!("generation job not found: {job_id}"),
            recoverable: true,
        })
}

fn execute_generation(
    input: GenerateImageInput,
    log_path: Option<PathBuf>,
) -> Result<Vec<VersionView>, CommandError> {
    let operation = if input.input_file.is_some() || input.input_version_id.is_some() {
        GenerationOperation::ImageToImage
    } else {
        GenerationOperation::TextToImage
    };
    let input_bytes = input
        .input_file
        .as_ref()
        .map(|path| {
            fs::read(path).map_err(|error| DomainError::Io {
                path: path.display().to_string(),
                message: error.to_string(),
            })
        })
        .transpose()?;
    let parameters = GenerationParameters {
        library_path: Some(input.library_path.clone()),
        provider: input.provider.clone(),
        model: "imagegen-skill".to_string(),
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        operation,
        input_version_id: input.input_version_id.map(imglab_core::AssetVersionId),
        parameters_json: input.parameters_json.unwrap_or_else(|| "{}".to_string()),
    };

    match input.provider.as_str() {
        "codex" | "codex-cli" => run_generation(
            codex_provider(&input.library_path, log_path),
            input.library_path,
            parameters,
            input_bytes,
        ),
        "fake" => run_generation(
            imglab_core::FakeImageProvider::success("fake"),
            input.library_path,
            parameters,
            input_bytes,
        ),
        provider => Err(CommandError {
            code: "InvalidGenerationParameters".to_string(),
            message: format!("unsupported provider: {provider}"),
            recoverable: true,
        }),
    }
}

fn codex_provider(library_path: &PathBuf, log_path: Option<PathBuf>) -> CodexCliImageProvider {
    let provider = CodexCliImageProvider::new("codex", library_path);
    match log_path {
        Some(path) => provider.with_log_path(path),
        None => provider,
    }
}

#[tauri::command]
fn update_asset_metadata(input: UpdateMetadataInput) -> Result<AssetView, CommandError> {
    service()
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: input.library_path,
            asset_id: AssetId(input.asset_id),
            rating: input.rating,
            category: input.category,
            status: input.status,
        })
        .map(asset_view)
        .map_err(Into::into)
}

#[tauri::command]
fn add_tag_to_asset(input: AddTagInput) -> Result<(), CommandError> {
    service()
        .add_tag_to_asset(&input.library_path, &AssetId(input.asset_id), &input.tag)
        .map_err(Into::into)
}

#[tauri::command]
fn create_manual_album(input: CreateAlbumInput) -> Result<AlbumView, CommandError> {
    let service = service();
    let library = service.open_library(&input.library_path)?;
    service
        .create_manual_album(&library.id, &input.name)
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
fn add_asset_to_album(input: AddAlbumAssetInput) -> Result<(), CommandError> {
    service()
        .add_asset(
            &imglab_core::AlbumId(input.album_id),
            &AssetId(input.asset_id),
        )
        .map_err(Into::into)
}

#[tauri::command]
fn create_suggestion(input: CreateSuggestionInput) -> Result<SuggestionView, CommandError> {
    service()
        .create_suggestion(CreateMetadataSuggestionRequest {
            library_path: input.library_path,
            asset_id: AssetId(input.asset_id),
            source: "desktop".to_string(),
            suggested_title: input.title,
            suggested_description: input.description,
            suggested_tags: input.tags,
            suggested_category: input.category,
            confidence_json: input.confidence_json.unwrap_or_else(|| "{}".to_string()),
        })
        .map(suggestion_view)
        .map_err(Into::into)
}

#[tauri::command]
fn list_pending_suggestions(library_path: PathBuf) -> Result<Vec<SuggestionView>, CommandError> {
    let service = service();
    let library = service.open_library(&library_path)?;
    service
        .list_pending(&library_path, &library.id)
        .map(|suggestions| suggestions.into_iter().map(suggestion_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn accept_suggestion(input: ReviewSuggestionInput) -> Result<AssetView, CommandError> {
    service()
        .accept(ReviewMetadataSuggestionRequest {
            library_path: input.library_path,
            suggestion_id: MetadataSuggestionId(input.suggestion_id),
            title: input.title,
            description: input.description,
            tags: input.tags,
            category: input.category,
        })
        .map(asset_view)
        .map_err(Into::into)
}

#[tauri::command]
fn reject_suggestion(library_path: PathBuf, suggestion_id: String) -> Result<(), CommandError> {
    service()
        .reject(&library_path, &MetadataSuggestionId(suggestion_id))
        .map_err(Into::into)
}

fn run_generation<P>(
    provider: P,
    library_path: PathBuf,
    parameters: GenerationParameters,
    input_bytes: Option<Vec<u8>>,
) -> Result<Vec<VersionView>, CommandError>
where
    P: ImageProvider,
{
    let library_root = library_path.clone();
    LocalGenerationService::new(provider)
        .generate(GenerateImageRequest {
            library_path,
            parameters,
            input_bytes,
        })
        .map(|versions| {
            versions
                .into_iter()
                .map(|version| version_view_with_library_path(&library_root, version))
                .collect()
        })
        .map_err(Into::into)
}

fn service() -> LocalLibraryService {
    LocalLibraryService::new(default_registry_path())
}

fn default_registry_path() -> PathBuf {
    std::env::var_os("IMGLAB_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-registry.sqlite"))
}

fn library_view(summary: imglab_core::LibrarySummary) -> LibraryView {
    LibraryView {
        id: summary.id.0,
        name: summary.name,
        root_path: summary.root_path,
        hidden: summary.hidden,
        schema_version: summary.schema_version,
    }
}

fn asset_view(summary: imglab_core::AssetSummary) -> AssetView {
    AssetView {
        id: summary.id.0,
        title: summary.title,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
    }
}

fn version_view(summary: imglab_core::VersionSummary) -> VersionView {
    VersionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        parent_version_id: summary.parent_version_id.map(|id| id.0),
        generation_event_id: summary.generation_event_id.map(|id| id.0),
        file_path: summary.file_path,
        sha256: summary.sha256,
        mime_type: summary.mime_type,
    }
}

fn version_view_with_library_path(
    library_path: &Path,
    summary: imglab_core::VersionSummary,
) -> VersionView {
    let mut view = version_view(summary);
    if view.file_path.is_relative() {
        view.file_path = library_path.join(&view.file_path);
    }
    view
}

fn album_view(summary: imglab_core::AlbumSummary) -> AlbumView {
    AlbumView {
        id: summary.id.0,
        name: summary.name,
        kind: match summary.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
    }
}

fn suggestion_view(summary: imglab_core::MetadataSuggestion) -> SuggestionView {
    SuggestionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        title: summary.suggested_title,
        description: summary.suggested_description,
        tags: summary.suggested_tags,
        category: summary.suggested_category,
        status: summary.status,
    }
}

fn generation_job_view(job: GenerationJobState) -> GenerationJobView {
    GenerationJobView {
        id: job.id,
        provider: job.provider,
        prompt: job.prompt,
        status: job.status,
        log_path: job.log_path,
        error: job.error,
        versions: job.versions,
    }
}

fn generation_job_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{millis}")
}

pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .invoke_handler(tauri::generate_handler![
            health,
            create_library,
            list_libraries,
            open_library,
            hide_library,
            import_asset,
            export_library,
            search_assets,
            gallery_items,
            generate_image,
            start_generation,
            get_generation_job,
            update_asset_metadata,
            add_tag_to_asset,
            create_manual_album,
            add_asset_to_album,
            create_suggestion,
            list_pending_suggestions,
            accept_suggestion,
            reject_suggestion
        ])
        .run(tauri::generate_context!())
        .expect("failed to run desktop application");
}
