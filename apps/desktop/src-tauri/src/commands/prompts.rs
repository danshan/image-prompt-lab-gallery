use crate::*;

#[tauri::command]
pub(crate) fn list_prompt_documents(
    input: ListPromptDocumentsInput,
) -> Result<Vec<PromptDocumentView>, CommandError> {
    desktop_app()
        .prompts()
        .list_prompt_documents(imglab_core::ListPromptDocumentsRequest {
            library_path: input.library_path,
            query: input.query,
            include_archived: input.include_archived.unwrap_or(false),
        })
        .map(|items| items.into_iter().map(prompt_document_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn create_prompt_document(
    input: CreatePromptDocumentInput,
) -> Result<PromptDocumentView, CommandError> {
    desktop_app()
        .prompts()
        .create_prompt_document(imglab_core::CreatePromptDocumentRequest {
            library_path: input.library_path,
            name: input.name,
            draft_body: input.draft_body,
            draft_negative_prompt: input.draft_negative_prompt,
            draft_style_prompt: input.draft_style_prompt,
            variables_schema_json: input.variables_schema_json,
            default_values_json: input.default_values_json,
            parameter_preset_json: input.parameter_preset_json,
            notes: input.notes,
        })
        .map(prompt_document_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn update_prompt_draft(
    input: UpdatePromptDraftInput,
) -> Result<PromptDocumentView, CommandError> {
    desktop_app()
        .prompts()
        .update_prompt_draft(imglab_core::UpdatePromptDraftRequest {
            library_path: input.library_path,
            prompt_id: input.prompt_id,
            name: input.name,
            draft_body: input.draft_body,
            draft_negative_prompt: input.draft_negative_prompt,
            draft_style_prompt: input.draft_style_prompt,
            variables_schema_json: input.variables_schema_json,
            default_values_json: input.default_values_json,
            parameter_preset_json: input.parameter_preset_json,
            notes: input.notes,
        })
        .map(prompt_document_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn save_prompt_version(
    input: SavePromptVersionInput,
) -> Result<PromptVersionView, CommandError> {
    desktop_app()
        .prompts()
        .save_prompt_version(imglab_core::SavePromptVersionRequest {
            library_path: input.library_path,
            prompt_id: input.prompt_id,
        })
        .map(prompt_version_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn list_prompt_versions(
    input: ListPromptVersionsInput,
) -> Result<Vec<PromptVersionView>, CommandError> {
    desktop_app()
        .prompts()
        .list_prompt_versions(imglab_core::ListPromptVersionsRequest {
            library_path: input.library_path,
            prompt_id: input.prompt_id,
        })
        .map(|items| items.into_iter().map(prompt_version_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn render_prompt_run(
    input: RenderPromptRunInput,
) -> Result<RenderPromptRunView, CommandError> {
    desktop_app()
        .prompts()
        .render_prompt_run(imglab_core::RenderPromptRunRequest {
            library_path: input.library_path,
            prompt_version_id: input.prompt_version_id,
            values_json: input.values_json,
        })
        .map(render_prompt_run_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn list_prompt_output_history(
    input: ListPromptOutputHistoryInput,
) -> Result<Vec<PromptOutputHistoryItemView>, CommandError> {
    desktop_app()
        .prompts()
        .list_prompt_output_history(imglab_core::ListPromptOutputHistoryRequest {
            library_path: input.library_path,
            prompt_version_id: input.prompt_version_id,
        })
        .map(|items| {
            items
                .into_iter()
                .map(prompt_output_history_item_view)
                .collect()
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn save_generation_prompt_as_prompt(
    input: SaveGenerationPromptAsPromptInput,
) -> Result<PromptVersionView, CommandError> {
    desktop_app()
        .prompts()
        .save_generation_prompt_as_prompt(imglab_core::SaveGenerationPromptAsPromptRequest {
            library_path: input.library_path,
            generation_event_id: input.generation_event_id,
            name: input.name,
        })
        .map(prompt_version_view)
        .map_err(Into::into)
}
