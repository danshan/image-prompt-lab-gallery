#[tauri::command]
fn create_suggestion(input: CreateSuggestionInput) -> Result<SuggestionView, CommandError> {
    service()
        .create_suggestion(CreateMetadataSuggestionRequest {
            library_path: input.library_path,
            asset_id: AssetId(input.asset_id),
            source: "desktop".to_string(),
            suggested_title: input.title,
            suggested_description: input.description,
            suggested_schema_prompt: input.schema_prompt,
            suggested_tags: input.tags,
            suggested_category: input.category,
            confidence_json: input.confidence_json.unwrap_or_else(|| "{}".to_string()),
        })
        .map(suggestion_view)
        .map_err(Into::into)
}

#[tauri::command]
fn list_pending_suggestions(library_path: PathBuf) -> Result<Vec<SuggestionView>, CommandError> {
    let service = service();
    let library = service.open_library(&library_path)?;
    service
        .list_pending(&library_path, &library.id)
        .map(|suggestions| suggestions.into_iter().map(suggestion_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn get_review_draft_detail(
    library_path: PathBuf,
    suggestion_id: String,
) -> Result<ReviewDraftDetailView, CommandError> {
    service()
        .get_review_draft_detail(&library_path, &MetadataSuggestionId(suggestion_id))
        .map(|detail| review_draft_detail_view(detail, &library_path))
        .map_err(Into::into)
}

#[tauri::command]
fn accept_suggestion(input: ReviewSuggestionInput) -> Result<AssetView, CommandError> {
    service()
        .accept(ReviewMetadataSuggestionRequest {
            library_path: input.library_path,
            suggestion_id: MetadataSuggestionId(input.suggestion_id),
            title: input.title,
            description: input.description,
            schema_prompt: input.schema_prompt,
            tags: input.tags,
            category: input.category,
        })
        .map(asset_view)
        .map_err(Into::into)
}

#[tauri::command]
fn batch_accept_suggestions(
    input: BatchReviewSuggestionsInput,
) -> Result<Vec<AssetView>, CommandError> {
    let library_path = input.library_path;
    service()
        .batch_accept(BatchReviewMetadataSuggestionRequest {
            library_path: library_path.clone(),
            suggestions: input
                .suggestions
                .into_iter()
                .map(|suggestion| ReviewMetadataSuggestionRequest {
                    library_path: library_path.clone(),
                    suggestion_id: MetadataSuggestionId(suggestion.suggestion_id),
                    title: suggestion.title,
                    description: suggestion.description,
                    schema_prompt: suggestion.schema_prompt,
                    tags: suggestion.tags,
                    category: suggestion.category,
                })
                .collect(),
        })
        .map(|assets| assets.into_iter().map(asset_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn reject_suggestion(input: RejectSuggestionInput) -> Result<(), CommandError> {
    service()
        .reject(
            &input.library_path,
            &MetadataSuggestionId(input.suggestion_id),
        )
        .map_err(Into::into)
}

#[tauri::command]
fn batch_reject_suggestions(input: BatchRejectSuggestionsInput) -> Result<(), CommandError> {
    let suggestion_ids = input
        .suggestion_ids
        .into_iter()
        .map(MetadataSuggestionId)
        .collect::<Vec<_>>();
    service()
        .batch_reject(&input.library_path, &suggestion_ids)
        .map_err(Into::into)
}

#[tauri::command]
fn list_suggestion_history(
    input: SuggestionHistoryInput,
) -> Result<Vec<SuggestionView>, CommandError> {
    service()
        .list_history(&input.library_path, &AssetId(input.asset_id))
        .map(|suggestions| suggestions.into_iter().map(suggestion_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
async fn regenerate_suggestion(
    input: GenerateReviewFieldInput,
) -> Result<SuggestionView, CommandError> {
    let library_path = input.library_path.clone();
    let asset_id = input.asset_id.clone();
    let context = input.context.clone();
    let title_input = GenerateReviewFieldInput {
        field: ReviewField::Title,
        context: context.clone(),
        ..input.clone()
    };
    let description_input = GenerateReviewFieldInput {
        field: ReviewField::Description,
        context: context.clone(),
        ..input.clone()
    };
    let schema_input = GenerateReviewFieldInput {
        field: ReviewField::SchemaPrompt,
        context,
        ..input
    };
    tauri::async_runtime::spawn_blocking(move || {
        let generator = CodexCliMetadataGenerator::new("codex", &library_path);
        let title = generator.generate(&title_input)?.value;
        let description = generator.generate(&description_input)?.value;
        let schema_prompt = generator.generate(&schema_input)?.value;
        service()
            .create_suggestion(CreateMetadataSuggestionRequest {
                library_path,
                asset_id: AssetId(asset_id),
                source: "codex_metadata".to_string(),
                suggested_title: Some(title),
                suggested_description: Some(description),
                suggested_schema_prompt: Some(schema_prompt),
                suggested_tags: title_input.context.tags,
                suggested_category: title_input.context.category,
                confidence_json: "{}".to_string(),
            })
            .map(suggestion_view)
            .map_err(Into::into)
    })
    .await
    .map_err(|error| CommandError {
        code: "MetadataGenerationFailed".to_string(),
        message: format!("metadata generation worker failed: {error}"),
        recoverable: true,
    })?
}

#[tauri::command]
async fn generate_review_field(
    input: GenerateReviewFieldInput,
) -> Result<GeneratedReviewFieldView, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        CodexCliMetadataGenerator::new("codex", &input.library_path).generate(&input)
    })
    .await
    .map_err(|error| CommandError {
        code: "MetadataGenerationFailed".to_string(),
        message: format!("metadata generation worker failed: {error}"),
        recoverable: true,
    })?
}
