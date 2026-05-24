use crate::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateInfoView {
    pub(crate) version: String,
    pub(crate) date: Option<String>,
    pub(crate) body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateCheckView {
    pub(crate) current_version: String,
    pub(crate) available: bool,
    pub(crate) update: Option<UpdateInfoView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateInstallView {
    pub(crate) installed: bool,
    pub(crate) version: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root_path: PathBuf,
    pub(crate) hidden: bool,
    pub(crate) schema_version: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryStatusView {
    pub(crate) storage_size_bytes: u64,
    pub(crate) integrity_status: String,
    pub(crate) integrity_issue_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryBackupView {
    pub(crate) library: LibraryView,
    pub(crate) cloned: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StudioOverviewView {
    pub(crate) library: LibraryView,
    pub(crate) status: LibraryStatusView,
    pub(crate) registered_library_count: u32,
    pub(crate) missing_library_count: u32,
    pub(crate) review_pending_count: u32,
    pub(crate) task_summary: StudioTaskSummaryView,
    pub(crate) provider_health: Vec<ProviderHealthSummaryView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StudioTaskSummaryView {
    pub(crate) active_count: u32,
    pub(crate) queued_count: u32,
    pub(crate) running_count: u32,
    pub(crate) retry_waiting_count: u32,
    pub(crate) failed_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderHealthSummaryView {
    pub(crate) provider: String,
    pub(crate) display_name: String,
    pub(crate) availability: String,
    pub(crate) credential_state: String,
    pub(crate) supported_operations: Vec<String>,
    pub(crate) recoverable_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticsOverviewView {
    pub(crate) provider_health: Vec<ProviderHealthSummaryView>,
    pub(crate) daemon_status: DaemonStatusView,
    pub(crate) library_status: LibraryStatusView,
    pub(crate) library_count: u32,
    pub(crate) missing_library_count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonStatusView {
    pub(crate) state: String,
    pub(crate) recoverable_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairIssueView {
    pub(crate) version_id: String,
    pub(crate) path: PathBuf,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairSummaryView {
    pub(crate) dry_run: bool,
    pub(crate) scanned_versions: usize,
    pub(crate) files_moved: usize,
    pub(crate) paths_updated: usize,
    pub(crate) checksums_updated: usize,
    pub(crate) dimensions_updated: usize,
    pub(crate) issues: Vec<RepairIssueView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetView {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) rating: Option<u8>,
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GalleryAssetView {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) rating: Option<u8>,
    pub(crate) status: String,
    pub(crate) provider: Option<String>,
    pub(crate) model_label: Option<String>,
    pub(crate) prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) review_pending_count: u32,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_number: Option<u32>,
    pub(crate) current_version_name: Option<String>,
    pub(crate) current_version_tree_name: Option<String>,
    pub(crate) image_path: Option<PathBuf>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) version_label: Option<String>,
    pub(crate) version_count: u32,
    pub(crate) version_tree_branch_count: u32,
    pub(crate) task_origin: Option<TaskOriginView>,
    pub(crate) albums: Vec<AlbumView>,
    pub(crate) album_context: Option<AlbumView>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskOriginView {
    pub(crate) task_id: String,
    pub(crate) task_type: String,
    pub(crate) status: String,
    pub(crate) provider: Option<String>,
    pub(crate) operation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VersionView {
    pub(crate) id: String,
    pub(crate) asset_id: String,
    pub(crate) parent_version_id: Option<String>,
    pub(crate) generation_event_id: Option<String>,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) file_path: PathBuf,
    pub(crate) checksum_algorithm: String,
    pub(crate) checksum: String,
    pub(crate) mime_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReferenceSourceView {
    pub(crate) asset_id: String,
    pub(crate) asset_title: Option<String>,
    pub(crate) asset_status: String,
    pub(crate) version_id: String,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) file_path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LineageEntryView {
    pub(crate) version: VersionView,
    pub(crate) generation_event: Option<GenerationEventView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VersionTreeNodeView {
    pub(crate) version_id: String,
    pub(crate) parent_version_id: Option<String>,
    pub(crate) tree_name: String,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) file_path: PathBuf,
    pub(crate) created_at: String,
    pub(crate) provider: Option<String>,
    pub(crate) model_label: Option<String>,
    pub(crate) generation_status: Option<String>,
    pub(crate) children: Vec<VersionTreeNodeView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VersionTreeIssueView {
    pub(crate) kind: String,
    pub(crate) version_id: Option<String>,
    pub(crate) parent_version_id: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromotedSourceView {
    pub(crate) source_asset_id: String,
    pub(crate) source_asset_title: Option<String>,
    pub(crate) source_version_id: String,
    pub(crate) source_version_number: u32,
    pub(crate) source_version_name: String,
    pub(crate) source_version_tree_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromoteAssetVersionView {
    pub(crate) asset: AssetView,
    pub(crate) version: VersionView,
    pub(crate) promoted_from: PromotedSourceView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GenerationEventView {
    pub(crate) id: String,
    pub(crate) asset_id: Option<String>,
    pub(crate) output_version_id: Option<String>,
    pub(crate) provider: String,
    pub(crate) provider_model: String,
    pub(crate) operation_type: String,
    pub(crate) prompt: String,
    pub(crate) prompt_version_id: Option<String>,
    pub(crate) parameters_json: String,
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptDocumentView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) draft_body: String,
    pub(crate) draft_negative_prompt: Option<String>,
    pub(crate) draft_style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
    pub(crate) latest_version_id: Option<String>,
    pub(crate) latest_version_number: Option<u32>,
    pub(crate) latest_version_name: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) archived_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptVersionView {
    pub(crate) id: String,
    pub(crate) prompt_id: String,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) body: String,
    pub(crate) negative_prompt: Option<String>,
    pub(crate) style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct PromptLineageView {
    pub(crate) prompt_id: String,
    pub(crate) prompt_name: String,
    pub(crate) prompt_version_id: String,
    pub(crate) prompt_version_number: u32,
    pub(crate) prompt_version_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenderPromptRunView {
    pub(crate) prompt_version_id: String,
    pub(crate) prompt_id: String,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) rendered_prompt: String,
    pub(crate) rendered_negative_prompt: Option<String>,
    pub(crate) values_json: String,
    pub(crate) parameter_preset_json: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptOutputHistoryItemView {
    pub(crate) generation_event_id: String,
    pub(crate) asset_id: Option<String>,
    pub(crate) output_version_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) provider: String,
    pub(crate) provider_model: String,
    pub(crate) status: String,
    pub(crate) prompt_snapshot: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AlbumView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AlbumListItemView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) item_count: u32,
    pub(crate) sort_order: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileContextView {
    pub(crate) filename: String,
    pub(crate) relative_location: PathBuf,
    pub(crate) mime_type: String,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) checksum_algorithm: String,
    pub(crate) checksum: String,
    pub(crate) integrity_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetDetailView {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) rating: Option<u8>,
    pub(crate) status: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) prompt: Option<String>,
    pub(crate) negative_prompt: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model_label: Option<String>,
    pub(crate) parameters_json: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) albums: Vec<AlbumView>,
    pub(crate) review_pending_count: u32,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_number: Option<u32>,
    pub(crate) current_version_name: Option<String>,
    pub(crate) focused_version_id: Option<String>,
    pub(crate) focused_version_tree_name: Option<String>,
    pub(crate) focused_version: Option<VersionView>,
    pub(crate) versions: Vec<VersionView>,
    pub(crate) version_tree: Vec<VersionTreeNodeView>,
    pub(crate) version_tree_issues: Vec<VersionTreeIssueView>,
    pub(crate) lineage: Vec<LineageEntryView>,
    pub(crate) source_reference: Option<ReferenceSourceView>,
    pub(crate) promoted_from: Option<PromotedSourceView>,
    pub(crate) file: Option<FileContextView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetInspectorDetailView {
    pub(crate) asset: AssetDetailView,
    pub(crate) canonical_metadata: CanonicalMetadataView,
    pub(crate) pending_suggestions: Vec<PendingSuggestionSummaryView>,
    pub(crate) generated_task_origin: Option<TaskOriginView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanonicalMetadataView {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) rating: Option<u8>,
    pub(crate) tags: Vec<String>,
    pub(crate) status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingSuggestionSummaryView {
    pub(crate) id: String,
    pub(crate) asset_id: String,
    pub(crate) title: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) tag_count: u32,
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SuggestionView {
    pub(crate) id: String,
    pub(crate) asset_id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) category: Option<String>,
    pub(crate) status: String,
    pub(crate) confidence_json: String,
    pub(crate) created_at: Option<String>,
    pub(crate) reviewed_at: Option<String>,
    pub(crate) confidence: ConfidenceScoreView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfidenceScoreView {
    pub(crate) overall: Option<u8>,
    pub(crate) title: Option<u8>,
    pub(crate) description: Option<u8>,
    pub(crate) schema_prompt: Option<u8>,
    pub(crate) tags: Option<u8>,
    pub(crate) category: Option<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewDraftDetailView {
    pub(crate) suggestion: SuggestionView,
    pub(crate) draft_seed: ReviewDraftSeedView,
    pub(crate) confidence: ConfidenceScoreView,
    pub(crate) history: Vec<SuggestionView>,
    pub(crate) generated_field_results: Vec<GeneratedReviewFieldResultView>,
    pub(crate) related_tasks: Vec<RelatedTaskSummaryView>,
    pub(crate) asset: AssetDetailView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewDraftSeedView {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) category: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneratedReviewFieldResultView {
    pub(crate) task_id: String,
    pub(crate) field: String,
    pub(crate) value: String,
    pub(crate) base_revision: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelatedTaskSummaryView {
    pub(crate) id: String,
    pub(crate) task_type: String,
    pub(crate) status: String,
    pub(crate) provider: Option<String>,
    pub(crate) operation: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateLibraryInput {
    pub(crate) root_path: PathBuf,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportAssetInput {
    pub(crate) library_path: PathBuf,
    pub(crate) source_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportLibraryInput {
    pub(crate) library_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) album_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenameLibraryAliasInput {
    pub(crate) library_id: String,
    pub(crate) alias: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportLibraryBackupInput {
    pub(crate) library_path: PathBuf,
    pub(crate) output_zip_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportLibraryBackupInput {
    pub(crate) zip_path: PathBuf,
    pub(crate) destination_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairLibraryInput {
    pub(crate) library_path: PathBuf,
    pub(crate) dry_run: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchInput {
    pub(crate) library_path: PathBuf,
    pub(crate) text: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) min_rating: Option<u8>,
    pub(crate) provider: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QueryGalleryInput {
    pub(crate) library_path: PathBuf,
    pub(crate) text: Option<String>,
    pub(crate) providers: Option<Vec<String>>,
    pub(crate) min_rating: Option<u8>,
    pub(crate) review_status: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) album_filter: Option<GalleryAlbumFilterInput>,
    pub(crate) album_id: Option<String>,
    pub(crate) sort: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GalleryAlbumFilterInput {
    pub(crate) mode: String,
    pub(crate) album_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetDetailInput {
    pub(crate) library_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) current_version_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromoteAssetVersionInput {
    pub(crate) library_path: PathBuf,
    pub(crate) source_version_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GenerateImageInput {
    pub(crate) library_path: PathBuf,
    pub(crate) provider: String,
    pub(crate) prompt: String,
    pub(crate) negative_prompt: Option<String>,
    pub(crate) input_file: Option<PathBuf>,
    pub(crate) input_version_id: Option<String>,
    pub(crate) parameters_json: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListPromptDocumentsInput {
    pub(crate) library_path: PathBuf,
    pub(crate) query: Option<String>,
    pub(crate) include_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreatePromptDocumentInput {
    pub(crate) library_path: PathBuf,
    pub(crate) name: String,
    pub(crate) draft_body: String,
    pub(crate) draft_negative_prompt: Option<String>,
    pub(crate) draft_style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdatePromptDraftInput {
    pub(crate) library_path: PathBuf,
    pub(crate) prompt_id: String,
    pub(crate) name: String,
    pub(crate) draft_body: String,
    pub(crate) draft_negative_prompt: Option<String>,
    pub(crate) draft_style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SavePromptVersionInput {
    pub(crate) library_path: PathBuf,
    pub(crate) prompt_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListPromptVersionsInput {
    pub(crate) library_path: PathBuf,
    pub(crate) prompt_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenderPromptRunInput {
    pub(crate) library_path: PathBuf,
    pub(crate) prompt_version_id: String,
    pub(crate) values_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListPromptOutputHistoryInput {
    pub(crate) library_path: PathBuf,
    pub(crate) prompt_version_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskQueryInput {
    pub(crate) library_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnqueueGenerationTasksInput {
    pub(crate) library_path: PathBuf,
    pub(crate) tasks: Vec<GenerationTaskDraftInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderDaemonTasksInput {
    pub(crate) library_path: PathBuf,
    pub(crate) task_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskActionInput {
    pub(crate) task_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GenerationTaskDraftInput {
    pub(crate) task_type: Option<String>,
    pub(crate) provider: String,
    pub(crate) prompt: String,
    pub(crate) negative_prompt: Option<String>,
    pub(crate) prompt_version_id: Option<String>,
    pub(crate) operation: Option<String>,
    pub(crate) input_file: Option<PathBuf>,
    pub(crate) input_version_id: Option<String>,
    pub(crate) parameters_json: Option<String>,
    pub(crate) priority: Option<i64>,
    pub(crate) max_attempts: Option<u32>,
    pub(crate) input: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskView {
    pub(crate) id: String,
    pub(crate) library_id: String,
    pub(crate) task_type: String,
    pub(crate) status: String,
    pub(crate) queue_position: i64,
    pub(crate) priority: i64,
    pub(crate) provider: Option<String>,
    pub(crate) operation: Option<String>,
    pub(crate) concurrency_group: Option<String>,
    pub(crate) attempt_count: u32,
    pub(crate) max_attempts: u32,
    pub(crate) next_retry_at: Option<String>,
    pub(crate) input: serde_json::Value,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) last_error_code: Option<String>,
    pub(crate) last_error_message: Option<String>,
    pub(crate) error_classification: Option<String>,
    pub(crate) wait_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskAttemptView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) attempt_number: u32,
    pub(crate) status: String,
    pub(crate) started_at: String,
    pub(crate) completed_at: Option<String>,
    pub(crate) log_path: Option<PathBuf>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_classification: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskEventView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) event_type: String,
    pub(crate) message: Option<String>,
    pub(crate) payload: Option<serde_json::Value>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskOutputView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) output_type: String,
    pub(crate) target_id: String,
    pub(crate) payload: Option<serde_json::Value>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonTaskDetailView {
    pub(crate) task: DaemonTaskView,
    pub(crate) attempts: Vec<DaemonTaskAttemptView>,
    pub(crate) events: Vec<DaemonTaskEventView>,
    pub(crate) outputs: Vec<DaemonTaskOutputView>,
    pub(crate) log_tail: String,
    pub(crate) log_tail_truncated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateMetadataInput {
    pub(crate) library_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) rating: Option<u8>,
    pub(crate) category: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddTagInput {
    pub(crate) library_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) tag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAlbumInput {
    pub(crate) library_path: PathBuf,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddAlbumAssetInput {
    pub(crate) album_id: String,
    pub(crate) asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchAddAlbumAssetsInput {
    pub(crate) album_id: String,
    pub(crate) asset_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenameAlbumInput {
    pub(crate) album_id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveAlbumAssetInput {
    pub(crate) album_id: String,
    pub(crate) asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderAlbumsInput {
    pub(crate) library_path: PathBuf,
    pub(crate) album_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderAlbumItemsInput {
    pub(crate) album_id: String,
    pub(crate) asset_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSmartAlbumInput {
    pub(crate) library_path: PathBuf,
    pub(crate) name: String,
    pub(crate) smart_query_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSuggestionInput {
    pub(crate) library_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) category: Option<String>,
    pub(crate) confidence_json: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewSuggestionInput {
    pub(crate) library_path: PathBuf,
    pub(crate) suggestion_id: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchReviewSuggestionsInput {
    pub(crate) library_path: PathBuf,
    pub(crate) suggestions: Vec<ReviewSuggestionInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RejectSuggestionInput {
    pub(crate) library_path: PathBuf,
    pub(crate) suggestion_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchRejectSuggestionsInput {
    pub(crate) library_path: PathBuf,
    pub(crate) suggestion_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SuggestionHistoryInput {
    pub(crate) library_path: PathBuf,
    pub(crate) asset_id: String,
}
