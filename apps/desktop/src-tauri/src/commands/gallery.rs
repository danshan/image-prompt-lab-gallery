use crate::*;

#[tauri::command]
pub(crate) fn search_assets(input: SearchInput) -> Result<Vec<AssetView>, CommandError> {
    let app = desktop_app();
    let library = app.library().open_library(&input.library_path)?;
    app.search()
        .execute(
            &library.id,
            SearchQuery {
                text: input.text,
                tags: input.tags,
                min_rating: input.min_rating,
                provider: input.provider,
                status: input.status,
                category: input.category,
            },
        )
        .map(|assets| assets.into_iter().map(asset_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn query_gallery(
    input: QueryGalleryInput,
) -> Result<Vec<GalleryAssetView>, CommandError> {
    let library_path = input.library_path.clone();
    desktop_app()
        .gallery()
        .query_gallery(&library_path, gallery_query_from_input(input)?)
        .map(|items| {
            items
                .into_iter()
                .map(|item| gallery_asset_view(&library_path, item))
                .collect()
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn get_asset_detail(input: AssetDetailInput) -> Result<AssetDetailView, CommandError> {
    let current_version_id = input
        .current_version_id
        .as_ref()
        .map(|id| imglab_core::AssetVersionId(id.clone()));
    desktop_app()
        .gallery()
        .get_asset_detail(
            &input.library_path,
            &AssetId(input.asset_id),
            current_version_id.as_ref(),
        )
        .map(|detail| asset_detail_view(detail, &input.library_path))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn get_asset_inspector_detail(
    input: AssetDetailInput,
) -> Result<AssetInspectorDetailView, CommandError> {
    let current_version_id = input
        .current_version_id
        .as_ref()
        .map(|id| imglab_core::AssetVersionId(id.clone()));
    desktop_app()
        .gallery()
        .get_asset_inspector_detail(
            &input.library_path,
            &AssetId(input.asset_id),
            current_version_id.as_ref(),
        )
        .map(|detail| asset_inspector_detail_view(detail, &input.library_path))
        .map_err(Into::into)
}
