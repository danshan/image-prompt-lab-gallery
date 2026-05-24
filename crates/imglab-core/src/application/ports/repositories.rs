pub use crate::{
    AlbumService, AssetService, GalleryReadService, LibraryService, MetadataReviewService,
    SearchService, TaskService,
};

use crate::{
    AddAssetTagRequest, AlbumId, AlbumListItem, AlbumSummary, AppendTaskAttemptRequest,
    AppendTaskEventRequest, AppendTaskOutputRequest, AssetDetailView, AssetId,
    AssetInspectorDetailView, AssetSummary, AssetVersionId, BatchAddAssetsToAlbumRequest,
    BatchCreateTasksRequest, BatchReviewMetadataSuggestionRequest, CompleteTaskAttemptRequest,
    ConfidenceScoreView, CreateGenerationEventRequest, CreateLibraryRequest,
    CreateMetadataSuggestionRequest, CreateSmartAlbumRequest, DiagnosticsOverviewView,
    DomainResult, ExportLibraryBackupRequest, ExportLibraryRequest, ExportSummary,
    GalleryAssetView, GalleryQuery, GenerationEventId, GenerationEventSummary,
    ImportLibraryBackupRequest, IntegrityIssue, LibraryBackupSummary, LibraryId, LibraryStatusView,
    LibrarySummary, ListPromptOutputHistoryRequest, LoadPromptVersionRequest, MetadataSuggestion,
    MetadataSuggestionId, PersistAssetVersionRequest, PersistImportedAssetRequest,
    PromoteAssetVersionRequest, PromoteAssetVersionSummary, PromptOutputHistoryItem,
    PromptVersionView, RenameLibraryAliasRequest, ReorderAlbumItemsRequest, ReorderAlbumsRequest,
    ReorderQueuedTasksRequest, RepairLibraryRequest, RepairSummary, ReviewDraftDetailView,
    ReviewMetadataSuggestionRequest, SearchQuery, StudioOverviewView, TaskAttempt, TaskDetail,
    TaskEvent, TaskId, TaskOutput, TaskOutputType, TaskSummary, UpdateAssetMetadataRequest,
    UpdateTaskStatusRequest, VersionSummary,
};
use std::path::Path;

pub trait LibraryRepository {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary>;
    fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary>;
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
    fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>>;
    fn library_status(&self, root_path: &Path) -> DomainResult<LibraryStatusView>;
    fn studio_overview(&self, root_path: &Path) -> DomainResult<StudioOverviewView>;
    fn diagnostics_overview(&self, root_path: &Path) -> DomainResult<DiagnosticsOverviewView>;
}

pub trait AssetRepository {
    fn load_version(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<VersionSummary>;
    fn list_versions_for_asset(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<VersionSummary>>;
    fn persist_imported_asset(
        &self,
        request: PersistImportedAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)>;
    fn persist_asset_version(
        &self,
        request: PersistAssetVersionRequest,
    ) -> DomainResult<VersionSummary>;
    fn promote_version_as_asset(
        &self,
        request: PromoteAssetVersionRequest,
    ) -> DomainResult<PromoteAssetVersionSummary>;
    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary>;
    fn mark_version_generated(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        version_id: &AssetVersionId,
        generation_event_id: &GenerationEventId,
    ) -> DomainResult<()>;
    fn add_tag_to_asset(&self, request: AddAssetTagRequest) -> DomainResult<()>;
}

pub trait GenerationEventRepository {
    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary>;
}

pub trait PromptRepository {
    fn create_prompt_document(
        &self,
        request: crate::CreatePromptDocumentRequest,
    ) -> DomainResult<crate::PromptDocumentView>;

    fn update_prompt_draft(
        &self,
        request: crate::UpdatePromptDraftRequest,
    ) -> DomainResult<crate::PromptDocumentView>;

    fn save_prompt_version(
        &self,
        request: crate::SavePromptVersionRequest,
    ) -> DomainResult<crate::PromptVersionView>;

    fn list_prompt_documents(
        &self,
        request: crate::ListPromptDocumentsRequest,
    ) -> DomainResult<Vec<crate::PromptDocumentView>>;

    fn list_prompt_versions(
        &self,
        request: crate::ListPromptVersionsRequest,
    ) -> DomainResult<Vec<crate::PromptVersionView>>;

    fn load_prompt_version(
        &self,
        request: LoadPromptVersionRequest,
    ) -> DomainResult<PromptVersionView>;

    fn list_prompt_output_history(
        &self,
        request: ListPromptOutputHistoryRequest,
    ) -> DomainResult<Vec<PromptOutputHistoryItem>>;

    fn save_generation_prompt_as_prompt(
        &self,
        request: crate::SaveGenerationPromptAsPromptRequest,
    ) -> DomainResult<crate::PromptVersionView>;
}

pub trait GalleryRepository {
    fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>>;
    fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetDetailView>;
    fn get_asset_inspector_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetInspectorDetailView>;
}

pub trait MetadataSuggestionRepository {
    fn create_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion>;
    fn list_pending(
        &self,
        library_path: &Path,
        library_id: &LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>>;
    fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary>;
    fn batch_accept(
        &self,
        request: BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>>;
    fn reject(&self, library_path: &Path, suggestion_id: &MetadataSuggestionId)
        -> DomainResult<()>;
    fn batch_reject(
        &self,
        library_path: &Path,
        suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()>;
    fn list_history(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>>;
    fn get_review_draft_detail(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<ReviewDraftDetailView>;
    fn normalize_confidence(&self, confidence_json: &str) -> ConfidenceScoreView;
}

pub trait AlbumRepository {
    fn list_albums(&self, library_id: &LibraryId) -> DomainResult<Vec<AlbumListItem>>;
    fn list_albums_in_library(&self, library_path: &Path) -> DomainResult<Vec<AlbumListItem>>;
    fn create_manual_album(&self, library_id: &LibraryId, name: &str)
        -> DomainResult<AlbumSummary>;
    fn create_manual_album_in_library(
        &self,
        library_path: &Path,
        name: &str,
    ) -> DomainResult<AlbumSummary>;
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

pub trait SearchRepository {
    fn search(&self, library_id: &LibraryId, query: SearchQuery)
        -> DomainResult<Vec<AssetSummary>>;
}

pub trait TaskRepository {
    fn create_tasks(&self, request: BatchCreateTasksRequest) -> DomainResult<Vec<TaskSummary>>;
    fn list_tasks(&self, library_path: &Path) -> DomainResult<Vec<TaskSummary>>;
    fn get_task_detail(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskDetail>;
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
        library_path: &Path,
        task_id: &TaskId,
        output_type: TaskOutputType,
        target_id: &str,
    ) -> DomainResult<bool>;
    fn reorder_queued_tasks(&self, request: ReorderQueuedTasksRequest) -> DomainResult<()>;
    fn retry_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary>;
    fn duplicate_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary>;
}
