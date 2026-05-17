use imglab_core::{
    AlbumService, AssetId, AssetService, CreateLibraryRequest, CreateMetadataSuggestionRequest,
    DomainError, ExportLibraryRequest, GalleryQuery, GalleryReadService, GallerySort,
    GenerateImageRequest, GenerationOperation, GenerationParameters, GenerationService,
    ImageProvider, ImportAssetRequest, LibraryId, LibraryService, LocalGenerationService,
    LocalLibraryService, MetadataReviewService, MetadataSuggestionId, RepairLibraryRequest,
    ReviewMetadataSuggestionRequest, ReviewStatusFilter, SearchQuery, SearchService,
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
struct LibraryStatusView {
    storage_size_bytes: u64,
    integrity_status: String,
    integrity_issue_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairIssueView {
    version_id: String,
    path: PathBuf,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairSummaryView {
    dry_run: bool,
    scanned_versions: usize,
    files_moved: usize,
    paths_updated: usize,
    checksums_updated: usize,
    dimensions_updated: usize,
    issues: Vec<RepairIssueView>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GalleryAssetView {
    id: String,
    title: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    provider: Option<String>,
    model_label: Option<String>,
    prompt: Option<String>,
    tags: Vec<String>,
    review_pending_count: u32,
    current_version_id: Option<String>,
    image_path: Option<PathBuf>,
    width: Option<u32>,
    height: Option<u32>,
    version_label: Option<String>,
    version_count: u32,
    created_at: String,
    updated_at: String,
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
    checksum_algorithm: String,
    checksum: String,
    mime_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LineageEntryView {
    version: VersionView,
    generation_event: Option<GenerationEventView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationEventView {
    id: String,
    asset_id: Option<String>,
    output_version_id: Option<String>,
    provider: String,
    provider_model: String,
    operation_type: String,
    prompt: String,
    parameters_json: String,
    status: String,
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
struct FileContextView {
    filename: String,
    relative_location: PathBuf,
    mime_type: String,
    size_bytes: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
    checksum_algorithm: String,
    checksum: String,
    integrity_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetDetailView {
    id: String,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    status: String,
    created_at: String,
    updated_at: String,
    prompt: Option<String>,
    negative_prompt: Option<String>,
    provider: Option<String>,
    model_label: Option<String>,
    parameters_json: Option<String>,
    tags: Vec<String>,
    albums: Vec<AlbumView>,
    review_pending_count: u32,
    versions: Vec<VersionView>,
    lineage: Vec<LineageEntryView>,
    file: Option<FileContextView>,
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
struct RepairLibraryInput {
    library_path: PathBuf,
    dry_run: bool,
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
struct QueryGalleryInput {
    library_path: PathBuf,
    text: Option<String>,
    providers: Option<Vec<String>>,
    min_rating: Option<u8>,
    review_status: Option<String>,
    tags: Option<Vec<String>>,
    album_id: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDetailInput {
    library_path: PathBuf,
    asset_id: String,
    current_version_id: Option<String>,
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
    title: Option<String>,
    description: Option<String>,
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
fn library_status(root_path: PathBuf) -> Result<LibraryStatusView, CommandError> {
    service()
        .library_status(&root_path)
        .map(library_status_view)
        .map_err(Into::into)
}

#[tauri::command]
fn repair_library(input: RepairLibraryInput) -> Result<RepairSummaryView, CommandError> {
    service()
        .repair_library(RepairLibraryRequest {
            library_path: input.library_path,
            dry_run: input.dry_run,
        })
        .map(repair_summary_view)
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
fn query_gallery(input: QueryGalleryInput) -> Result<Vec<GalleryAssetView>, CommandError> {
    let library_path = input.library_path.clone();
    service()
        .query_gallery(&library_path, gallery_query_from_input(input)?)
        .map(|items| {
            items
                .into_iter()
                .map(|item| gallery_asset_view(&library_path, item))
                .collect()
        })
        .map_err(Into::into)
}

#[tauri::command]
fn get_asset_detail(input: AssetDetailInput) -> Result<AssetDetailView, CommandError> {
    let current_version_id = input
        .current_version_id
        .as_ref()
        .map(|id| imglab_core::AssetVersionId(id.clone()));
    service()
        .get_asset_detail(
            &input.library_path,
            &AssetId(input.asset_id),
            current_version_id.as_ref(),
        )
        .map(asset_detail_view)
        .map_err(Into::into)
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
    let input_bytes = generation_input_bytes(&input)?;
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

fn generation_input_bytes(input: &GenerateImageInput) -> Result<Option<Vec<u8>>, DomainError> {
    if let Some(path) = input.input_file.as_ref() {
        return fs::read(path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    if let Some(version_id) = input.input_version_id.as_ref() {
        let database_path = LocalLibraryService::database_path(&input.library_path);
        let connection =
            Connection::open(&database_path).map_err(|error| DomainError::Database {
                message: error.to_string(),
            })?;
        let relative_path: String = connection
            .query_row(
                "SELECT file_path FROM asset_versions WHERE id = ?1",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|error| DomainError::Database {
                message: error.to_string(),
            })?;
        let path = input.library_path.join(relative_path);
        return fs::read(&path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    Ok(None)
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
            title: input.title,
            description: input.description,
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

fn gallery_query_from_input(input: QueryGalleryInput) -> Result<GalleryQuery, CommandError> {
    Ok(GalleryQuery {
        text: input.text,
        providers: input.providers.unwrap_or_default(),
        min_rating: input.min_rating,
        review_status: review_status_from_input(input.review_status.as_deref())?,
        tags: input.tags.unwrap_or_default(),
        album_id: input.album_id.map(imglab_core::AlbumId),
        sort: gallery_sort_from_input(input.sort.as_deref())?,
    })
}

fn review_status_from_input(value: Option<&str>) -> Result<ReviewStatusFilter, CommandError> {
    match value.unwrap_or("any") {
        "any" => Ok(ReviewStatusFilter::Any),
        "pending" | "pending_review" => Ok(ReviewStatusFilter::Pending),
        other => Err(CommandError {
            code: "InvalidGalleryQuery".to_string(),
            message: format!("unsupported review status filter: {other}"),
            recoverable: true,
        }),
    }
}

fn gallery_sort_from_input(value: Option<&str>) -> Result<GallerySort, CommandError> {
    match value.unwrap_or("newest") {
        "newest" => Ok(GallerySort::Newest),
        "oldest" => Ok(GallerySort::Oldest),
        "rating_desc" | "ratingDesc" => Ok(GallerySort::RatingDesc),
        "title_asc" | "titleAsc" => Ok(GallerySort::TitleAsc),
        "provider_asc" | "providerAsc" => Ok(GallerySort::ProviderAsc),
        other => Err(CommandError {
            code: "InvalidGalleryQuery".to_string(),
            message: format!("unsupported gallery sort: {other}"),
            recoverable: true,
        }),
    }
}

fn gallery_asset_view(
    library_path: &Path,
    summary: imglab_core::GalleryAssetView,
) -> GalleryAssetView {
    GalleryAssetView {
        id: summary.id.0,
        title: summary.title,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
        provider: summary.provider,
        model_label: summary.model_label,
        prompt: summary.prompt,
        tags: summary.tags,
        review_pending_count: summary.review_pending_count,
        current_version_id: summary.current_version_id.map(|id| id.0),
        image_path: summary
            .image_path
            .map(|path| absolutize_library_path(library_path, path)),
        width: summary.width,
        height: summary.height,
        version_label: summary.version_label,
        version_count: summary.version_count,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
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
        checksum_algorithm: summary.checksum_algorithm,
        checksum: summary.checksum,
        mime_type: summary.mime_type,
    }
}

fn generation_event_view(summary: imglab_core::GenerationEventSummary) -> GenerationEventView {
    GenerationEventView {
        id: summary.id.0,
        asset_id: summary.asset_id.map(|id| id.0),
        output_version_id: summary.output_version_id.map(|id| id.0),
        provider: summary.provider,
        provider_model: summary.provider_model,
        operation_type: match summary.operation_type {
            GenerationOperation::TextToImage => "text_to_image",
            GenerationOperation::ImageToImage => "image_to_image",
        }
        .to_string(),
        prompt: summary.prompt,
        parameters_json: summary.parameters_json,
        status: summary.status,
    }
}

fn asset_detail_view(summary: imglab_core::AssetDetailView) -> AssetDetailView {
    AssetDetailView {
        id: summary.id.0,
        title: summary.title,
        description: summary.description,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
        prompt: summary.prompt,
        negative_prompt: summary.negative_prompt,
        provider: summary.provider,
        model_label: summary.model_label,
        parameters_json: summary.parameters_json,
        tags: summary.tags,
        albums: summary
            .albums
            .into_iter()
            .map(|album| AlbumView {
                id: album.id.0,
                name: album.name,
                kind: match album.kind {
                    imglab_core::AlbumKind::Manual => "manual",
                    imglab_core::AlbumKind::Smart => "smart",
                }
                .to_string(),
            })
            .collect(),
        review_pending_count: summary.review_pending_count,
        versions: summary.versions.into_iter().map(version_view).collect(),
        lineage: summary
            .lineage
            .into_iter()
            .map(|entry| LineageEntryView {
                version: version_view(entry.version),
                generation_event: entry.generation_event.map(generation_event_view),
            })
            .collect(),
        file: summary.file.map(|file| FileContextView {
            filename: file.filename,
            relative_location: file.relative_location,
            mime_type: file.mime_type,
            size_bytes: file.size_bytes,
            width: file.width,
            height: file.height,
            checksum_algorithm: file.checksum_algorithm,
            checksum: file.checksum,
            integrity_status: file.integrity_status,
        }),
    }
}

fn library_status_view(summary: imglab_core::LibraryStatusView) -> LibraryStatusView {
    LibraryStatusView {
        storage_size_bytes: summary.storage_size_bytes,
        integrity_status: summary.integrity_status,
        integrity_issue_count: summary.integrity_issue_count,
    }
}

fn repair_summary_view(summary: imglab_core::RepairSummary) -> RepairSummaryView {
    RepairSummaryView {
        dry_run: summary.dry_run,
        scanned_versions: summary.scanned_versions,
        files_moved: summary.files_moved,
        paths_updated: summary.paths_updated,
        checksums_updated: summary.checksums_updated,
        dimensions_updated: summary.dimensions_updated,
        issues: summary
            .issues
            .into_iter()
            .map(|issue| RepairIssueView {
                version_id: issue.version_id.0,
                path: issue.path,
                message: issue.message,
            })
            .collect(),
    }
}

fn absolutize_library_path(library_path: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        library_path.join(path)
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
            library_status,
            repair_library,
            hide_library,
            import_asset,
            export_library,
            search_assets,
            gallery_items,
            query_gallery,
            get_asset_detail,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_gallery_sort_input() {
        assert!(matches!(
            gallery_sort_from_input(Some("ratingDesc")).expect("sort"),
            GallerySort::RatingDesc
        ));
        let error = gallery_sort_from_input(Some("unknown")).expect_err("invalid sort");
        assert_eq!(error.code, "InvalidGalleryQuery");
        assert!(error.recoverable);
    }

    #[test]
    fn maps_provider_capability_error_as_recoverable() {
        let error: CommandError = DomainError::UnsupportedProviderCapability {
            provider: "codex-cli".to_string(),
            capability: "image_to_image".to_string(),
        }
        .into();

        assert_eq!(error.code, "UnsupportedProviderCapability");
        assert!(error.recoverable);
    }
}
