use crate::application::ports::{
    AssetRepository, GenerationEventRepository, ImageGenerationProvider, ManagedFileStore,
    MetadataSuggestionRepository,
};
use crate::domain::asset::{ensure_same_asset_parent, next_version_number};
use crate::domain::generation::operation_to_str;
use crate::{
    AssetId, AssetSummary, AssetVersionId, CreateGenerationEventRequest,
    CreateMetadataSuggestionRequest, DomainError, DomainResult, GenerateImageRequest,
    GenerationOperation, GenerationParameters, GenerationResult, PersistAssetVersionRequest,
    PersistImportedAssetRequest, VersionSummary,
};
pub use crate::{GenerationRequestInput, PreparedGenerationRequest};
use serde_json::json;

pub struct GenerateImageUseCase<P, A, E, M, F> {
    provider: P,
    assets: A,
    events: E,
    metadata: M,
    files: F,
}

impl<P, A, E, M, F> GenerateImageUseCase<P, A, E, M, F> {
    pub fn new(provider: P, assets: A, events: E, metadata: M, files: F) -> Self {
        Self {
            provider,
            assets,
            events,
            metadata,
            files,
        }
    }
}

impl<P, A, E, M, F> GenerateImageUseCase<P, A, E, M, F>
where
    P: ImageGenerationProvider,
    A: AssetRepository,
    E: GenerationEventRepository,
    M: MetadataSuggestionRepository,
    F: ManagedFileStore,
{
    pub fn execute(&self, request: GenerateImageRequest) -> DomainResult<Vec<VersionSummary>> {
        self.ensure_provider_supports(request.parameters.operation)?;
        self.provider.validate_parameters(&request.parameters)?;

        let input = self.resolve_generation_input(&request)?;
        let result = self.call_provider(&request.parameters, input.bytes.as_deref());

        let result = match result {
            Ok(result) => result,
            Err(error) => {
                self.record_provider_failure(&request, input.reference.as_ref(), &error)?;
                return Err(error);
            }
        };

        self.persist_generated_images(
            request,
            input.output_parent_version_id,
            input.event_input_version_id,
            result,
        )
    }

    fn ensure_provider_supports(&self, operation: GenerationOperation) -> DomainResult<()> {
        if self.provider.supports_operation(operation) {
            return Ok(());
        }
        Err(DomainError::UnsupportedProviderCapability {
            provider: self.provider.name().to_string(),
            capability: operation_to_str(operation).to_string(),
        })
    }

    fn resolve_generation_input(
        &self,
        request: &GenerateImageRequest,
    ) -> DomainResult<ResolvedGenerationInput> {
        let mut parameters_input_version_id = request.parameters.input_version_id.clone();
        let mut reference = None;
        let mut bytes = request.input_bytes.clone();

        if matches!(
            request.parameters.operation,
            GenerationOperation::ImageToImage
        ) && parameters_input_version_id.is_none()
        {
            if let Some(input_file) = &request.input_file {
                let imported_file =
                    self.files
                        .import_original(&request.library_path, input_file, None)?;
                let imported = self
                    .assets
                    .persist_imported_asset(PersistImportedAssetRequest {
                        library_path: request.library_path.clone(),
                        version_id: imported_file.version_id,
                        file: imported_file.metadata,
                        status: "reference".to_string(),
                        version_number: 1,
                        version_label: "reference".to_string(),
                    })?;
                parameters_input_version_id = Some(imported.1.id.clone());
                bytes = Some(self.files.read_source_bytes(input_file)?);
                reference = Some(imported);
            }
        }

        if matches!(
            request.parameters.operation,
            GenerationOperation::ImageToImage
        ) && bytes.is_none()
        {
            if let Some(input_version_id) = &parameters_input_version_id {
                bytes = Some(
                    self.files
                        .read_version_bytes(&request.library_path, input_version_id)?,
                );
            }
        }

        Ok(ResolvedGenerationInput {
            bytes,
            reference,
            output_parent_version_id: request.parameters.input_version_id.clone(),
            event_input_version_id: parameters_input_version_id,
        })
    }

    fn call_provider(
        &self,
        parameters: &GenerationParameters,
        input: Option<&[u8]>,
    ) -> DomainResult<GenerationResult> {
        match parameters.operation {
            GenerationOperation::TextToImage => self.provider.generate_from_text(parameters),
            GenerationOperation::ImageToImage => {
                let input = input.ok_or_else(|| DomainError::InvalidGenerationParameters {
                    message: "image-to-image generation requires input bytes".to_string(),
                })?;
                self.provider.generate_from_image(parameters, input)
            }
        }
    }

    fn record_provider_failure(
        &self,
        request: &GenerateImageRequest,
        reference: Option<&(AssetSummary, VersionSummary)>,
        error: &DomainError,
    ) -> DomainResult<()> {
        if let Some((reference_asset, reference_version)) = reference {
            self.events
                .record_generation_event(generation_event_request(
                    request,
                    GenerationEventDetails {
                        asset_id: Some(reference_asset.id.clone()),
                        output_version_id: None,
                        input_asset_version_id: Some(reference_version.id.clone()),
                        raw_request_json: None,
                        raw_response_json: None,
                        status: "failed",
                        error_code: Some(error.code().to_string()),
                        error_message: Some(error.to_string()),
                    },
                ))?;
        }
        Ok(())
    }

    fn persist_generated_images(
        &self,
        request: GenerateImageRequest,
        parent_version_id: Option<AssetVersionId>,
        event_input_version_id: Option<AssetVersionId>,
        result: GenerationResult,
    ) -> DomainResult<Vec<VersionSummary>> {
        let mut versions = Vec::new();
        let raw_request_json = Some(result.raw_request_json.clone());
        let raw_response_json = Some(result.raw_response_json.clone());

        for image in result.images {
            let source_path = self.files.write_generated_bytes(
                &request.library_path,
                &image.mime_type,
                &image.bytes,
            )?;
            let imported_file = self.files.import_original(
                &request.library_path,
                &source_path,
                Some(&image.mime_type),
            )?;
            let version = self.persist_single_output(
                &request,
                parent_version_id.clone(),
                event_input_version_id.clone(),
                imported_file,
                raw_request_json.clone(),
                raw_response_json.clone(),
            )?;
            self.create_generation_metadata_suggestion(&request, &version.asset_id)?;
            versions.push(version);
        }

        Ok(versions)
    }

    fn persist_single_output(
        &self,
        request: &GenerateImageRequest,
        parent_version_id: Option<AssetVersionId>,
        event_input_version_id: Option<AssetVersionId>,
        imported_file: crate::ManagedFileImport,
        raw_request_json: Option<String>,
        raw_response_json: Option<String>,
    ) -> DomainResult<VersionSummary> {
        match parent_version_id {
            Some(parent_version_id) => self.persist_child_output(
                request,
                parent_version_id,
                event_input_version_id,
                imported_file,
                raw_request_json,
                raw_response_json,
            ),
            None => self.persist_new_asset_output(
                request,
                event_input_version_id,
                imported_file,
                raw_request_json,
                raw_response_json,
            ),
        }
    }

    fn persist_child_output(
        &self,
        request: &GenerateImageRequest,
        parent_version_id: AssetVersionId,
        event_input_version_id: Option<AssetVersionId>,
        imported_file: crate::ManagedFileImport,
        raw_request_json: Option<String>,
        raw_response_json: Option<String>,
    ) -> DomainResult<VersionSummary> {
        let parent = self
            .assets
            .load_version(&request.library_path, &parent_version_id)?;
        let asset_id = parent.asset_id.clone();
        ensure_same_asset_parent(&asset_id, &parent.asset_id)?;
        let versions = self
            .assets
            .list_versions_for_asset(&request.library_path, &asset_id)?;
        let current_max = versions.iter().map(|version| version.version_number).max();
        let event = self
            .events
            .record_generation_event(generation_event_request(
                request,
                GenerationEventDetails {
                    asset_id: Some(asset_id.clone()),
                    output_version_id: None,
                    input_asset_version_id: event_input_version_id,
                    raw_request_json,
                    raw_response_json,
                    status: "completed",
                    error_code: None,
                    error_message: None,
                },
            ))?;
        self.assets
            .persist_asset_version(PersistAssetVersionRequest {
                library_path: request.library_path.clone(),
                asset_id,
                parent_version_id: Some(parent_version_id),
                generation_event_id: Some(event.id),
                version_id: imported_file.version_id,
                file: imported_file.metadata,
                version_number: next_version_number(current_max),
                version_label: Some("generated".to_string()),
            })
    }

    fn persist_new_asset_output(
        &self,
        request: &GenerateImageRequest,
        event_input_version_id: Option<AssetVersionId>,
        imported_file: crate::ManagedFileImport,
        raw_request_json: Option<String>,
        raw_response_json: Option<String>,
    ) -> DomainResult<VersionSummary> {
        let (asset, mut version) =
            self.assets
                .persist_imported_asset(PersistImportedAssetRequest {
                    library_path: request.library_path.clone(),
                    version_id: imported_file.version_id,
                    file: imported_file.metadata,
                    status: "imported".to_string(),
                    version_number: 1,
                    version_label: "import".to_string(),
                })?;
        let event = self
            .events
            .record_generation_event(generation_event_request(
                request,
                GenerationEventDetails {
                    asset_id: Some(asset.id.clone()),
                    output_version_id: Some(version.id.clone()),
                    input_asset_version_id: event_input_version_id,
                    raw_request_json,
                    raw_response_json,
                    status: "completed",
                    error_code: None,
                    error_message: None,
                },
            ))?;
        self.assets.mark_version_generated(
            &request.library_path,
            &asset.id,
            &version.id,
            &event.id,
        )?;
        version.generation_event_id = Some(event.id);
        Ok(version)
    }

    fn create_generation_metadata_suggestion(
        &self,
        request: &GenerateImageRequest,
        asset_id: &AssetId,
    ) -> DomainResult<()> {
        self.metadata
            .create_suggestion(CreateMetadataSuggestionRequest {
                library_path: request.library_path.clone(),
                asset_id: asset_id.clone(),
                source: format!("generation:{}", request.parameters.provider),
                suggested_title: default_title_from_prompt(&request.parameters.prompt),
                suggested_description: None,
                suggested_schema_prompt: None,
                suggested_tags: Vec::new(),
                suggested_category: None,
                confidence_json: generation_confidence_json(&request.parameters),
            })?;
        Ok(())
    }
}

