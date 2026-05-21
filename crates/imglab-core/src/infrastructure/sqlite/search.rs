use crate::application::ports::SearchRepository;
use crate::library::LocalLibraryService;
use crate::{AssetSummary, DomainResult, LibraryId, SearchQuery, SearchService};

impl SearchRepository for LocalLibraryService {
    fn search(
        &self,
        library_id: &LibraryId,
        query: SearchQuery,
    ) -> DomainResult<Vec<AssetSummary>> {
        SearchService::search(self, library_id, query)
    }
}
