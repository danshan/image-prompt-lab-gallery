#[tauri::command]
fn update_asset_metadata(input: UpdateMetadataInput) -> Result<AssetView, CommandError> {
    service()
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
fn add_tag_to_asset(input: AddTagInput) -> Result<(), CommandError> {
    service()
        .add_tag_to_asset(&input.library_path, &AssetId(input.asset_id), &input.tag)
        .map_err(Into::into)
}
