use crate::application::ports::MetadataSuggestionRepository;
use crate::library::LocalLibraryService;
use crate::{
    AssetId, AssetSummary, BatchReviewMetadataSuggestionRequest, ConfidenceScoreView,
    CreateMetadataSuggestionRequest, DomainResult, MetadataReviewService, MetadataSuggestion,
    MetadataSuggestionId, ReviewDraftDetailView, ReviewMetadataSuggestionRequest,
};
use std::path::Path;

impl MetadataSuggestionRepository for LocalLibraryService {
    fn create_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        MetadataReviewService::create_suggestion(self, request)
    }

    fn list_pending(
        &self,
        library_path: &Path,
        library_id: &crate::LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        MetadataReviewService::list_pending(self, library_path, library_id)
    }

    fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary> {
        MetadataReviewService::accept(self, request)
    }

    fn batch_accept(
        &self,
        request: BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>> {
        MetadataReviewService::batch_accept(self, request)
    }

    fn reject(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()> {
        MetadataReviewService::reject(self, library_path, suggestion_id)
    }

    fn batch_reject(
        &self,
        library_path: &Path,
        suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()> {
        MetadataReviewService::batch_reject(self, library_path, suggestion_ids)
    }

    fn list_history(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        MetadataReviewService::list_history(self, library_path, asset_id)
    }

    fn get_review_draft_detail(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<ReviewDraftDetailView> {
        MetadataReviewService::get_review_draft_detail(self, library_path, suggestion_id)
    }

    fn normalize_confidence(&self, confidence_json: &str) -> ConfidenceScoreView {
        MetadataReviewService::normalize_confidence(self, confidence_json)
    }
}
