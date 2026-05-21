use crate::{AssetVersionId, DomainResult, LibraryId, ManagedFileImport};
use std::path::{Path, PathBuf};

pub trait ManagedFileStore {
    fn read_source_bytes(&self, source_path: &Path) -> DomainResult<Vec<u8>>;
    fn import_original(
        &self,
        library_path: &Path,
        source_path: &Path,
        mime_type_override: Option<&str>,
    ) -> DomainResult<ManagedFileImport>;
    fn read_version_bytes(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<Vec<u8>>;
    fn write_generated_bytes(
        &self,
        library_path: &Path,
        mime_type: &str,
        bytes: &[u8],
    ) -> DomainResult<PathBuf>;
}

pub trait LibraryRegistry {
    fn contains_library_id(&self, library_id: &LibraryId) -> DomainResult<bool>;
}
