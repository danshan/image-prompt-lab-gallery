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
    pub fn create_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        self.repository.create_suggestion(request)
    }

    pub fn list_pending(
        &self,
        library_path: &Path,
        library_id: &crate::LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        self.repository.list_pending(library_path, library_id)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetId, BatchReviewMetadataSuggestionRequest, DomainError, LibraryId,
        ReviewMetadataSuggestionRequest,
    };
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(Default)]
    struct RecordingMetadataRepository {
        created: RefCell<Vec<CreateMetadataSuggestionRequest>>,
        listed: RefCell<Vec<(PathBuf, LibraryId)>>,
    }

    impl MetadataSuggestionRepository for RecordingMetadataRepository {
        fn create_suggestion(
            &self,
            request: CreateMetadataSuggestionRequest,
        ) -> DomainResult<MetadataSuggestion> {
            self.created.borrow_mut().push(request.clone());
            Ok(MetadataSuggestion {
                id: MetadataSuggestionId("suggestion-1".to_string()),
                asset_id: request.asset_id,
                suggested_title: request.suggested_title,
                suggested_description: request.suggested_description,
                suggested_schema_prompt: request.suggested_schema_prompt,
                suggested_tags: request.suggested_tags,
                suggested_category: request.suggested_category,
                confidence_json: request.confidence_json,
                status: "pending_review".to_string(),
                created_at: None,
                reviewed_at: None,
            })
        }

        fn list_pending(
            &self,
            library_path: &Path,
            library_id: &LibraryId,
        ) -> DomainResult<Vec<MetadataSuggestion>> {
            self.listed
                .borrow_mut()
                .push((library_path.to_path_buf(), library_id.clone()));
            Ok(vec![MetadataSuggestion {
                id: MetadataSuggestionId("suggestion-1".to_string()),
                asset_id: AssetId("asset-1".to_string()),
                suggested_title: Some("Title".to_string()),
                suggested_description: None,
                suggested_schema_prompt: None,
                suggested_tags: Vec::new(),
                suggested_category: None,
                confidence_json: "{}".to_string(),
                status: "pending_review".to_string(),
                created_at: None,
                reviewed_at: None,
            }])
        }

        fn accept(&self, _request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn batch_accept(
            &self,
            _request: BatchReviewMetadataSuggestionRequest,
        ) -> DomainResult<Vec<AssetSummary>> {
            Ok(Vec::new())
        }

        fn reject(
            &self,
            _library_path: &Path,
            _suggestion_id: &MetadataSuggestionId,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn batch_reject(
            &self,
            _library_path: &Path,
            _suggestion_ids: &[MetadataSuggestionId],
        ) -> DomainResult<()> {
            Ok(())
        }

        fn list_history(
            &self,
            _library_path: &Path,
            _asset_id: &AssetId,
        ) -> DomainResult<Vec<MetadataSuggestion>> {
            Ok(Vec::new())
        }

        fn get_review_draft_detail(
            &self,
            _library_path: &Path,
            _suggestion_id: &MetadataSuggestionId,
        ) -> DomainResult<ReviewDraftDetailView> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn normalize_confidence(&self, _confidence_json: &str) -> ConfidenceScoreView {
            ConfidenceScoreView {
                overall: None,
                title: None,
                description: None,
                schema_prompt: None,
                tags: None,
                category: None,
            }
        }
    }

    #[test]
    fn review_use_case_creates_suggestions_through_repository() {
        let repository = RecordingMetadataRepository::default();
        let use_case = ReviewMetadataSuggestionUseCase::new(repository);

        let suggestion = use_case
            .create_suggestion(CreateMetadataSuggestionRequest {
                library_path: PathBuf::from("/tmp/library"),
                asset_id: AssetId("asset-1".to_string()),
                source: "test".to_string(),
                suggested_title: Some("Title".to_string()),
                suggested_description: None,
                suggested_schema_prompt: None,
                suggested_tags: vec!["tag".to_string()],
                suggested_category: None,
                confidence_json: "{}".to_string(),
            })
            .expect("suggestion should be created");

        assert_eq!(suggestion.id.0, "suggestion-1");
        assert_eq!(use_case.repository.created.borrow().len(), 1);
    }

    #[test]
    fn review_use_case_lists_pending_suggestions_through_repository() {
        let repository = RecordingMetadataRepository::default();
        let use_case = ReviewMetadataSuggestionUseCase::new(repository);

        let suggestions = use_case
            .list_pending(
                Path::new("/tmp/library"),
                &LibraryId("library-1".to_string()),
            )
            .expect("pending suggestions should be listed");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(use_case.repository.listed.borrow().len(), 1);
    }
}
