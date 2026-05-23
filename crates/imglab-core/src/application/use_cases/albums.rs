use crate::application::ports::{AlbumRepository, GalleryRepository, SearchRepository};
use crate::{
    AlbumId, AlbumListItem, AlbumSummary, AssetDetailView, AssetId, AssetInspectorDetailView,
    AssetSummary, AssetVersionId, BatchAddAssetsToAlbumRequest, CreateSmartAlbumRequest,
    DomainResult, GalleryAssetView, GalleryQuery, LibraryId, ReorderAlbumItemsRequest,
    ReorderAlbumsRequest, SearchQuery, UpdateAssetMetadataRequest,
};
use std::path::Path;

pub struct AlbumUseCase<R> {
    repository: R,
}

impl<R> AlbumUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> AlbumUseCase<R>
where
    R: AlbumRepository,
{
    pub fn list_albums(&self, library_id: &LibraryId) -> DomainResult<Vec<AlbumListItem>> {
        self.repository.list_albums(library_id)
    }

    pub fn list_albums_in_library(&self, library_path: &Path) -> DomainResult<Vec<AlbumListItem>> {
        self.repository.list_albums_in_library(library_path)
    }

    pub fn create_manual_album(
        &self,
        library_id: &LibraryId,
        name: &str,
    ) -> DomainResult<AlbumSummary> {
        self.repository.create_manual_album(library_id, name)
    }

    pub fn create_manual_album_in_library(
        &self,
        library_path: &Path,
        name: &str,
    ) -> DomainResult<AlbumSummary> {
        self.repository
            .create_manual_album_in_library(library_path, name)
    }

    pub fn create_smart_album(
        &self,
        request: CreateSmartAlbumRequest,
    ) -> DomainResult<AlbumSummary> {
        self.repository.create_smart_album(request)
    }

    pub fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        self.repository.add_asset(album_id, asset_id)
    }

    pub fn batch_add_assets(&self, request: BatchAddAssetsToAlbumRequest) -> DomainResult<()> {
        self.repository.batch_add_assets(request)
    }

    pub fn remove_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        self.repository.remove_asset(album_id, asset_id)
    }

    pub fn rename_album(&self, album_id: &AlbumId, name: &str) -> DomainResult<AlbumSummary> {
        self.repository.rename_album(album_id, name)
    }

    pub fn delete_album(&self, album_id: &AlbumId) -> DomainResult<()> {
        self.repository.delete_album(album_id)
    }

    pub fn reorder_albums(&self, request: ReorderAlbumsRequest) -> DomainResult<()> {
        self.repository.reorder_albums(request)
    }

    pub fn reorder_album_items(&self, request: ReorderAlbumItemsRequest) -> DomainResult<()> {
        self.repository.reorder_album_items(request)
    }

    pub fn update_asset_metadata(
        &self,
        request: UpdateAssetMetadataRequest,
    ) -> DomainResult<AssetSummary> {
        self.repository.update_asset_metadata(request)
    }
}

pub struct QueryGalleryUseCase<R> {
    repository: R,
}

impl<R> QueryGalleryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> QueryGalleryUseCase<R>
where
    R: GalleryRepository,
{
    pub fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>> {
        self.repository.query_gallery(library_path, query)
    }

    pub fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetDetailView> {
        self.repository
            .get_asset_detail(library_path, asset_id, current_version_id)
    }

    pub fn get_asset_inspector_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<AssetInspectorDetailView> {
        self.repository
            .get_asset_inspector_detail(library_path, asset_id, current_version_id)
    }
}

pub struct SearchUseCase<R> {
    repository: R,
}

impl<R> SearchUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> SearchUseCase<R>
where
    R: SearchRepository,
{
    pub fn execute(
        &self,
        library_id: &LibraryId,
        query: SearchQuery,
    ) -> DomainResult<Vec<AssetSummary>> {
        self.repository.search(library_id, query)
    }
}
