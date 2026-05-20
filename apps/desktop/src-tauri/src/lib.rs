mod app_logs;
mod daemon_client;
mod metadata_generation;

use app_logs::{AppLogContentView, AppLogView, ReadAppLogInput};
use daemon_client::{
    BatchCreateTasksInput, DaemonSidecar, DaemonTask, DaemonTaskAttempt, DaemonTaskDetail,
    DaemonTaskEvent, DaemonTaskInput, DaemonTaskOutput,
};
use imglab_core::{
    prepare_generation_request, AlbumId, AlbumService, AssetId, AssetService,
    BatchAddAssetsToAlbumRequest, BatchReviewMetadataSuggestionRequest, CreateLibraryRequest,
    CreateMetadataSuggestionRequest, CreateSmartAlbumRequest, DomainError,
    ExportLibraryBackupRequest, ExportLibraryRequest, GalleryQuery, GalleryReadService,
    GallerySort, GenerateImageRequest, GenerationOperation, GenerationRequestInput,
    GenerationService, ImageProvider, ImportAssetRequest, ImportLibraryBackupRequest, LibraryId,
    LibraryService, LocalGenerationService, LocalLibraryService, MetadataReviewService,
    MetadataSuggestionId, RenameLibraryAliasRequest, ReorderAlbumItemsRequest,
    ReorderAlbumsRequest, RepairLibraryRequest, ReviewMetadataSuggestionRequest,
    ReviewStatusFilter, SearchQuery, SearchService, UpdateAssetMetadataRequest,
};
use imglab_provider_codex::CodexCliImageProvider;
use metadata_generation::{
    CodexCliMetadataGenerator, GenerateReviewFieldInput, GeneratedReviewFieldView, ReviewField,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Default)]
