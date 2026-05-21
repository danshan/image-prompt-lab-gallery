use crate::application::ports::LibraryRegistry;
pub use crate::library::LocalLibraryService;
use crate::{DomainResult, LibraryId};

impl LibraryRegistry for LocalLibraryService {
    fn contains_library_id(&self, library_id: &LibraryId) -> DomainResult<bool> {
        self.registry_contains_library_id(&library_id.0)
    }
}
