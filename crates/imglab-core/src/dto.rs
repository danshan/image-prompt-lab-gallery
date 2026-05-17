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

#[derive(Debug, Clone)]
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
    pub file_path: PathBuf,
    pub checksum_algorithm: String,
    pub checksum: String,
    pub mime_type: String,
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
    pub image_path: Option<PathBuf>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub version_label: Option<String>,
    pub version_count: u32,
    pub created_at: String,
    pub updated_at: String,
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
    pub versions: Vec<VersionSummary>,
    pub lineage: Vec<LineageEntry>,
    pub file: Option<FileContextView>,
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
