use crate::dto::*;
use crate::DomainResult;

pub trait LibraryService {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary>;
    fn open_library(&self, root_path: &std::path::Path) -> DomainResult<LibrarySummary>;
    fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>>;
    fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()>;
    fn export_library(&self, request: ExportLibraryRequest) -> DomainResult<ExportSummary>;
    fn check_integrity(&self, root_path: &std::path::Path) -> DomainResult<Vec<IntegrityIssue>>;
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
    fn reject(
        &self,
        library_path: &std::path::Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()>;
}

pub trait AlbumService {
    fn create_manual_album(&self, library_id: &LibraryId, name: &str)
        -> DomainResult<AlbumSummary>;
    fn create_smart_album(&self, request: CreateSmartAlbumRequest) -> DomainResult<AlbumSummary>;
    fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()>;
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
