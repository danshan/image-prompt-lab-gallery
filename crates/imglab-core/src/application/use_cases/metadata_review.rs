use crate::application::ports::MetadataSuggestionRepository;
use crate::{
    AssetSummary, BatchReviewMetadataSuggestionRequest, ConfidenceScoreView,
    CreateMetadataSuggestionRequest, DomainResult, MetadataSuggestion, MetadataSuggestionId,
    ReviewDraftDetailView, ReviewMetadataSuggestionRequest,
};
use std::path::Path;

pub struct CreateMetadataSuggestionUseCase<R> {
    repository: R,
}

impl<R> CreateMetadataSuggestionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> CreateMetadataSuggestionUseCase<R>
where
    R: MetadataSuggestionRepository,
{
    pub fn execute(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        self.repository.create_suggestion(request)
    }
}

pub struct ReviewMetadataSuggestionUseCase<R> {
    repository: R,
}

impl<R> ReviewMetadataSuggestionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> ReviewMetadataSuggestionUseCase<R>
where
    R: MetadataSuggestionRepository,
{
    pub fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary> {
        self.repository.accept(request)
    }

    pub fn batch_accept(
        &self,
        request: BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>> {
        self.repository.batch_accept(request)
    }

    pub fn reject(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()> {
        self.repository.reject(library_path, suggestion_id)
    }

    pub fn batch_reject(
        &self,
        library_path: &Path,
        suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()> {
        self.repository.batch_reject(library_path, suggestion_ids)
    }

    pub fn list_history(
        &self,
        library_path: &Path,
        asset_id: &crate::AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        self.repository.list_history(library_path, asset_id)
    }

    pub fn get_review_draft_detail(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<ReviewDraftDetailView> {
        self.repository
            .get_review_draft_detail(library_path, suggestion_id)
    }

    pub fn normalize_confidence(&self, confidence_json: &str) -> ConfidenceScoreView {
        self.repository.normalize_confidence(confidence_json)
    }
}
