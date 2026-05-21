use crate::application::ports::AlbumRepository;
use crate::library::LocalLibraryService;
use crate::{
    AlbumId, AlbumListItem, AlbumService, AlbumSummary, AssetId, AssetSummary,
    BatchAddAssetsToAlbumRequest, CreateSmartAlbumRequest, DomainResult, LibraryId,
    ReorderAlbumItemsRequest, ReorderAlbumsRequest, UpdateAssetMetadataRequest,
};

impl AlbumRepository for LocalLibraryService {
    fn list_albums(&self, library_id: &LibraryId) -> DomainResult<Vec<AlbumListItem>> {
        AlbumService::list_albums(self, library_id)
    }

    fn create_manual_album(
        &self,
        library_id: &LibraryId,
        name: &str,
    ) -> DomainResult<AlbumSummary> {
        AlbumService::create_manual_album(self, library_id, name)
    }

    fn create_smart_album(&self, request: CreateSmartAlbumRequest) -> DomainResult<AlbumSummary> {
        AlbumService::create_smart_album(self, request)
    }

    fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        AlbumService::add_asset(self, album_id, asset_id)
    }

    fn batch_add_assets(&self, request: BatchAddAssetsToAlbumRequest) -> DomainResult<()> {
        AlbumService::batch_add_assets(self, request)
    }

    fn remove_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        AlbumService::remove_asset(self, album_id, asset_id)
    }

    fn rename_album(&self, album_id: &AlbumId, name: &str) -> DomainResult<AlbumSummary> {
        AlbumService::rename_album(self, album_id, name)
    }

    fn delete_album(&self, album_id: &AlbumId) -> DomainResult<()> {
        AlbumService::delete_album(self, album_id)
    }

    fn reorder_albums(&self, request: ReorderAlbumsRequest) -> DomainResult<()> {
        AlbumService::reorder_albums(self, request)
    }

    fn reorder_album_items(&self, request: ReorderAlbumItemsRequest) -> DomainResult<()> {
        AlbumService::reorder_album_items(self, request)
    }

    fn update_asset_metadata(
        &self,
        request: UpdateAssetMetadataRequest,
    ) -> DomainResult<AssetSummary> {
        AlbumService::update_asset_metadata(self, request)
    }
}
