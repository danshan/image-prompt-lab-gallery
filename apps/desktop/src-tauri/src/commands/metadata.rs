use crate::*;

#[tauri::command]
pub(crate) fn update_asset_metadata(input: UpdateMetadataInput) -> Result<AssetView, CommandError> {
    desktop_app()
        .albums()
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: input.library_path,
            asset_id: AssetId(input.asset_id),
            title: input.title,
            description: input.description,
            schema_prompt: input.schema_prompt,
            rating: input.rating,
            category: input.category,
            status: input.status,
        })
        .map(asset_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn add_tag_to_asset(input: AddTagInput) -> Result<(), CommandError> {
    desktop_app()
        .library()
        .add_tag_to_asset(&input.library_path, &AssetId(input.asset_id), &input.tag)
        .map_err(Into::into)
}
