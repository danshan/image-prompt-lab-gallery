use crate::application::ports::GalleryRepository;
use crate::library::LocalLibraryService;
use crate::{
    AssetDetailView, AssetId, AssetInspectorDetailView, AssetVersionId, DomainResult,
    GalleryAssetView, GalleryQuery, GalleryReadService,
};
use std::path::Path;

impl GalleryRepository for LocalLibraryService {
    fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>> {
        GalleryReadService::query_gallery(self, library_path, query)
    }

    fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetDetailView> {
        GalleryReadService::get_asset_detail(self, library_path, asset_id, current_version_id)
    }

    fn get_asset_inspector_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetInspectorDetailView> {
        GalleryReadService::get_asset_inspector_detail(
            self,
            library_path,
            asset_id,
            current_version_id,
        )
    }
}
