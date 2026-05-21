use crate::application::ports::AssetRepository;
use crate::library::{
    import_asset_with_status, list_versions_for_asset, load_version,
    mark_imported_version_as_generated, persist_asset_version, LocalLibraryService,
};
use crate::{
    AssetId, AssetService, AssetSummary, AssetVersionId, CreateGenerationEventRequest,
    DomainResult, GenerationEventId, GenerationEventSummary, PersistAssetVersionRequest,
    PersistImportedAssetRequest, VersionSummary,
};
use std::path::Path;

impl AssetRepository for LocalLibraryService {
    fn load_version(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<VersionSummary> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        load_version(&connection, version_id)
    }

    fn list_versions_for_asset(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<VersionSummary>> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        list_versions_for_asset(&connection, asset_id)
    }

    fn persist_imported_asset(
        &self,
        request: PersistImportedAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        import_asset_with_status(self, request)
    }

    fn persist_asset_version(
        &self,
        request: PersistAssetVersionRequest,
    ) -> DomainResult<VersionSummary> {
        persist_asset_version(request)
    }

    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        AssetService::record_generation_event(self, request)
    }

    fn mark_version_generated(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        version_id: &AssetVersionId,
        generation_event_id: &GenerationEventId,
    ) -> DomainResult<()> {
        mark_imported_version_as_generated(library_path, asset_id, version_id, generation_event_id)
    }
}
