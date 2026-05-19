use crate::dto::*;
use crate::DomainResult;

pub trait LibraryService {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary>;
    fn open_library(&self, root_path: &std::path::Path) -> DomainResult<LibrarySummary>;
    fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>>;
    fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()>;
    fn rename_library_alias(
        &self,
        request: RenameLibraryAliasRequest,
    ) -> DomainResult<LibrarySummary>;
    fn unregister_library(&self, library_id: &LibraryId) -> DomainResult<()>;
    fn export_library(&self, request: ExportLibraryRequest) -> DomainResult<ExportSummary>;
    fn export_library_backup_zip(&self, request: ExportLibraryBackupRequest) -> DomainResult<()>;
    fn import_library_backup_zip(
        &self,
        request: ImportLibraryBackupRequest,
    ) -> DomainResult<LibraryBackupSummary>;
    fn repair_library(&self, request: RepairLibraryRequest) -> DomainResult<RepairSummary>;
    fn check_integrity(&self, root_path: &std::path::Path) -> DomainResult<Vec<IntegrityIssue>>;
    fn library_status(&self, root_path: &std::path::Path) -> DomainResult<LibraryStatusView>;
    fn studio_overview(&self, root_path: &std::path::Path) -> DomainResult<StudioOverviewView>;
    fn diagnostics_overview(
        &self,
        root_path: &std::path::Path,
    ) -> DomainResult<DiagnosticsOverviewView>;
}

pub trait AssetService {
    fn import_asset(
        &self,
        request: ImportAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)>;
    fn create_child_version(
        &self,
        request: CreateChildVersionRequest,
    ) -> DomainResult<VersionSummary>;
    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary>;
    fn get_lineage(
        &self,
        library_path: &std::path::Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<Vec<LineageEntry>>;
}

pub trait GenerationService {
    fn generate(&self, request: GenerateImageRequest) -> DomainResult<Vec<VersionSummary>>;
}

pub trait MetadataReviewService {
    fn create_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion>;
    fn list_pending(
        &self,
        library_path: &std::path::Path,
        library_id: &LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>>;
    fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary>;
    fn batch_accept(
        &self,
        request: BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>>;
    fn reject(
        &self,
        library_path: &std::path::Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()>;
    fn batch_reject(
        &self,
        library_path: &std::path::Path,
        suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()>;
    fn list_history(
        &self,
        library_path: &std::path::Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>>;
    fn get_review_draft_detail(
        &self,
        library_path: &std::path::Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<ReviewDraftDetailView>;
    fn normalize_confidence(&self, confidence_json: &str) -> ConfidenceScoreView;
}

pub trait AlbumService {
    fn list_albums(&self, library_id: &LibraryId) -> DomainResult<Vec<AlbumListItem>>;
    fn create_manual_album(&self, library_id: &LibraryId, name: &str)
        -> DomainResult<AlbumSummary>;
    fn create_smart_album(&self, request: CreateSmartAlbumRequest) -> DomainResult<AlbumSummary>;
    fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()>;
    fn batch_add_assets(&self, request: BatchAddAssetsToAlbumRequest) -> DomainResult<()>;
    fn remove_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()>;
    fn rename_album(&self, album_id: &AlbumId, name: &str) -> DomainResult<AlbumSummary>;
    fn delete_album(&self, album_id: &AlbumId) -> DomainResult<()>;
    fn reorder_albums(&self, request: ReorderAlbumsRequest) -> DomainResult<()>;
    fn reorder_album_items(&self, request: ReorderAlbumItemsRequest) -> DomainResult<()>;
    fn update_asset_metadata(
        &self,
        request: UpdateAssetMetadataRequest,
    ) -> DomainResult<AssetSummary>;
}

pub trait SearchService {
    fn search(&self, library_id: &LibraryId, query: SearchQuery)
        -> DomainResult<Vec<AssetSummary>>;
}

pub trait GalleryReadService {
    fn query_gallery(
        &self,
        library_path: &std::path::Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>>;
    fn get_asset_detail(
        &self,
        library_path: &std::path::Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetDetailView>;
    fn get_asset_inspector_detail(
        &self,
        library_path: &std::path::Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetInspectorDetailView>;
}

pub trait TaskService {
    fn create_tasks(&self, request: BatchCreateTasksRequest) -> DomainResult<Vec<TaskSummary>>;
    fn list_tasks(&self, library_path: &std::path::Path) -> DomainResult<Vec<TaskSummary>>;
    fn get_task_detail(
        &self,
        library_path: &std::path::Path,
        task_id: &TaskId,
    ) -> DomainResult<TaskDetail>;
    fn update_task_status(&self, request: UpdateTaskStatusRequest) -> DomainResult<TaskSummary>;
    fn append_task_event(&self, request: AppendTaskEventRequest) -> DomainResult<TaskEvent>;
    fn append_task_attempt(&self, request: AppendTaskAttemptRequest) -> DomainResult<TaskAttempt>;
    fn complete_task_attempt(
        &self,
        request: CompleteTaskAttemptRequest,
    ) -> DomainResult<TaskAttempt>;
    fn append_task_output(&self, request: AppendTaskOutputRequest) -> DomainResult<TaskOutput>;
    fn has_task_output(
        &self,
        library_path: &std::path::Path,
        task_id: &TaskId,
        output_type: TaskOutputType,
        target_id: &str,
    ) -> DomainResult<bool>;
    fn reorder_queued_tasks(&self, request: ReorderQueuedTasksRequest) -> DomainResult<()>;
    fn retry_task(
        &self,
        library_path: &std::path::Path,
        task_id: &TaskId,
    ) -> DomainResult<TaskSummary>;
    fn duplicate_task(
        &self,
        library_path: &std::path::Path,
        task_id: &TaskId,
    ) -> DomainResult<TaskSummary>;
}

pub trait ImageProvider {
    fn name(&self) -> &'static str;
    fn supports_operation(&self, operation: GenerationOperation) -> bool {
        matches!(operation, GenerationOperation::TextToImage)
    }
    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()>;
    fn generate_from_text(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult>;
    fn generate_from_image(
        &self,
        parameters: &GenerationParameters,
        input: &[u8],
    ) -> DomainResult<GenerationResult>;
}

pub trait ProviderCredentialStore {
    fn resolve_credentials(&self, provider: &str) -> DomainResult<ProviderCredentials>;
}

#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    pub provider: String,
    pub api_key: String,
}
