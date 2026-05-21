use crate::application::ports::GenerationEventRepository;
use crate::library::LocalLibraryService;
use crate::{AssetService, CreateGenerationEventRequest, DomainResult, GenerationEventSummary};

impl GenerationEventRepository for LocalLibraryService {
    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        AssetService::record_generation_event(self, request)
    }
}
