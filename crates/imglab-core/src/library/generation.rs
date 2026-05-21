use super::{
    assets::{
        default_title_from_prompt, import_asset_with_status, load_version,
        mark_imported_version_as_generated,
    },
    io_error, serialization_error,
    storage::extension_for_mime_type,
    LocalLibraryService,
};
pub use crate::domain::generation::normalize_provider_name;
use crate::domain::generation::{
    default_generation_model_label, infer_generation_operation, operation_to_str,
};
use crate::{
    AssetId, AssetService, AssetSummary, AssetVersionId, CreateChildVersionRequest,
    CreateGenerationEventRequest, CreateMetadataSuggestionRequest, DomainError, DomainResult,
    GenerateImageRequest, GenerationOperation, GenerationParameters, GenerationRequestInput,
    ImageProvider, ImportAssetRequest, MetadataReviewService, PreparedGenerationRequest,
    VersionSummary,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct LocalGenerationService<P> {
    provider: P,
}

impl<P> LocalGenerationService<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P> crate::GenerationService for LocalGenerationService<P>
where
    P: ImageProvider,
{
    fn generate(&self, request: GenerateImageRequest) -> DomainResult<Vec<VersionSummary>> {
        let library_service =
            LocalLibraryService::new(std::env::temp_dir().join("imglab-unused-registry.sqlite"));
        if !self
            .provider
            .supports_operation(request.parameters.operation)
        {
            return Err(DomainError::UnsupportedProviderCapability {
                provider: self.provider.name().to_string(),
                capability: operation_to_str(request.parameters.operation).to_string(),
            });
        }
        self.provider.validate_parameters(&request.parameters)?;

        let mut parameters = request.parameters.clone();
        let mut input_bytes = request.input_bytes.clone();
        let mut uploaded_reference: Option<(AssetSummary, VersionSummary)> = None;
        if matches!(parameters.operation, GenerationOperation::ImageToImage)
            && parameters.input_version_id.is_none()
        {
            if let Some(input_file) = &request.input_file {
                let imported_file = crate::application::ports::ManagedFileStore::import_original(
                    &library_service,
                    &request.library_path,
                    input_file,
                    None,
                )?;
                let reference = import_asset_with_status(
                    &library_service,
                    crate::PersistImportedAssetRequest {
                        library_path: request.library_path.clone(),
                        version_id: imported_file.version_id,
                        file: imported_file.metadata,
                        status: "reference".to_string(),
                        version_number: 1,
                        version_label: "reference".to_string(),
                    },
                )?;
                let reference_path = request.library_path.join(&reference.1.file_path);
                input_bytes = Some(fs::read(&reference_path).map_err(|error| DomainError::Io {
                    path: reference_path.display().to_string(),
                    message: error.to_string(),
                })?);
                parameters.input_version_id = Some(reference.1.id.clone());
                uploaded_reference = Some(reference);
            }
        }
        if matches!(parameters.operation, GenerationOperation::ImageToImage)
            && input_bytes.is_none()
        {
            if let Some(input_version_id) = &parameters.input_version_id {
                let connection = LocalLibraryService::open_library_database(&request.library_path)?;
                let input_version = load_version(&connection, input_version_id)?;
                let input_path = request.library_path.join(input_version.file_path);
                input_bytes = Some(fs::read(&input_path).map_err(|error| DomainError::Io {
                    path: input_path.display().to_string(),
                    message: error.to_string(),
                })?);
            }
        }

        let result = match parameters.operation {
            GenerationOperation::TextToImage => self.provider.generate_from_text(&parameters),
            GenerationOperation::ImageToImage => {
                let input = input_bytes.as_deref().ok_or_else(|| {
                    DomainError::InvalidGenerationParameters {
                        message: "image-to-image generation requires input bytes".to_string(),
                    }
                })?;
                self.provider.generate_from_image(&parameters, input)
            }
        }
        .inspect_err(|error| {
            if let Some((reference_asset, reference_version)) = &uploaded_reference {
                let _ = library_service.record_generation_event(CreateGenerationEventRequest {
                    library_path: request.library_path.clone(),
                    asset_id: Some(reference_asset.id.clone()),
                    output_version_id: None,
                    provider: parameters.provider.clone(),
                    provider_model: parameters.model.clone(),
                    operation_type: parameters.operation,
                    prompt: parameters.prompt.clone(),
                    negative_prompt: parameters.negative_prompt.clone(),
                    input_asset_version_id: Some(reference_version.id.clone()),
                    parameters_json: parameters.parameters_json.clone(),
                    raw_request_json: None,
                    raw_response_json: None,
                    status: "failed".to_string(),
                    error_code: Some(error.code().to_string()),
                    error_message: Some(error.to_string()),
                });
            }
        })?;

        let mut versions = Vec::new();
        let mut asset_id = None;
        let parent_version_id = request.parameters.input_version_id.clone();
        let input_asset_version_id = parameters.input_version_id.clone();

        if let Some(parent_id) = &parent_version_id {
            let connection = LocalLibraryService::open_library_database(&request.library_path)?;
            let parent = load_version(&connection, parent_id)?;
            asset_id = Some(parent.asset_id);
        }

        for image in result.images {
            let temp_dir =
                std::env::temp_dir().join(format!("imglab-generated-{}", Uuid::new_v4()));
            fs::create_dir_all(&temp_dir).map_err(|error| io_error(&temp_dir, error))?;
            let extension = extension_for_mime_type(&image.mime_type);
            let temp_file = temp_dir.join(format!("output.{extension}"));
            fs::write(&temp_file, &image.bytes).map_err(|error| io_error(&temp_file, error))?;

            let version = if let (Some(asset_id), Some(parent_version_id)) =
                (asset_id.clone(), parent_version_id.clone())
            {
                let event =
                    library_service.record_generation_event(CreateGenerationEventRequest {
                        library_path: request.library_path.clone(),
                        asset_id: Some(asset_id.clone()),
                        output_version_id: None,
                        provider: parameters.provider.clone(),
                        provider_model: parameters.model.clone(),
                        operation_type: parameters.operation,
                        prompt: parameters.prompt.clone(),
                        negative_prompt: parameters.negative_prompt.clone(),
                        input_asset_version_id: Some(parent_version_id.clone()),
                        parameters_json: parameters.parameters_json.clone(),
                        raw_request_json: Some(result.raw_request_json.clone()),
                        raw_response_json: Some(result.raw_response_json.clone()),
                        status: "completed".to_string(),
                        error_code: None,
                        error_message: None,
                    })?;
                library_service.create_child_version(CreateChildVersionRequest {
                    library_path: request.library_path.clone(),
                    asset_id,
                    parent_version_id,
                    generation_event_id: Some(event.id),
                    source_path: temp_file,
                    mime_type: image.mime_type,
                    version_label: Some("generated".to_string()),
                })?
            } else {
                let (asset, mut version) = library_service.import_asset(ImportAssetRequest {
                    library_path: request.library_path.clone(),
                    source_path: temp_file,
                })?;
                let event =
                    library_service.record_generation_event(CreateGenerationEventRequest {
                        library_path: request.library_path.clone(),
                        asset_id: Some(asset.id.clone()),
                        output_version_id: Some(version.id.clone()),
                        provider: parameters.provider.clone(),
                        provider_model: parameters.model.clone(),
                        operation_type: parameters.operation,
                        prompt: parameters.prompt.clone(),
                        negative_prompt: parameters.negative_prompt.clone(),
                        input_asset_version_id: input_asset_version_id.clone(),
                        parameters_json: parameters.parameters_json.clone(),
                        raw_request_json: Some(result.raw_request_json.clone()),
                        raw_response_json: Some(result.raw_response_json.clone()),
                        status: "completed".to_string(),
                        error_code: None,
                        error_message: None,
                    })?;
                mark_imported_version_as_generated(
                    &request.library_path,
                    &asset.id,
                    &version.id,
                    &event.id,
                )?;
                version.generation_event_id = Some(event.id);
                version
            };

            create_generation_metadata_suggestion(&library_service, &request, &version.asset_id)?;
            versions.push(version);
        }

        Ok(versions)
    }
}

fn create_generation_metadata_suggestion(
    library_service: &LocalLibraryService,
    request: &GenerateImageRequest,
    asset_id: &AssetId,
) -> DomainResult<()> {
    library_service.create_suggestion(CreateMetadataSuggestionRequest {
        library_path: request.library_path.clone(),
        asset_id: asset_id.clone(),
        source: format!("generation:{}", request.parameters.provider),
        suggested_title: default_title_from_prompt(&request.parameters.prompt),
        suggested_description: None,
        suggested_schema_prompt: Some(schema_prompt_draft_from_generation(&request.parameters)?),
        suggested_tags: Vec::new(),
        suggested_category: None,
        confidence_json: json!({
            "source": "generation",
            "provider": request.parameters.provider,
            "model": request.parameters.model,
            "operation": operation_to_str(request.parameters.operation),
        })
        .to_string(),
    })?;
    Ok(())
}

fn schema_prompt_draft_from_generation(parameters: &GenerationParameters) -> DomainResult<String> {
    let aspect_ratio = generation_parameter_value(&parameters.parameters_json, "aspect_ratio")
        .or_else(|| generation_parameter_value(&parameters.parameters_json, "aspectRatio"))
        .unwrap_or_else(|| "unspecified".to_string());
    let schema = json!({
        "GLOBAL_SETTINGS": {
            "aspect_ratio": aspect_ratio,
            "style": "derived from source prompt",
            "clarity": "sharp foreground, readable subject detail",
            "render_flags": ["sharp_foreground", "micro_texture", "editorial_finish"]
        },
        "ENVIRONMENT": {
            "background": "preserve the generated image environment cues",
            "lighting": "preserve the generated image lighting direction and contrast",
            "atmosphere": ["match the final image mood", "avoid unsupported scene changes"]
        },
        "CORE_ASSETS": {
            "primary_subject": parameters.prompt,
            "materials": ["infer visible materials from final image"],
            "composition": "preserve the generated composition and camera framing"
        },
        "MOTION_OR_DETAIL_SYSTEMS": [
            {
                "object": "visible detail systems",
                "state": "preserve the generated image behavior and placement"
            }
        ],
        "OUTPUT": {
            "mood": "match the accepted visual direction",
            "avoid": ["cheap e-commerce banner", "plastic CGI", "fake brand logos"]
        }
    });
    let body = serde_json::to_string_pretty(&schema).map_err(serialization_error)?;
    Ok(format!(
        "// VERSION: 0.1\n// AESTHETIC: derived from generation prompt\n{body}"
    ))
}

fn generation_parameter_value(parameters_json: &str, key: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(parameters_json).ok()?;
    value.get(key).and_then(|value| match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

pub fn prepare_generation_request(
    input: GenerationRequestInput,
) -> DomainResult<PreparedGenerationRequest> {
    let provider = normalize_provider_name(&input.provider)?;
    let operation = input.operation.unwrap_or_else(|| {
        infer_generation_operation(input.input_file.as_ref(), input.input_version_id.as_ref())
    });
    let input_bytes = load_generation_input_bytes(
        &input.library_path,
        input.input_file.as_ref(),
        input.input_version_id.as_ref(),
    )?;
    let parameters = GenerationParameters {
        library_path: Some(input.library_path.clone()),
        provider: provider.clone(),
        model: input
            .model
            .unwrap_or_else(|| default_generation_model_label(&provider).to_string()),
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        operation,
        input_version_id: input.input_version_id,
        parameters_json: input.parameters_json.unwrap_or_else(|| "{}".to_string()),
    };

    Ok(PreparedGenerationRequest {
        provider,
        request: GenerateImageRequest {
            library_path: input.library_path,
            parameters,
            input_file: input.input_file,
            input_bytes,
        },
    })
}

fn load_generation_input_bytes(
    library_path: &Path,
    input_file: Option<&PathBuf>,
    input_version_id: Option<&AssetVersionId>,
) -> DomainResult<Option<Vec<u8>>> {
    if let Some(path) = input_file {
        return fs::read(path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    if let Some(version_id) = input_version_id {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let version = load_version(&connection, version_id)?;
        let path = library_path.join(version.file_path);
        return fs::read(&path).map(Some).map_err(|error| DomainError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        });
    }

    Ok(None)
}