struct ResolvedGenerationInput {
    bytes: Option<Vec<u8>>,
    reference: Option<(AssetSummary, VersionSummary)>,
    output_parent_version_id: Option<AssetVersionId>,
    event_input_version_id: Option<AssetVersionId>,
}

struct GenerationEventDetails {
    asset_id: Option<AssetId>,
    output_version_id: Option<AssetVersionId>,
    input_asset_version_id: Option<AssetVersionId>,
    raw_request_json: Option<String>,
    raw_response_json: Option<String>,
    status: &'static str,
    error_code: Option<String>,
    error_message: Option<String>,
}

fn generation_event_request(
    request: &GenerateImageRequest,
    details: GenerationEventDetails,
) -> CreateGenerationEventRequest {
    CreateGenerationEventRequest {
        library_path: request.library_path.clone(),
        asset_id: details.asset_id,
        output_version_id: details.output_version_id,
        provider: request.parameters.provider.clone(),
        provider_model: request.parameters.model.clone(),
        operation_type: request.parameters.operation,
        prompt: request.parameters.prompt.clone(),
        negative_prompt: request.parameters.negative_prompt.clone(),
        input_asset_version_id: details.input_asset_version_id,
        prompt_version_id: request.parameters.prompt_version_id.clone(),
        parameters_json: request.parameters.parameters_json.clone(),
        raw_request_json: details.raw_request_json,
        raw_response_json: details.raw_response_json,
        status: details.status.to_string(),
        error_code: details.error_code,
        error_message: details.error_message,
    }
}

