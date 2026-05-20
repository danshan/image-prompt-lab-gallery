use crate::*;

#[tauri::command]
pub(crate) fn list_albums(library_path: PathBuf) -> Result<Vec<AlbumListItemView>, CommandError> {
    service()
        .list_albums_in_library(&library_path)
        .map(|albums| albums.into_iter().map(album_list_item_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn create_manual_album(input: CreateAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .create_manual_album_in_library(&input.library_path, &input.name)
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn create_smart_album(input: CreateSmartAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .create_smart_album(CreateSmartAlbumRequest {
            library_path: input.library_path,
            name: input.name,
            smart_query_json: input.smart_query_json,
        })
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn add_asset_to_album(input: AddAlbumAssetInput) -> Result<(), CommandError> {
    service()
        .add_asset(
            &imglab_core::AlbumId(input.album_id),
            &AssetId(input.asset_id),
        )
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn batch_add_assets_to_album(
    input: BatchAddAlbumAssetsInput,
) -> Result<(), CommandError> {
    service()
        .batch_add_assets(BatchAddAssetsToAlbumRequest {
            album_id: AlbumId(input.album_id),
            asset_ids: input.asset_ids.into_iter().map(AssetId).collect(),
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn remove_asset_from_album(input: RemoveAlbumAssetInput) -> Result<(), CommandError> {
    service()
        .remove_asset(&AlbumId(input.album_id), &AssetId(input.asset_id))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn rename_album(input: RenameAlbumInput) -> Result<AlbumView, CommandError> {
    service()
        .rename_album(&AlbumId(input.album_id), &input.name)
        .map(album_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn delete_album(album_id: String) -> Result<(), CommandError> {
    service()
        .delete_album(&AlbumId(album_id))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn reorder_albums(input: ReorderAlbumsInput) -> Result<(), CommandError> {
    service()
        .reorder_albums(ReorderAlbumsRequest {
            library_path: input.library_path,
            album_ids: input.album_ids.into_iter().map(AlbumId).collect(),
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn reorder_album_items(input: ReorderAlbumItemsInput) -> Result<(), CommandError> {
    service()
        .reorder_album_items(ReorderAlbumItemsRequest {
            album_id: AlbumId(input.album_id),
            asset_ids: input.asset_ids.into_iter().map(AssetId).collect(),
        })
        .map_err(Into::into)
}
