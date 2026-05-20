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