fn default_title_from_prompt(prompt: &str) -> Option<String> {
    let title: String = prompt.chars().take(80).collect();
    let title = title.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn generation_confidence_json(parameters: &GenerationParameters) -> String {
    json!({
        "source": "generation",
        "provider": parameters.provider,
        "model": parameters.model,
        "operation": operation_to_str(parameters.operation),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AddAssetTagRequest, ConfidenceScoreView, GeneratedImage, GenerationEventId,
        GenerationEventSummary, ManagedFileImport, ManagedFileMetadata, MetadataSuggestion,
        MetadataSuggestionId, PromoteAssetVersionRequest, PromoteAssetVersionSummary,
    };
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(Default)]
    struct FakeAssets {
        imported: RefCell<Vec<PathBuf>>,
        references: RefCell<Vec<PathBuf>>,
        persisted_versions: RefCell<Vec<PersistAssetVersionRequest>>,
        generated_marks: RefCell<Vec<GenerationEventId>>,
    }

    impl AssetRepository for FakeAssets {
        fn load_version(
            &self,
            _library_path: &std::path::Path,
            version_id: &AssetVersionId,
        ) -> DomainResult<VersionSummary> {
            Ok(VersionSummary {
                id: version_id.clone(),
                ..version_summary("asset-parent", "version-parent")
            })
        }

        fn list_versions_for_asset(
            &self,
            _library_path: &std::path::Path,
            asset_id: &AssetId,
        ) -> DomainResult<Vec<VersionSummary>> {
            Ok(vec![version_summary(&asset_id.0, "version-parent")])
        }

        fn persist_imported_asset(
            &self,
            request: PersistImportedAssetRequest,
        ) -> DomainResult<(AssetSummary, VersionSummary)> {
            if request.status == "reference" {
                self.references
                    .borrow_mut()
                    .push(request.file.file_path.clone());
                return Ok((
                    asset_summary("asset-reference"),
                    persisted_version_summary(
                        "asset-reference",
                        request.version_id,
                        request.file,
                        request.version_number,
                    ),
                ));
            }
            self.imported
                .borrow_mut()
                .push(request.file.file_path.clone());
            Ok((
                asset_summary("asset-new"),
                persisted_version_summary(
                    "asset-new",
                    request.version_id,
                    request.file,
                    request.version_number,
                ),
            ))
        }

        fn persist_asset_version(
            &self,
            request: PersistAssetVersionRequest,
        ) -> DomainResult<VersionSummary> {
            let mut version = persisted_version_summary(
                &request.asset_id.0,
                request.version_id.clone(),
                request.file.clone(),
                request.version_number,
            );
            version.parent_version_id = request.parent_version_id.clone();
            version.generation_event_id = request.generation_event_id.clone();
            self.persisted_versions.borrow_mut().push(request);
            Ok(version)
        }

        fn promote_version_as_asset(
            &self,
            _request: PromoteAssetVersionRequest,
        ) -> DomainResult<PromoteAssetVersionSummary> {
            unimplemented!("generation use case does not promote versions")
        }

        fn record_generation_event(
            &self,
            _request: CreateGenerationEventRequest,
        ) -> DomainResult<GenerationEventSummary> {
            unreachable!("generation use case writes events through GenerationEventRepository")
        }

        fn mark_version_generated(
            &self,
            _library_path: &std::path::Path,
            _asset_id: &AssetId,
            _version_id: &AssetVersionId,
            generation_event_id: &GenerationEventId,
        ) -> DomainResult<()> {
            self.generated_marks
                .borrow_mut()
                .push(generation_event_id.clone());
            Ok(())
        }

        fn add_tag_to_asset(&self, _request: AddAssetTagRequest) -> DomainResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeEvents {
        requests: RefCell<Vec<CreateGenerationEventRequest>>,
    }

    impl GenerationEventRepository for FakeEvents {
        fn record_generation_event(
            &self,
            request: CreateGenerationEventRequest,
        ) -> DomainResult<GenerationEventSummary> {
            self.requests.borrow_mut().push(request.clone());
            Ok(GenerationEventSummary {
                id: GenerationEventId(format!("event-{}", self.requests.borrow().len())),
                asset_id: request.asset_id,
                output_version_id: request.output_version_id,
                provider: request.provider,
                provider_model: request.provider_model,
                operation_type: request.operation_type,
                prompt: request.prompt,
                prompt_version_id: request.prompt_version_id,
                parameters_json: request.parameters_json,
                status: request.status,
            })
        }
    }

    #[derive(Default)]
    struct FakeMetadata {
        suggestions: RefCell<Vec<CreateMetadataSuggestionRequest>>,
    }

    impl MetadataSuggestionRepository for FakeMetadata {
        fn create_suggestion(
            &self,
            request: CreateMetadataSuggestionRequest,
        ) -> DomainResult<MetadataSuggestion> {
            self.suggestions.borrow_mut().push(request.clone());
            Ok(MetadataSuggestion {
                id: MetadataSuggestionId("suggestion-1".to_string()),
                asset_id: request.asset_id,
                suggested_title: request.suggested_title,
                suggested_description: request.suggested_description,
                suggested_schema_prompt: request.suggested_schema_prompt,
                suggested_tags: request.suggested_tags,
                suggested_category: request.suggested_category,
                confidence_json: request.confidence_json,
                status: "pending_review".to_string(),
                created_at: None,
                reviewed_at: None,
            })
        }

        fn list_pending(
            &self,
            _library_path: &std::path::Path,
            _library_id: &crate::LibraryId,
        ) -> DomainResult<Vec<MetadataSuggestion>> {
            Ok(Vec::new())
        }

        fn accept(
            &self,
            _request: crate::ReviewMetadataSuggestionRequest,
        ) -> DomainResult<AssetSummary> {
            Ok(asset_summary("asset-new"))
        }

        fn batch_accept(
            &self,
            _request: crate::BatchReviewMetadataSuggestionRequest,
        ) -> DomainResult<Vec<AssetSummary>> {
            Ok(Vec::new())
        }

        fn reject(
            &self,
            _library_path: &std::path::Path,
            _suggestion_id: &MetadataSuggestionId,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn batch_reject(
            &self,
            _library_path: &std::path::Path,
            _suggestion_ids: &[MetadataSuggestionId],
        ) -> DomainResult<()> {
            Ok(())
        }

        fn list_history(
            &self,
            _library_path: &std::path::Path,
            _asset_id: &AssetId,
        ) -> DomainResult<Vec<MetadataSuggestion>> {
            Ok(Vec::new())
        }

        fn get_review_draft_detail(
            &self,
            _library_path: &std::path::Path,
            _suggestion_id: &MetadataSuggestionId,
        ) -> DomainResult<crate::ReviewDraftDetailView> {
            Err(DomainError::Database {
                message: "not needed in generation use case tests".to_string(),
            })
        }

        fn normalize_confidence(&self, _confidence_json: &str) -> ConfidenceScoreView {
            ConfidenceScoreView {
                overall: None,
                title: None,
                description: None,
                schema_prompt: None,
                tags: None,
                category: None,
            }
        }
    }

    struct FakeFiles;

    impl ManagedFileStore for FakeFiles {
        fn read_source_bytes(&self, _source_path: &std::path::Path) -> DomainResult<Vec<u8>> {
            Ok(vec![9, 9, 9])
        }

        fn import_original(
            &self,
            _library_path: &std::path::Path,
            source_path: &std::path::Path,
            mime_type_override: Option<&str>,
        ) -> DomainResult<ManagedFileImport> {
            Ok(ManagedFileImport {
                version_id: AssetVersionId(format!(
                    "managed-version-{}",
                    source_path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("file")
                )),
                metadata: ManagedFileMetadata {
                    file_path: source_path.to_path_buf(),
                    checksum_algorithm: "SHA-256".to_string(),
                    checksum: "abc".to_string(),
                    width: None,
                    height: None,
                    mime_type: mime_type_override.unwrap_or("image/png").to_string(),
                },
            })
        }

        fn read_version_bytes(
            &self,
            _library_path: &std::path::Path,
            _version_id: &AssetVersionId,
        ) -> DomainResult<Vec<u8>> {
            Ok(vec![8, 8, 8])
        }

        fn write_generated_bytes(
            &self,
            _library_path: &std::path::Path,
            mime_type: &str,
            _bytes: &[u8],
        ) -> DomainResult<PathBuf> {
            let extension = if mime_type == "image/png" {
                "png"
            } else {
                "bin"
            };
            Ok(PathBuf::from(format!("/tmp/generated-output.{extension}")))
        }
    }

    struct FakeProvider {
        fail: bool,
    }

    impl ImageGenerationProvider for FakeProvider {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn supports_operation(&self, _operation: GenerationOperation) -> bool {
            true
        }

        fn validate_parameters(&self, _parameters: &GenerationParameters) -> DomainResult<()> {
            Ok(())
        }

        fn generate_from_text(
            &self,
            _parameters: &GenerationParameters,
        ) -> DomainResult<GenerationResult> {
            self.result()
        }

        fn generate_from_image(
            &self,
            _parameters: &GenerationParameters,
            _input: &[u8],
        ) -> DomainResult<GenerationResult> {
            self.result()
        }
    }

    impl FakeProvider {
        fn result(&self) -> DomainResult<GenerationResult> {
            if self.fail {
                return Err(DomainError::GenerationFailed {
                    provider: "fake".to_string(),
                    message: "boom".to_string(),
                });
            }
            Ok(GenerationResult {
                images: vec![GeneratedImage {
                    bytes: vec![1, 2, 3],
                    mime_type: "image/png".to_string(),
                    provider_metadata_json: "{}".to_string(),
                }],
                raw_request_json: "{\"request\":true}".to_string(),
                raw_response_json: "{\"response\":true}".to_string(),
            })
        }
    }

    fn text_request() -> GenerateImageRequest {
        GenerateImageRequest {
            library_path: PathBuf::from("/tmp/library"),
            parameters: GenerationParameters {
                library_path: Some(PathBuf::from("/tmp/library")),
                provider: "fake".to_string(),
                model: "fake-image".to_string(),
                prompt: "A compact test image".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::TextToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
            input_file: None,
            input_bytes: None,
        }
    }

    fn image_request_with_upload() -> GenerateImageRequest {
        let mut request = text_request();
        request.parameters.operation = GenerationOperation::ImageToImage;
        request.input_file = Some(PathBuf::from("/tmp/reference.png"));
        request
    }

    fn asset_summary(id: &str) -> AssetSummary {
        AssetSummary {
            id: AssetId(id.to_string()),
            title: None,
            category: None,
            rating: None,
            status: "generated".to_string(),
        }
    }

    fn version_summary(asset_id: &str, version_id: &str) -> VersionSummary {
        VersionSummary {
            id: AssetVersionId(version_id.to_string()),
            asset_id: AssetId(asset_id.to_string()),
            parent_version_id: None,
            generation_event_id: None,
            version_number: 1,
            version_name: crate::version_name(1),
            file_path: PathBuf::from("generated/output.png"),
            checksum_algorithm: "SHA-256".to_string(),
            checksum: "abc".to_string(),
            mime_type: "image/png".to_string(),
        }
    }

    fn persisted_version_summary(
        asset_id: &str,
        version_id: AssetVersionId,
        file: ManagedFileMetadata,
        version_number: u32,
    ) -> VersionSummary {
        VersionSummary {
            id: version_id,
            asset_id: AssetId(asset_id.to_string()),
            parent_version_id: None,
            generation_event_id: None,
            version_number,
            version_name: crate::version_name(version_number),
            file_path: file.file_path,
            checksum_algorithm: file.checksum_algorithm,
            checksum: file.checksum,
            mime_type: file.mime_type,
        }
    }

    #[test]
    fn text_to_image_records_event_suggestion_and_generated_version() {
        let use_case = GenerateImageUseCase::new(
            FakeProvider { fail: false },
            FakeAssets::default(),
            FakeEvents::default(),
            FakeMetadata::default(),
            FakeFiles,
        );

        let versions = use_case.execute(text_request()).expect("generate");

        assert_eq!(versions.len(), 1);
        assert_eq!(use_case.events.requests.borrow()[0].status, "completed");
        assert_eq!(use_case.metadata.suggestions.borrow().len(), 1);
        assert_eq!(use_case.assets.generated_marks.borrow().len(), 1);
    }

    #[test]
    fn uploaded_reference_failure_records_failed_reference_event() {
        let use_case = GenerateImageUseCase::new(
            FakeProvider { fail: true },
            FakeAssets::default(),
            FakeEvents::default(),
            FakeMetadata::default(),
            FakeFiles,
        );

        let error = use_case
            .execute(image_request_with_upload())
            .expect_err("generation should fail");

        assert!(matches!(error, DomainError::GenerationFailed { .. }));
        assert_eq!(use_case.assets.references.borrow().len(), 1);
        let event = &use_case.events.requests.borrow()[0];
        assert_eq!(event.status, "failed");
        assert_eq!(
            event.input_asset_version_id,
            Some(AssetVersionId("managed-version-reference".to_string()))
        );
    }
}