struct DesktopState {
    daemon_sidecar: Arc<Mutex<Option<DaemonSidecar>>>,
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

fn updater_error(error: impl std::fmt::Display) -> CommandError {
    CommandError {
        code: "UpdaterError".to_string(),
        message: error.to_string(),
        recoverable: true,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfoView {
    version: String,
    date: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckView {
    current_version: String,
    available: bool,
    update: Option<UpdateInfoView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInstallView {
    installed: bool,
    version: Option<String>,
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
struct LibraryBackupView {
    library: LibraryView,
    cloned: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StudioOverviewView {
    library: LibraryView,
    status: LibraryStatusView,
    registered_library_count: u32,
    missing_library_count: u32,
    review_pending_count: u32,
    task_summary: StudioTaskSummaryView,
    provider_health: Vec<ProviderHealthSummaryView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StudioTaskSummaryView {
    active_count: u32,
    queued_count: u32,
    running_count: u32,
    retry_waiting_count: u32,
    failed_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderHealthSummaryView {
    provider: String,
    display_name: String,
    availability: String,
    credential_state: String,
    supported_operations: Vec<String>,
    recoverable_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticsOverviewView {
    provider_health: Vec<ProviderHealthSummaryView>,
    daemon_status: DaemonStatusView,
    library_status: LibraryStatusView,
    library_count: u32,
    missing_library_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonStatusView {
    state: String,
    recoverable_error: Option<String>,
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
    current_version_number: Option<u32>,
    current_version_name: Option<String>,
    image_path: Option<PathBuf>,
    width: Option<u32>,
    height: Option<u32>,
    version_label: Option<String>,
    version_count: u32,
    task_origin: Option<TaskOriginView>,
    albums: Vec<AlbumView>,
    album_context: Option<AlbumView>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskOriginView {
    task_id: String,
    task_type: String,
    status: String,
    provider: Option<String>,
    operation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionView {
    id: String,
    asset_id: String,
    parent_version_id: Option<String>,
    generation_event_id: Option<String>,
    version_number: u32,
    version_name: String,
    file_path: PathBuf,
    checksum_algorithm: String,
    checksum: String,
    mime_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceSourceView {
    asset_id: String,
    asset_title: Option<String>,
    asset_status: String,
    version_id: String,
    version_number: u32,
    version_name: String,
    file_path: PathBuf,
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
struct AlbumListItemView {
    id: String,
    name: String,
    kind: String,
    item_count: u32,
    sort_order: i64,
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
    schema_prompt: Option<String>,
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
    current_version_id: Option<String>,
    current_version_number: Option<u32>,
    current_version_name: Option<String>,
    versions: Vec<VersionView>,
    lineage: Vec<LineageEntryView>,
    source_reference: Option<ReferenceSourceView>,
    file: Option<FileContextView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetInspectorDetailView {
    asset: AssetDetailView,
    canonical_metadata: CanonicalMetadataView,
    pending_suggestions: Vec<PendingSuggestionSummaryView>,
    generated_task_origin: Option<TaskOriginView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalMetadataView {
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    category: Option<String>,
    rating: Option<u8>,
    tags: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingSuggestionSummaryView {
    id: String,
    asset_id: String,
    title: Option<String>,
    category: Option<String>,
    tag_count: u32,
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuggestionView {
    id: String,
    asset_id: String,
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
    status: String,
    confidence_json: String,
    created_at: Option<String>,
    reviewed_at: Option<String>,
    confidence: ConfidenceScoreView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfidenceScoreView {
    overall: Option<u8>,
    title: Option<u8>,
    description: Option<u8>,
    schema_prompt: Option<u8>,
    tags: Option<u8>,
    category: Option<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewDraftDetailView {
    suggestion: SuggestionView,
    draft_seed: ReviewDraftSeedView,
    confidence: ConfidenceScoreView,
    history: Vec<SuggestionView>,
    generated_field_results: Vec<GeneratedReviewFieldResultView>,
    related_tasks: Vec<RelatedTaskSummaryView>,
    asset: AssetDetailView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewDraftSeedView {
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedReviewFieldResultView {
    task_id: String,
    field: String,
    value: String,
    base_revision: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelatedTaskSummaryView {
    id: String,
    task_type: String,
    status: String,
    provider: Option<String>,
    operation: Option<String>,
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
struct RenameLibraryAliasInput {
    library_id: String,
    alias: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportLibraryBackupInput {
    library_path: PathBuf,
    output_zip_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportLibraryBackupInput {
    zip_path: PathBuf,
    destination_path: PathBuf,
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
struct DaemonTaskQueryInput {
    library_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnqueueGenerationTasksInput {
    library_path: PathBuf,
    tasks: Vec<GenerationTaskDraftInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderDaemonTasksInput {
    library_path: PathBuf,
    task_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskActionInput {
    task_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationTaskDraftInput {
    task_type: Option<String>,
    provider: String,
    prompt: String,
    negative_prompt: Option<String>,
    operation: Option<String>,
    input_file: Option<PathBuf>,
    input_version_id: Option<String>,
    parameters_json: Option<String>,
    priority: Option<i64>,
    max_attempts: Option<u32>,
    input: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskView {
    id: String,
    library_id: String,
    task_type: String,
    status: String,
    queue_position: i64,
    priority: i64,
    provider: Option<String>,
    operation: Option<String>,
    concurrency_group: Option<String>,
    attempt_count: u32,
    max_attempts: u32,
    next_retry_at: Option<String>,
    input: serde_json::Value,
    created_at: String,
    updated_at: String,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
    error_classification: Option<String>,
    wait_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskAttemptView {
    id: String,
    task_id: String,
    attempt_number: u32,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    log_path: Option<PathBuf>,
    exit_code: Option<i32>,
    error_code: Option<String>,
    error_message: Option<String>,
    error_classification: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskEventView {
    id: String,
    task_id: String,
    event_type: String,
    message: Option<String>,
    payload: Option<serde_json::Value>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskOutputView {
    id: String,
    task_id: String,
    output_type: String,
    target_id: String,
    payload: Option<serde_json::Value>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonTaskDetailView {
    task: DaemonTaskView,
    attempts: Vec<DaemonTaskAttemptView>,
    events: Vec<DaemonTaskEventView>,
    outputs: Vec<DaemonTaskOutputView>,
    log_tail: String,
    log_tail_truncated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMetadataInput {
    library_path: PathBuf,
    asset_id: String,
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
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
struct BatchAddAlbumAssetsInput {
    album_id: String,
    asset_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameAlbumInput {
    album_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveAlbumAssetInput {
    album_id: String,
    asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderAlbumsInput {
    library_path: PathBuf,
    album_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderAlbumItemsInput {
    album_id: String,
    asset_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSmartAlbumInput {
    library_path: PathBuf,
    name: String,
    smart_query_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSuggestionInput {
    library_path: PathBuf,
    asset_id: String,
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
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
    schema_prompt: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchReviewSuggestionsInput {
    library_path: PathBuf,
    suggestions: Vec<ReviewSuggestionInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RejectSuggestionInput {
    library_path: PathBuf,
    suggestion_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchRejectSuggestionsInput {
    library_path: PathBuf,
    suggestion_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuggestionHistoryInput {
    library_path: PathBuf,
    asset_id: String,
}

#[tauri::command]
fn health() -> &'static str {
    "ok"
}

#[tauri::command]
fn create_library(input: CreateLibraryInput) -> Result<LibraryView, CommandError> {
    let root_path = normalize_library_root_path(input.root_path)?;
    service()
        .create_library(CreateLibraryRequest {
            root_path,
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
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .open_library(&root_path)
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn library_status(root_path: PathBuf) -> Result<LibraryStatusView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .library_status(&root_path)
        .map(library_status_view)
        .map_err(Into::into)
}

#[tauri::command]
fn studio_overview(root_path: PathBuf) -> Result<StudioOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .studio_overview(&root_path)
        .map(studio_overview_view)
        .map_err(Into::into)
}

#[tauri::command]
fn diagnostics_overview(root_path: PathBuf) -> Result<DiagnosticsOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .diagnostics_overview(&root_path)
        .map(diagnostics_overview_view)
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
fn rename_library_alias(input: RenameLibraryAliasInput) -> Result<LibraryView, CommandError> {
    service()
        .rename_library_alias(RenameLibraryAliasRequest {
            library_id: LibraryId(input.library_id),
            alias: input.alias,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn unregister_library(library_id: String) -> Result<(), CommandError> {
    service()
        .unregister_library(&LibraryId(library_id))
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
fn export_library_backup_zip(input: ExportLibraryBackupInput) -> Result<(), CommandError> {
    let library_path = normalize_library_root_path(input.library_path)?;
    service()
        .export_library_backup_zip(ExportLibraryBackupRequest {
            library_path,
            output_zip_path: input.output_zip_path,
        })
        .map_err(Into::into)
}

#[tauri::command]
fn import_library_backup_zip(
    input: ImportLibraryBackupInput,
) -> Result<LibraryBackupView, CommandError> {
    let destination_path = normalize_library_root_path(input.destination_path)?;
    service()
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path: input.zip_path,
            destination_path,
        })
        .map(|summary| LibraryBackupView {
            library: library_view(summary.library),
            cloned: summary.cloned,
        })
        .map_err(Into::into)
}

#[tauri::command]
fn reveal_library_folder(root_path: PathBuf) -> Result<(), CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    if !root_path.is_dir() {
        return Err(CommandError {
            code: "LibraryNotFound".to_string(),
            message: format!("library folder is missing: {}", root_path.display()),
            recoverable: true,
        });
    }
    reveal_path(&root_path)
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
        .map(|detail| asset_detail_view(detail, &input.library_path))
        .map_err(Into::into)
}

#[tauri::command]
fn get_asset_inspector_detail(
    input: AssetDetailInput,
) -> Result<AssetInspectorDetailView, CommandError> {
    let current_version_id = input
        .current_version_id
        .as_ref()
        .map(|id| imglab_core::AssetVersionId(id.clone()));
    service()
        .get_asset_inspector_detail(
            &input.library_path,
            &AssetId(input.asset_id),
            current_version_id.as_ref(),
        )
        .map(|detail| asset_inspector_detail_view(detail, &input.library_path))
        .map_err(Into::into)
}

#[tauri::command]
async fn generate_image(input: GenerateImageInput) -> Result<Vec<VersionView>, CommandError> {
    tauri::async_runtime::spawn_blocking(move || execute_generation(input, None))
        .await
        .map_err(|error| CommandError {
            code: "GenerationFailed".to_string(),
            message: format!("generation worker failed: {error}"),
            recoverable: true,
        })?
}

#[tauri::command]
fn daemon_health(state: tauri::State<'_, DesktopState>) -> Result<bool, CommandError> {
    ensure_daemon_client(&state).map(|_| true)
}

#[tauri::command]
fn enqueue_generation_tasks(
    input: EnqueueGenerationTasksInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonTaskView>, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    let tasks = input
        .tasks
        .into_iter()
        .map(generation_draft_to_daemon_task)
        .collect::<Result<Vec<_>, _>>()?;
    client
        .batch_create_tasks(BatchCreateTasksInput { library_id, tasks })
        .map(|tasks| tasks.into_iter().map(daemon_task_view).collect())
}

#[tauri::command]
fn list_daemon_tasks(
    input: DaemonTaskQueryInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonTaskView>, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client
        .list_tasks(&library_id)
        .map(|tasks| tasks.into_iter().map(daemon_task_view).collect())
}

#[tauri::command]
fn get_daemon_task_detail(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskDetailView, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let detail = client.get_task(&input.task_id)?;
    let tail = client.tail_task_log(&input.task_id)?;
    Ok(daemon_task_detail_view(
        detail,
        tail.content,
        tail.truncated,
    ))
}

#[tauri::command]
fn reorder_daemon_tasks(
    input: ReorderDaemonTasksInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<(), CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client.reorder_tasks(library_id, input.task_ids)
}

#[tauri::command]
fn cancel_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .cancel_task(&input.task_id)
        .map(daemon_task_view)
}

#[tauri::command]
fn retry_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .retry_task(&input.task_id)
        .map(daemon_task_view)
}

#[tauri::command]
fn duplicate_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .duplicate_task(&input.task_id)
        .map(daemon_task_view)
}

fn generation_draft_to_daemon_task(
    input: GenerationTaskDraftInput,
) -> Result<DaemonTaskInput, CommandError> {
    let parameters = input
        .parameters_json
        .as_deref()
        .map(serde_json::from_str::<serde_json::Value>)
        .transpose()
        .map_err(|error| CommandError {
            code: "InvalidGenerationParameters".to_string(),
            message: format!("invalid generation parameters JSON: {error}"),
            recoverable: true,
        })?;
    Ok(DaemonTaskInput {
        task_type: input
            .task_type
            .unwrap_or_else(|| "image_generation".to_string()),
        provider: Some(input.provider),
        operation: Some(
            input
                .operation
                .unwrap_or_else(|| "text_to_image".to_string()),
        ),
        priority: input.priority,
        concurrency_group: None,
        max_attempts: input.max_attempts,
        input: input.input.unwrap_or_else(|| {
            serde_json::json!({
                "prompt": input.prompt,
                "negativePrompt": input.negative_prompt,
                "inputFile": input.input_file,
                "inputVersionId": input.input_version_id,
                "parametersJson": parameters,
            })
        }),
    })
}

fn execute_generation(
    input: GenerateImageInput,
    log_path: Option<PathBuf>,
) -> Result<Vec<VersionView>, CommandError> {
    let prepared = prepare_generation_request(GenerationRequestInput {
        library_path: input.library_path,
        provider: input.provider,
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        input_file: input.input_file,
        input_version_id: input.input_version_id.map(imglab_core::AssetVersionId),
        parameters_json: input.parameters_json,
    })?;

    match prepared.provider.as_str() {
        "codex" | "codex-cli" => run_generation(
            codex_provider(&prepared.request.library_path, log_path),
            prepared.request,
        ),
        "fake" => run_generation(
            imglab_core::FakeImageProvider::success("fake"),
            prepared.request,
        ),
        _ => unreachable!("provider is normalized before dispatch"),
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
            title: input.title,
            description: input.description,
            schema_prompt: input.schema_prompt,
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
fn list_albums(library_path: PathBuf) -> Result<Vec<AlbumListItemView>, CommandError> {
    service()
        .list_albums_in_library(&library_path)
        .map(|albums| albums.into_iter().map(album_list_item_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn create_manual_album(input: CreateAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .create_manual_album_in_library(&input.library_path, &input.name)
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
fn create_smart_album(input: CreateSmartAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .create_smart_album(CreateSmartAlbumRequest {
            library_path: input.library_path,
            name: input.name,
            smart_query_json: input.smart_query_json,
        })
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
fn batch_add_assets_to_album(input: BatchAddAlbumAssetsInput) -> Result<(), CommandError> {
    service()
        .batch_add_assets(BatchAddAssetsToAlbumRequest {
            album_id: AlbumId(input.album_id),
            asset_ids: input.asset_ids.into_iter().map(AssetId).collect(),
        })
        .map_err(Into::into)
}

#[tauri::command]
fn remove_asset_from_album(input: RemoveAlbumAssetInput) -> Result<(), CommandError> {
    service()
        .remove_asset(&AlbumId(input.album_id), &AssetId(input.asset_id))
        .map_err(Into::into)
}

#[tauri::command]
fn rename_album(input: RenameAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .rename_album(&AlbumId(input.album_id), &input.name)
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
fn delete_album(album_id: String) -> Result<(), CommandError> {
    service()
        .delete_album(&AlbumId(album_id))
        .map_err(Into::into)
}

#[tauri::command]
fn reorder_albums(input: ReorderAlbumsInput) -> Result<(), CommandError> {
    service()
        .reorder_albums(ReorderAlbumsRequest {
            library_path: input.library_path,
            album_ids: input.album_ids.into_iter().map(AlbumId).collect(),
        })
        .map_err(Into::into)
}

#[tauri::command]
fn reorder_album_items(input: ReorderAlbumItemsInput) -> Result<(), CommandError> {
    service()
        .reorder_album_items(ReorderAlbumItemsRequest {
            album_id: AlbumId(input.album_id),
            asset_ids: input.asset_ids.into_iter().map(AssetId).collect(),
        })
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
            suggested_schema_prompt: input.schema_prompt,
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
fn get_review_draft_detail(
    library_path: PathBuf,
    suggestion_id: String,
) -> Result<ReviewDraftDetailView, CommandError> {
    service()
        .get_review_draft_detail(&library_path, &MetadataSuggestionId(suggestion_id))
        .map(|detail| review_draft_detail_view(detail, &library_path))
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
            schema_prompt: input.schema_prompt,
            tags: input.tags,
            category: input.category,
        })
        .map(asset_view)
        .map_err(Into::into)
}

#[tauri::command]
fn batch_accept_suggestions(
    input: BatchReviewSuggestionsInput,
) -> Result<Vec<AssetView>, CommandError> {
    let library_path = input.library_path;
    service()
        .batch_accept(BatchReviewMetadataSuggestionRequest {
            library_path: library_path.clone(),
            suggestions: input
                .suggestions
                .into_iter()
                .map(|suggestion| ReviewMetadataSuggestionRequest {
                    library_path: library_path.clone(),
                    suggestion_id: MetadataSuggestionId(suggestion.suggestion_id),
                    title: suggestion.title,
                    description: suggestion.description,
                    schema_prompt: suggestion.schema_prompt,
                    tags: suggestion.tags,
                    category: suggestion.category,
                })
                .collect(),
        })
        .map(|assets| assets.into_iter().map(asset_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn reject_suggestion(input: RejectSuggestionInput) -> Result<(), CommandError> {
    service()
        .reject(
            &input.library_path,
            &MetadataSuggestionId(input.suggestion_id),
        )
        .map_err(Into::into)
}

#[tauri::command]
fn batch_reject_suggestions(input: BatchRejectSuggestionsInput) -> Result<(), CommandError> {
    let suggestion_ids = input
        .suggestion_ids
        .into_iter()
        .map(MetadataSuggestionId)
        .collect::<Vec<_>>();
    service()
        .batch_reject(&input.library_path, &suggestion_ids)
        .map_err(Into::into)
}

#[tauri::command]
fn list_suggestion_history(
    input: SuggestionHistoryInput,
) -> Result<Vec<SuggestionView>, CommandError> {
    service()
        .list_history(&input.library_path, &AssetId(input.asset_id))
        .map(|suggestions| suggestions.into_iter().map(suggestion_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
async fn regenerate_suggestion(
    input: GenerateReviewFieldInput,
) -> Result<SuggestionView, CommandError> {
    let library_path = input.library_path.clone();
    let asset_id = input.asset_id.clone();
    let context = input.context.clone();
    let title_input = GenerateReviewFieldInput {
        field: ReviewField::Title,
        context: context.clone(),
        ..input.clone()
    };
    let description_input = GenerateReviewFieldInput {
        field: ReviewField::Description,
        context: context.clone(),
        ..input.clone()
    };
    let schema_input = GenerateReviewFieldInput {
        field: ReviewField::SchemaPrompt,
        context,
        ..input
    };
    tauri::async_runtime::spawn_blocking(move || {
        let generator = CodexCliMetadataGenerator::new("codex", &library_path);
        let title = generator.generate(&title_input)?.value;
        let description = generator.generate(&description_input)?.value;
        let schema_prompt = generator.generate(&schema_input)?.value;
        service()
            .create_suggestion(CreateMetadataSuggestionRequest {
                library_path,
                asset_id: AssetId(asset_id),
                source: "codex_metadata".to_string(),
                suggested_title: Some(title),
                suggested_description: Some(description),
                suggested_schema_prompt: Some(schema_prompt),
                suggested_tags: title_input.context.tags,
                suggested_category: title_input.context.category,
                confidence_json: "{}".to_string(),
            })
            .map(suggestion_view)
            .map_err(Into::into)
    })
    .await
    .map_err(|error| CommandError {
        code: "MetadataGenerationFailed".to_string(),
        message: format!("metadata generation worker failed: {error}"),
        recoverable: true,
    })?
}

#[tauri::command]
async fn generate_review_field(
    input: GenerateReviewFieldInput,
) -> Result<GeneratedReviewFieldView, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        CodexCliMetadataGenerator::new("codex", &input.library_path).generate(&input)
    })
    .await
    .map_err(|error| CommandError {
        code: "MetadataGenerationFailed".to_string(),
        message: format!("metadata generation worker failed: {error}"),
        recoverable: true,
    })?
}

#[tauri::command]
fn list_app_logs() -> Result<Vec<AppLogView>, CommandError> {
    app_logs::list_app_logs()
}

#[tauri::command]
fn read_app_log(input: ReadAppLogInput) -> Result<AppLogContentView, CommandError> {
    app_logs::read_app_log(&input.path)
}

fn run_generation<P>(
    provider: P,
    request: GenerateImageRequest,
) -> Result<Vec<VersionView>, CommandError>
where
    P: ImageProvider,
{
    let library_root = request.library_path.clone();
    LocalGenerationService::new(provider)
        .generate(request)
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

fn ensure_daemon_client(
    state: &tauri::State<'_, DesktopState>,
) -> Result<daemon_client::DaemonClient, CommandError> {
    let runtime_dir = daemon_runtime_dir();
    let runtime_path = runtime_dir.join("runtime.json");
    {
        let mut guard = state.daemon_sidecar.lock().map_err(|_| CommandError {
            code: "ConcurrentWriteConflict".to_string(),
            message: "daemon sidecar state lock poisoned".to_string(),
            recoverable: false,
        })?;
        if let Some(sidecar) = guard.as_ref() {
            if sidecar.client.health().is_ok() {
                return Ok(sidecar.client.clone());
            }
        }
        if let Some(mut sidecar) = guard.take() {
            let _ = sidecar.child.kill();
            let _ = sidecar.child.wait();
        }
    }

    if !should_start_managed_daemon() {
        match daemon_client::discover_daemon(&runtime_path) {
            Ok(Some(client)) => return Ok(client),
            Ok(None) => {}
            Err(error) if error.recoverable => {}
            Err(error) => return Err(error),
        }
    }

    let daemon_bin = daemon_binary_path()?;
    let sidecar = daemon_client::start_daemon_sidecar(&daemon_bin, &runtime_dir)?;
    let client = sidecar.client.clone();
    let mut guard = state.daemon_sidecar.lock().map_err(|_| CommandError {
        code: "ConcurrentWriteConflict".to_string(),
        message: "daemon sidecar state lock poisoned".to_string(),
        recoverable: false,
    })?;
    *guard = Some(sidecar);
    Ok(client)
}

fn daemon_runtime_dir() -> PathBuf {
    std::env::var_os("IMGLAB_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-daemon"))
}

fn daemon_binary_path() -> Result<PathBuf, CommandError> {
    if let Some(path) = std::env::var_os("IMGLAB_DAEMON_BIN").map(PathBuf::from) {
        return Ok(path);
    }
    if let Some(path) = workspace_debug_daemon_binary() {
        return Ok(path);
    }
    let exe = std::env::current_exe().map_err(|error| CommandError {
        code: "DaemonStartFailed".to_string(),
        message: format!("failed to locate current executable: {error}"),
        recoverable: true,
    })?;
    let Some(dir) = exe.parent() else {
        return Err(CommandError {
            code: "DaemonStartFailed".to_string(),
            message: "failed to resolve daemon binary directory".to_string(),
            recoverable: true,
        });
    };
    Ok(dir.join("imglab-daemon"))
}

fn should_start_managed_daemon() -> bool {
    std::env::var_os("IMGLAB_DAEMON_REUSE_RUNTIME").is_none()
        && (std::env::var_os("IMGLAB_DAEMON_BIN").is_some()
            || workspace_debug_daemon_binary().is_some())
}

fn workspace_debug_daemon_binary() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent()?.parent()?.parent()?;
    let binary_name = if cfg!(target_os = "windows") {
        "imglab-daemon.exe"
    } else {
        "imglab-daemon"
    };
    let path = workspace_root
        .join("target")
        .join("debug")
        .join(binary_name);
    path.exists().then_some(path)
}

fn normalize_library_root_path(path: PathBuf) -> Result<PathBuf, CommandError> {
    let path = expand_home_path(path)?;
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(invalid_path_error(
            "library path must be absolute or start with ~/".to_string(),
        ))
    }
}

fn expand_home_path(path: PathBuf) -> Result<PathBuf, CommandError> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return home_dir();
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        return home_dir().map(|home| home.join(rest));
    }

    if raw.starts_with('~') {
        return Err(invalid_path_error(
            "library path only supports ~ for the current user".to_string(),
        ));
    }

    Ok(path)
}

fn home_dir() -> Result<PathBuf, CommandError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| invalid_path_error("HOME is not set to an absolute path".to_string()))
}

fn invalid_path_error(message: String) -> CommandError {
    CommandError {
        code: "InvalidPath".to_string(),
        message,
        recoverable: true,
    }
}

fn reveal_path(path: &Path) -> Result<(), CommandError> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).status()
    } else {
        Command::new("xdg-open").arg(path).status()
    }
    .map_err(|error| CommandError {
        code: "RevealFailed".to_string(),
        message: format!("failed to open folder: {error}"),
        recoverable: true,
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(CommandError {
            code: "RevealFailed".to_string(),
            message: format!("open folder command exited with status: {status}"),
            recoverable: true,
        })
    }
}

fn library_view(summary: imglab_core::LibrarySummary) -> LibraryView {
    let root_path = expand_home_path(summary.root_path.clone()).unwrap_or(summary.root_path);
    LibraryView {
        id: summary.id.0,
        name: summary.name,
        root_path,
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
        "album_order" | "albumOrder" => Ok(GallerySort::AlbumOrder),
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
        current_version_number: summary.current_version_number,
        current_version_name: summary.current_version_name,
        image_path: summary
            .image_path
            .map(|path| absolutize_library_path(library_path, path)),
        width: summary.width,
        height: summary.height,
        version_label: summary.version_label,
        version_count: summary.version_count,
        task_origin: summary.task_origin.map(task_origin_view),
        albums: summary
            .albums
            .into_iter()
            .map(album_membership_view)
            .collect(),
        album_context: summary.album_context.map(album_membership_view),
        created_at: summary.created_at,
        updated_at: summary.updated_at,
    }
}

fn task_origin_view(origin: imglab_core::TaskOriginView) -> TaskOriginView {
    TaskOriginView {
        task_id: origin.task_id.0,
        task_type: task_type_value(origin.task_type),
        status: task_status_value(origin.status),
        provider: origin.provider,
        operation: origin.operation.map(operation_value),
    }
}

fn task_type_value(task_type: imglab_core::TaskType) -> String {
    task_type.as_str().to_string()
}

fn task_status_value(status: imglab_core::TaskStatus) -> String {
    status.as_str().to_string()
}

fn operation_value(operation: GenerationOperation) -> String {
    match operation {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
    .to_string()
}

fn version_view(summary: imglab_core::VersionSummary) -> VersionView {
    VersionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        parent_version_id: summary.parent_version_id.map(|id| id.0),
        generation_event_id: summary.generation_event_id.map(|id| id.0),
        version_number: summary.version_number,
        version_name: summary.version_name,
        file_path: summary.file_path,
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
        operation_type: operation_value(summary.operation_type),
        prompt: summary.prompt,
        parameters_json: summary.parameters_json,
        status: summary.status,
    }
}

fn studio_overview_view(summary: imglab_core::StudioOverviewView) -> StudioOverviewView {
    StudioOverviewView {
        library: library_view(summary.library),
        status: library_status_view(summary.status),
        registered_library_count: summary.registered_library_count,
        missing_library_count: summary.missing_library_count,
        review_pending_count: summary.review_pending_count,
        task_summary: StudioTaskSummaryView {
            active_count: summary.task_summary.active_count,
            queued_count: summary.task_summary.queued_count,
            running_count: summary.task_summary.running_count,
            retry_waiting_count: summary.task_summary.retry_waiting_count,
            failed_count: summary.task_summary.failed_count,
        },
        provider_health: summary
            .provider_health
            .into_iter()
            .map(provider_health_summary_view)
            .collect(),
    }
}

fn provider_health_summary_view(
    summary: imglab_core::ProviderHealthSummaryView,
) -> ProviderHealthSummaryView {
    ProviderHealthSummaryView {
        provider: summary.provider,
        display_name: summary.display_name,
        availability: summary.availability,
        credential_state: summary.credential_state,
        supported_operations: summary
            .supported_operations
            .into_iter()
            .map(operation_value)
            .collect(),
        recoverable_error: summary.recoverable_error,
    }
}

fn diagnostics_overview_view(
    summary: imglab_core::DiagnosticsOverviewView,
) -> DiagnosticsOverviewView {
    DiagnosticsOverviewView {
        provider_health: summary
            .provider_health
            .into_iter()
            .map(provider_health_summary_view)
            .collect(),
        daemon_status: DaemonStatusView {
            state: summary.daemon_status.state,
            recoverable_error: summary.daemon_status.recoverable_error,
        },
        library_status: library_status_view(summary.library_status),
        library_count: summary.library_count,
        missing_library_count: summary.missing_library_count,
    }
}

fn album_membership_view(album: imglab_core::AlbumMembershipView) -> AlbumView {
    AlbumView {
        id: album.id.0,
        name: album.name,
        kind: match album.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
    }
}

fn asset_detail_view(
    summary: imglab_core::AssetDetailView,
    library_path: &Path,
) -> AssetDetailView {
    AssetDetailView {
        id: summary.id.0,
        title: summary.title,
        description: summary.description,
        schema_prompt: summary.schema_prompt,
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
        current_version_id: summary.current_version_id.map(|id| id.0),
        current_version_number: summary.current_version_number,
        current_version_name: summary.current_version_name,
        versions: summary
            .versions
            .into_iter()
            .map(|version| version_view_with_library_path(library_path, version))
            .collect(),
        lineage: summary
            .lineage
            .into_iter()
            .map(|entry| LineageEntryView {
                version: version_view_with_library_path(library_path, entry.version),
                generation_event: entry.generation_event.map(generation_event_view),
            })
            .collect(),
        source_reference: summary
            .source_reference
            .map(|source| reference_source_view(source, library_path)),
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

fn reference_source_view(
    summary: imglab_core::ReferenceSourceView,
    library_path: &Path,
) -> ReferenceSourceView {
    ReferenceSourceView {
        asset_id: summary.asset_id.0,
        asset_title: summary.asset_title,
        asset_status: summary.asset_status,
        version_id: summary.version_id.0,
        version_number: summary.version_number,
        version_name: summary.version_name,
        file_path: absolutize_library_path(library_path, summary.file_path),
    }
}

fn asset_inspector_detail_view(
    summary: imglab_core::AssetInspectorDetailView,
    library_path: &Path,
) -> AssetInspectorDetailView {
    AssetInspectorDetailView {
        asset: asset_detail_view(summary.asset, library_path),
        canonical_metadata: CanonicalMetadataView {
            title: summary.canonical_metadata.title,
            description: summary.canonical_metadata.description,
            schema_prompt: summary.canonical_metadata.schema_prompt,
            category: summary.canonical_metadata.category,
            rating: summary.canonical_metadata.rating,
            tags: summary.canonical_metadata.tags,
            status: summary.canonical_metadata.status,
        },
        pending_suggestions: summary
            .pending_suggestions
            .into_iter()
            .map(|suggestion| PendingSuggestionSummaryView {
                id: suggestion.id.0,
                asset_id: suggestion.asset_id.0,
                title: suggestion.title,
                category: suggestion.category,
                tag_count: suggestion.tag_count,
                created_at: suggestion.created_at,
            })
            .collect(),
        generated_task_origin: summary.generated_task_origin.map(task_origin_view),
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

fn album_list_item_view(item: imglab_core::AlbumListItem) -> AlbumListItemView {
    AlbumListItemView {
        id: item.id.0,
        name: item.name,
        kind: match item.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
        item_count: item.item_count,
        sort_order: item.sort_order,
    }
}

fn suggestion_view(summary: imglab_core::MetadataSuggestion) -> SuggestionView {
    let confidence = service().normalize_confidence(&summary.confidence_json);
    SuggestionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        title: summary.suggested_title,
        description: summary.suggested_description,
        schema_prompt: summary.suggested_schema_prompt,
        tags: summary.suggested_tags,
        category: summary.suggested_category,
        status: summary.status,
        confidence_json: summary.confidence_json,
        created_at: summary.created_at,
        reviewed_at: summary.reviewed_at,
        confidence: confidence_score_view(confidence),
    }
}

fn review_draft_detail_view(
    summary: imglab_core::ReviewDraftDetailView,
    library_path: &Path,
) -> ReviewDraftDetailView {
    ReviewDraftDetailView {
        suggestion: suggestion_view(summary.suggestion),
        draft_seed: ReviewDraftSeedView {
            title: summary.draft_seed.title,
            description: summary.draft_seed.description,
            schema_prompt: summary.draft_seed.schema_prompt,
            tags: summary.draft_seed.tags,
            category: summary.draft_seed.category,
        },
        confidence: confidence_score_view(summary.confidence),
        history: summary.history.into_iter().map(suggestion_view).collect(),
        generated_field_results: summary
            .generated_field_results
            .into_iter()
            .map(|result| GeneratedReviewFieldResultView {
                task_id: result.task_id.0,
                field: result.field,
                value: result.value,
                base_revision: result.base_revision,
                created_at: result.created_at,
            })
            .collect(),
        related_tasks: summary
            .related_tasks
            .into_iter()
            .map(|task| RelatedTaskSummaryView {
                id: task.id.0,
                task_type: task_type_value(task.task_type),
                status: task_status_value(task.status),
                provider: task.provider,
                operation: task.operation.map(operation_value),
            })
            .collect(),
        asset: asset_detail_view(summary.asset, library_path),
    }
}

fn confidence_score_view(summary: imglab_core::ConfidenceScoreView) -> ConfidenceScoreView {
    ConfidenceScoreView {
        overall: summary.overall,
        title: summary.title,
        description: summary.description,
        schema_prompt: summary.schema_prompt,
        tags: summary.tags,
        category: summary.category,
    }
}

fn daemon_task_view(task: DaemonTask) -> DaemonTaskView {
    DaemonTaskView {
        id: task.id,
        library_id: task.library_id,
        task_type: task.task_type,
        status: task.status,
        queue_position: task.queue_position,
        priority: task.priority,
        provider: task.provider,
        operation: task.operation,
        concurrency_group: task.concurrency_group,
        attempt_count: task.attempt_count,
        max_attempts: task.max_attempts,
        next_retry_at: task.next_retry_at,
        input: task.input,
        created_at: task.created_at,
        updated_at: task.updated_at,
        last_error_code: task.last_error_code,
        last_error_message: task.last_error_message,
        error_classification: task.error_classification,
        wait_reason: task.wait_reason,
    }
}

fn daemon_task_attempt_view(attempt: DaemonTaskAttempt) -> DaemonTaskAttemptView {
    DaemonTaskAttemptView {
        id: attempt.id,
        task_id: attempt.task_id,
        attempt_number: attempt.attempt_number,
        status: attempt.status,
        started_at: attempt.started_at,
        completed_at: attempt.completed_at,
        log_path: attempt.log_path,
        exit_code: attempt.exit_code,
        error_code: attempt.error_code,
        error_message: attempt.error_message,
        error_classification: attempt.error_classification,
    }
}

fn daemon_task_event_view(event: DaemonTaskEvent) -> DaemonTaskEventView {
    DaemonTaskEventView {
        id: event.id,
        task_id: event.task_id,
        event_type: event.event_type,
        message: event.message,
        payload: event.payload,
        created_at: event.created_at,
    }
}

fn daemon_task_output_view(output: DaemonTaskOutput) -> DaemonTaskOutputView {
    DaemonTaskOutputView {
        id: output.id,
        task_id: output.task_id,
        output_type: output.output_type,
        target_id: output.target_id,
        payload: output.payload,
        created_at: output.created_at,
    }
}

fn daemon_task_detail_view(
    detail: DaemonTaskDetail,
    log_tail: String,
    log_tail_truncated: bool,
) -> DaemonTaskDetailView {
    DaemonTaskDetailView {
        task: daemon_task_view(detail.task),
        attempts: detail
            .attempts
            .into_iter()
            .map(daemon_task_attempt_view)
            .collect(),
        events: detail
            .events
            .into_iter()
            .map(daemon_task_event_view)
            .collect(),
        outputs: detail
            .outputs
            .into_iter()
            .map(daemon_task_output_view)
            .collect(),
        log_tail,
        log_tail_truncated,
    }
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateCheckView, CommandError> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(updater_error)?
        .check()
        .await
        .map_err(updater_error)?;

    Ok(UpdateCheckView {
        current_version,
        available: update.is_some(),
        update: update.map(|update| UpdateInfoView {
            version: update.version.to_string(),
            date: update.date.map(|date| date.to_string()),
            body: update.body,
        }),
    })
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<UpdateInstallView, CommandError> {
    let update = app
        .updater()
        .map_err(updater_error)?
        .check()
        .await
        .map_err(updater_error)?;

    let Some(update) = update else {
        return Ok(UpdateInstallView {
            installed: false,
            version: None,
        });
    };

    let version = update.version.to_string();
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(updater_error)?;

    Ok(UpdateInstallView {
        installed: true,
        version: Some(version),
    })
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) -> Result<(), CommandError> {
    app.restart();
}

pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_background_color(Some(tauri::window::Color(32, 37, 39, 255)))?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            create_library,
            list_libraries,
            open_library,
            library_status,
            studio_overview,
            diagnostics_overview,
            repair_library,
            hide_library,
            rename_library_alias,
            unregister_library,
            import_asset,
            export_library,
            export_library_backup_zip,
            import_library_backup_zip,
            reveal_library_folder,
            search_assets,
            query_gallery,
            get_asset_detail,
            get_asset_inspector_detail,
            generate_image,
            daemon_health,
            enqueue_generation_tasks,
            list_daemon_tasks,
            get_daemon_task_detail,
            reorder_daemon_tasks,
            cancel_daemon_task,
            retry_daemon_task,
            duplicate_daemon_task,
            update_asset_metadata,
            add_tag_to_asset,
            list_albums,
            create_manual_album,
            create_smart_album,
            add_asset_to_album,
            batch_add_assets_to_album,
            remove_asset_from_album,
            rename_album,
            delete_album,
            reorder_albums,
            reorder_album_items,
            create_suggestion,
            list_pending_suggestions,
            get_review_draft_detail,
            accept_suggestion,
            batch_accept_suggestions,
            reject_suggestion,
            batch_reject_suggestions,
            list_suggestion_history,
            generate_review_field,
            regenerate_suggestion,
            list_app_logs,
            read_app_log,
            check_for_update,
            install_update,
            restart_app
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

    #[test]
    fn expands_home_relative_library_path() {
        let normalized = normalize_library_root_path(PathBuf::from("~/Documents/image-prompt-lab"))
            .expect("normalized path");

        assert!(normalized.is_absolute());
        assert!(normalized.ends_with("Documents/image-prompt-lab"));
        assert!(!normalized.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn rejects_relative_library_path() {
        let error = normalize_library_root_path(PathBuf::from("relative/image-prompt-lab"))
            .expect_err("error");

        assert_eq!(error.code, "InvalidPath");
        assert!(error.recoverable);
    }
}
