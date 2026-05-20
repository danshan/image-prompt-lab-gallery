use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetVersionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationEventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataSuggestionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskAttemptId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutputId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySummary {
    pub id: LibraryId,
    pub name: String,
    pub root_path: PathBuf,
    pub hidden: bool,
    pub schema_version: u32,
}

#[derive(Debug, Clone)]
pub struct CreateLibraryRequest {
    pub root_path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ImportAssetRequest {
    pub library_path: PathBuf,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExportLibraryRequest {
    pub library_path: PathBuf,
    pub output_path: PathBuf,
    pub album_id: Option<AlbumId>,
}

#[derive(Debug, Clone)]
pub struct RenameLibraryAliasRequest {
    pub library_id: LibraryId,
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct ExportLibraryBackupRequest {
    pub library_path: PathBuf,
    pub output_zip_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ImportLibraryBackupRequest {
    pub zip_path: PathBuf,
    pub destination_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryBackupSummary {
    pub library: LibrarySummary,
    pub cloned: bool,
}

#[derive(Debug, Clone)]
pub struct RepairLibraryRequest {
    pub library_path: PathBuf,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairIssue {
    pub version_id: AssetVersionId,
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairSummary {
    pub dry_run: bool,
    pub scanned_versions: usize,
    pub files_moved: usize,
    pub paths_updated: usize,
    pub checksums_updated: usize,
    pub dimensions_updated: usize,
    pub issues: Vec<RepairIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSummary {
    pub exported_files: usize,
    pub exported_sidecars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityIssueKind {
    MissingFile,
    HashMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityIssue {
    pub version_id: AssetVersionId,
    pub path: PathBuf,
    pub kind: IntegrityIssueKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationOperation {
    TextToImage,
    ImageToImage,
}

#[derive(Debug, Clone)]
pub struct GenerationParameters {
    pub library_path: Option<PathBuf>,
    pub provider: String,
    pub model: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub operation: GenerationOperation,
    pub input_version_id: Option<AssetVersionId>,
    pub parameters_json: String,
}

#[derive(Debug, Clone)]
pub struct GenerateImageRequest {
    pub library_path: PathBuf,
    pub parameters: GenerationParameters,
    pub input_file: Option<PathBuf>,
    pub input_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct GenerationRequestInput {
    pub library_path: PathBuf,
    pub provider: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub input_file: Option<PathBuf>,
    pub input_version_id: Option<AssetVersionId>,
    pub parameters_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedGenerationRequest {
    pub provider: String,
    pub request: GenerateImageRequest,
}

#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub provider_metadata_json: String,
}

#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub images: Vec<GeneratedImage>,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

#[derive(Debug, Clone)]
pub struct AssetSummary {
    pub id: AssetId,
    pub title: Option<String>,
    pub category: Option<String>,
    pub rating: Option<u8>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct VersionSummary {
    pub id: AssetVersionId,
    pub asset_id: AssetId,
    pub parent_version_id: Option<AssetVersionId>,
    pub generation_event_id: Option<GenerationEventId>,
    pub version_number: u32,
    pub version_name: String,
    pub file_path: PathBuf,
    pub checksum_algorithm: String,
    pub checksum: String,
    pub mime_type: String,
}

pub fn version_name(version_number: u32) -> String {
    format!("v{version_number}")
}

#[derive(Debug, Clone)]
pub struct CreateGenerationEventRequest {
    pub library_path: PathBuf,
    pub asset_id: Option<AssetId>,
    pub output_version_id: Option<AssetVersionId>,
    pub provider: String,
    pub provider_model: String,
    pub operation_type: GenerationOperation,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub input_asset_version_id: Option<AssetVersionId>,
    pub parameters_json: String,
    pub raw_request_json: Option<String>,
    pub raw_response_json: Option<String>,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerationEventSummary {
    pub id: GenerationEventId,
    pub asset_id: Option<AssetId>,
    pub output_version_id: Option<AssetVersionId>,
    pub provider: String,
    pub provider_model: String,
    pub operation_type: GenerationOperation,
    pub prompt: String,
    pub parameters_json: String,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    ImageGeneration,
    MetadataFieldGeneration,
    MetadataSuggestionGeneration,
}

impl TaskType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ImageGeneration => "image_generation",
            Self::MetadataFieldGeneration => "metadata_field_generation",
            Self::MetadataSuggestionGeneration => "metadata_suggestion_generation",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "image_generation" => Some(Self::ImageGeneration),
            "metadata_field_generation" => Some(Self::MetadataFieldGeneration),
            "metadata_suggestion_generation" => Some(Self::MetadataSuggestionGeneration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    RetryWaiting,
    FailedRetryable,
    FailedFinal,
    CancelRequested,
    Canceled,
    Completed,
    InterruptedRetryable,
    InterruptedFinal,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::RetryWaiting => "retry_waiting",
            Self::FailedRetryable => "failed_retryable",
            Self::FailedFinal => "failed_final",
            Self::CancelRequested => "cancel_requested",
            Self::Canceled => "canceled",
            Self::Completed => "completed",
            Self::InterruptedRetryable => "interrupted_retryable",
            Self::InterruptedFinal => "interrupted_final",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "retry_waiting" => Some(Self::RetryWaiting),
            "failed_retryable" => Some(Self::FailedRetryable),
            "failed_final" => Some(Self::FailedFinal),
            "cancel_requested" => Some(Self::CancelRequested),
            "canceled" => Some(Self::Canceled),
            "completed" => Some(Self::Completed),
            "interrupted_retryable" => Some(Self::InterruptedRetryable),
            "interrupted_final" => Some(Self::InterruptedFinal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskErrorClassification {
    Transient,
    RetryableManual,
    Final,
    Cancel,
    Conflict,
}

impl TaskErrorClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::RetryableManual => "retryable_manual",
            Self::Final => "final",
            Self::Cancel => "cancel",
            Self::Conflict => "conflict",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "transient" => Some(Self::Transient),
            "retryable_manual" => Some(Self::RetryableManual),
            "final" => Some(Self::Final),
            "cancel" => Some(Self::Cancel),
            "conflict" => Some(Self::Conflict),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOutputType {
    Asset,
    AssetVersion,
    GenerationEvent,
    MetadataSuggestion,
    MetadataFieldResult,
}

impl TaskOutputType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::AssetVersion => "asset_version",
            Self::GenerationEvent => "generation_event",
            Self::MetadataSuggestion => "metadata_suggestion",
            Self::MetadataFieldResult => "metadata_field_result",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "asset" => Some(Self::Asset),
            "asset_version" => Some(Self::AssetVersion),
            "generation_event" => Some(Self::GenerationEvent),
            "metadata_suggestion" => Some(Self::MetadataSuggestion),
            "metadata_field_result" => Some(Self::MetadataFieldResult),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub task_type: TaskType,
    pub provider: Option<String>,
    pub operation: Option<GenerationOperation>,
    pub priority: i64,
    pub concurrency_group: Option<String>,
    pub max_attempts: u32,
    pub input_json: String,
}

#[derive(Debug, Clone)]
pub struct BatchCreateTasksRequest {
    pub library_path: PathBuf,
    pub library_id: LibraryId,
    pub tasks: Vec<CreateTaskInput>,
}

#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub id: TaskId,
    pub library_id: LibraryId,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub queue_position: i64,
    pub priority: i64,
    pub provider: Option<String>,
    pub operation: Option<GenerationOperation>,
    pub concurrency_group: Option<String>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<String>,
    pub input_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub error_classification: Option<TaskErrorClassification>,
    pub wait_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskAttempt {
    pub id: TaskAttemptId,
    pub task_id: TaskId,
    pub attempt_number: u32,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub log_path: Option<PathBuf>,
    pub exit_code: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub error_classification: Option<TaskErrorClassification>,
}

#[derive(Debug, Clone)]
pub struct TaskEvent {
    pub id: TaskEventId,
    pub task_id: TaskId,
    pub event_type: String,
    pub message: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutput {
    pub id: TaskOutputId,
    pub task_id: TaskId,
    pub output_type: TaskOutputType,
    pub target_id: String,
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct TaskDetail {
    pub task: TaskSummary,
    pub attempts: Vec<TaskAttempt>,
    pub events: Vec<TaskEvent>,
    pub outputs: Vec<TaskOutput>,
    pub output_links: Vec<TaskOutputLinkView>,
    pub related_assets: Vec<AssetId>,
    pub related_reviews: Vec<MetadataSuggestionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutputLinkView {
    pub output_id: TaskOutputId,
    pub output_type: TaskOutputType,
    pub target_id: String,
    pub asset_id: Option<AssetId>,
    pub version_id: Option<AssetVersionId>,
    pub generation_event_id: Option<GenerationEventId>,
    pub suggestion_id: Option<MetadataSuggestionId>,
}

#[derive(Debug, Clone)]
pub struct UpdateTaskStatusRequest {
    pub library_path: PathBuf,
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub next_retry_at: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub error_classification: Option<TaskErrorClassification>,
    pub wait_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppendTaskEventRequest {
    pub library_path: PathBuf,
    pub task_id: TaskId,
    pub event_type: String,
    pub message: Option<String>,
    pub payload_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppendTaskAttemptRequest {
    pub library_path: PathBuf,
    pub task_id: TaskId,
    pub status: String,
    pub log_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CompleteTaskAttemptRequest {
    pub library_path: PathBuf,
    pub attempt_id: TaskAttemptId,
    pub status: String,
    pub exit_code: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub error_classification: Option<TaskErrorClassification>,
}

#[derive(Debug, Clone)]
pub struct AppendTaskOutputRequest {
    pub library_path: PathBuf,
    pub task_id: TaskId,
    pub output_type: TaskOutputType,
    pub target_id: String,
    pub payload_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReorderQueuedTasksRequest {
    pub library_path: PathBuf,
    pub task_ids: Vec<TaskId>,
}

#[derive(Debug, Clone)]
pub struct CreateChildVersionRequest {
    pub library_path: PathBuf,
    pub asset_id: AssetId,
    pub parent_version_id: AssetVersionId,
    pub generation_event_id: Option<GenerationEventId>,
    pub source_path: PathBuf,
    pub mime_type: String,
    pub version_label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LineageEntry {
    pub version: VersionSummary,
    pub generation_event: Option<GenerationEventSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySort {
    Newest,
    Oldest,
    RatingDesc,
    TitleAsc,
    ProviderAsc,
    AlbumOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewStatusFilter {
    Any,
    Pending,
}

impl Default for ReviewStatusFilter {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Clone)]
pub struct GalleryQuery {
    pub text: Option<String>,
    pub providers: Vec<String>,
    pub min_rating: Option<u8>,
    pub review_status: ReviewStatusFilter,
    pub tags: Vec<String>,
    pub album_id: Option<AlbumId>,
    pub sort: GallerySort,
}

impl Default for GalleryQuery {
    fn default() -> Self {
        Self {
            text: None,
            providers: Vec::new(),
            min_rating: None,
            review_status: ReviewStatusFilter::Any,
            tags: Vec::new(),
            album_id: None,
            sort: GallerySort::Newest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryAssetView {
    pub id: AssetId,
    pub title: Option<String>,
    pub category: Option<String>,
    pub rating: Option<u8>,
    pub status: String,
    pub provider: Option<String>,
    pub model_label: Option<String>,
    pub prompt: Option<String>,
    pub tags: Vec<String>,
    pub review_pending_count: u32,
    pub current_version_id: Option<AssetVersionId>,
    pub current_version_number: Option<u32>,
    pub current_version_name: Option<String>,
    pub image_path: Option<PathBuf>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub version_label: Option<String>,
    pub version_count: u32,
    pub task_origin: Option<TaskOriginView>,
    pub albums: Vec<AlbumMembershipView>,
    pub album_context: Option<AlbumMembershipView>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOriginView {
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub provider: Option<String>,
    pub operation: Option<GenerationOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumMembershipView {
    pub id: AlbumId,
    pub name: String,
    pub kind: AlbumKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContextView {
    pub filename: String,
    pub relative_location: PathBuf,
    pub mime_type: String,
    pub size_bytes: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub checksum_algorithm: String,
    pub checksum: String,
    pub integrity_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryStatusView {
    pub storage_size_bytes: u64,
    pub integrity_status: String,
    pub integrity_issue_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioOverviewView {
    pub library: LibrarySummary,
    pub status: LibraryStatusView,
    pub registered_library_count: u32,
    pub missing_library_count: u32,
    pub review_pending_count: u32,
    pub task_summary: StudioTaskSummaryView,
    pub provider_health: Vec<ProviderHealthSummaryView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioTaskSummaryView {
    pub active_count: u32,
    pub queued_count: u32,
    pub running_count: u32,
    pub retry_waiting_count: u32,
    pub failed_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHealthSummaryView {
    pub provider: String,
    pub display_name: String,
    pub availability: String,
    pub credential_state: String,
    pub supported_operations: Vec<GenerationOperation>,
    pub recoverable_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetDetailView {
    pub id: AssetId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub schema_prompt: Option<String>,
    pub category: Option<String>,
    pub rating: Option<u8>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub provider: Option<String>,
    pub model_label: Option<String>,
    pub parameters_json: Option<String>,
    pub tags: Vec<String>,
    pub albums: Vec<AlbumMembershipView>,
    pub review_pending_count: u32,
    pub current_version_id: Option<AssetVersionId>,
    pub current_version_number: Option<u32>,
    pub current_version_name: Option<String>,
    pub versions: Vec<VersionSummary>,
    pub lineage: Vec<LineageEntry>,
    pub source_reference: Option<ReferenceSourceView>,
    pub file: Option<FileContextView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSourceView {
    pub asset_id: AssetId,
    pub asset_title: Option<String>,
    pub asset_status: String,
    pub version_id: AssetVersionId,
    pub version_number: u32,
    pub version_name: String,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AssetInspectorDetailView {
    pub asset: AssetDetailView,
    pub canonical_metadata: CanonicalMetadataView,
    pub pending_suggestions: Vec<PendingSuggestionSummaryView>,
    pub generated_task_origin: Option<TaskOriginView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalMetadataView {
    pub title: Option<String>,
    pub description: Option<String>,
    pub schema_prompt: Option<String>,
    pub category: Option<String>,
    pub rating: Option<u8>,
    pub tags: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSuggestionSummaryView {
    pub id: MetadataSuggestionId,
    pub asset_id: AssetId,
    pub title: Option<String>,
    pub category: Option<String>,
    pub tag_count: u32,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReviewDraftDetailView {
    pub suggestion: MetadataSuggestion,
    pub draft_seed: ReviewDraftSeedView,
    pub confidence: ConfidenceScoreView,
    pub history: Vec<MetadataSuggestion>,
    pub generated_field_results: Vec<GeneratedReviewFieldResultView>,
    pub related_tasks: Vec<RelatedTaskSummaryView>,
    pub asset: AssetDetailView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewDraftSeedView {
    pub title: Option<String>,
    pub description: Option<String>,
    pub schema_prompt: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedReviewFieldResultView {
    pub task_id: TaskId,
    pub field: String,
    pub value: String,
    pub base_revision: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedTaskSummaryView {
    pub id: TaskId,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub provider: Option<String>,
    pub operation: Option<GenerationOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsOverviewView {
    pub provider_health: Vec<ProviderHealthSummaryView>,
    pub daemon_status: DaemonStatusView,
    pub library_status: LibraryStatusView,
    pub library_count: u32,
    pub missing_library_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStatusView {
    pub state: String,
    pub recoverable_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetadataSuggestion {
    pub id: MetadataSuggestionId,
    pub asset_id: AssetId,
    pub suggested_title: Option<String>,
    pub suggested_description: Option<String>,
    pub suggested_schema_prompt: Option<String>,
    pub suggested_tags: Vec<String>,
    pub suggested_category: Option<String>,
    pub confidence_json: String,
    pub status: String,
    pub created_at: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMetadataSuggestionRequest {
    pub library_path: PathBuf,
    pub asset_id: AssetId,
    pub source: String,
    pub suggested_title: Option<String>,
    pub suggested_description: Option<String>,
    pub suggested_schema_prompt: Option<String>,
    pub suggested_tags: Vec<String>,
    pub suggested_category: Option<String>,
    pub confidence_json: String,
}

#[derive(Debug, Clone)]
pub struct ReviewMetadataSuggestionRequest {
    pub library_path: PathBuf,
    pub suggestion_id: MetadataSuggestionId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub schema_prompt: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchReviewMetadataSuggestionRequest {
    pub library_path: PathBuf,
    pub suggestions: Vec<ReviewMetadataSuggestionRequest>,
}

#[derive(Debug, Clone)]
pub struct ConfidenceScoreView {
    pub overall: Option<u8>,
    pub title: Option<u8>,
    pub description: Option<u8>,
    pub schema_prompt: Option<u8>,
    pub tags: Option<u8>,
    pub category: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct AlbumSummary {
    pub id: AlbumId,
    pub name: String,
    pub kind: AlbumKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumListItem {
    pub id: AlbumId,
    pub name: String,
    pub kind: AlbumKind,
    pub item_count: u32,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct CreateSmartAlbumRequest {
    pub library_path: PathBuf,
    pub name: String,
    pub smart_query_json: String,
}

#[derive(Debug, Clone)]
pub struct ReorderAlbumsRequest {
    pub library_path: PathBuf,
    pub album_ids: Vec<AlbumId>,
}

#[derive(Debug, Clone)]
pub struct ReorderAlbumItemsRequest {
    pub album_id: AlbumId,
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone)]
pub struct BatchAddAssetsToAlbumRequest {
    pub album_id: AlbumId,
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SmartAlbumQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub providers: Vec<String>,
    pub min_rating: Option<u8>,
    pub review_status: ReviewStatusFilter,
    pub category: Option<String>,
    pub status: Option<String>,
    pub created_at_from: Option<String>,
    pub created_at_to: Option<String>,
    pub sort: Option<GallerySort>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssetMetadataRequest {
    pub library_path: PathBuf,
    pub asset_id: AssetId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub schema_prompt: Option<String>,
    pub rating: Option<u8>,
    pub category: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlbumKind {
    Manual,
    Smart,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub min_rating: Option<u8>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
}
