use crate::application::ports::{
    AssetRepository, GenerationEventRepository, ImageGenerationProvider, ManagedFileStore,
    MetadataSuggestionRepository,
};
use crate::{
    AssetId, AssetSummary, AssetVersionId, ConfidenceScoreView, CreateGenerationEventRequest,
    CreateMetadataSuggestionRequest, DomainError, DomainResult, GeneratedImage, GenerationEventId,
    GenerationEventSummary, GenerationOperation, GenerationParameters, GenerationResult,
    ManagedFileImport, ManagedFileMetadata, MetadataSuggestion, MetadataSuggestionId,
    PersistAssetVersionRequest, PersistImportedAssetRequest, PromoteAssetVersionRequest,
    PromoteAssetVersionSummary, VersionSummary,
};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct InMemoryAssetRepository {
    pub imported_paths: RefCell<Vec<PathBuf>>,
    pub reference_paths: RefCell<Vec<PathBuf>>,
    pub persisted_versions: RefCell<Vec<PersistAssetVersionRequest>>,
    pub generated_marks: RefCell<Vec<GenerationEventId>>,
}

impl AssetRepository for InMemoryAssetRepository {
    fn load_version(
        &self,
        _library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<VersionSummary> {
        Ok(VersionSummary {
            id: version_id.clone(),
            ..version_summary("asset-existing", &version_id.0)
        })
    }

    fn list_versions_for_asset(
        &self,
        _library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<VersionSummary>> {
        Ok(vec![version_summary(&asset_id.0, "version-existing")])
    }

    fn persist_imported_asset(
        &self,
        request: PersistImportedAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        if request.status == "reference" {
            self.reference_paths
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
        self.imported_paths
            .borrow_mut()
            .push(request.file.file_path.clone());
        Ok((
            asset_summary("asset-imported"),
            persisted_version_summary(
                "asset-imported",
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
        self.persisted_versions.borrow_mut().push(request.clone());
        let mut version = persisted_version_summary(
            &request.asset_id.0,
            request.version_id,
            request.file,
            request.version_number,
        );
        version.parent_version_id = request.parent_version_id;
        version.generation_event_id = request.generation_event_id;
        Ok(version)
    }

    fn promote_version_as_asset(
        &self,
        _request: PromoteAssetVersionRequest,
    ) -> DomainResult<PromoteAssetVersionSummary> {
        unimplemented!("promote workflow is covered by repository integration tests")
    }

    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        Ok(generation_event_summary(request, 1))
    }

    fn mark_version_generated(
        &self,
        _library_path: &Path,
        _asset_id: &AssetId,
        _version_id: &AssetVersionId,
        generation_event_id: &GenerationEventId,
    ) -> DomainResult<()> {
        self.generated_marks
            .borrow_mut()
            .push(generation_event_id.clone());
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryGenerationEventRepository {
    pub requests: RefCell<Vec<CreateGenerationEventRequest>>,
}

impl GenerationEventRepository for InMemoryGenerationEventRepository {
    fn record_generation_event(
        &self,
        request: CreateGenerationEventRequest,
    ) -> DomainResult<GenerationEventSummary> {
        let sequence = self.requests.borrow().len() + 1;
        self.requests.borrow_mut().push(request.clone());
        Ok(generation_event_summary(request, sequence))
    }
}

#[derive(Default)]
pub struct InMemoryMetadataSuggestionRepository {
    pub suggestions: RefCell<Vec<CreateMetadataSuggestionRequest>>,
}

impl MetadataSuggestionRepository for InMemoryMetadataSuggestionRepository {
    fn create_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        self.suggestions.borrow_mut().push(request.clone());
        Ok(MetadataSuggestion {
            id: MetadataSuggestionId(format!("suggestion-{}", self.suggestions.borrow().len())),
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
        _library_path: &Path,
        _library_id: &crate::LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        Ok(Vec::new())
    }

    fn accept(
        &self,
        _request: crate::ReviewMetadataSuggestionRequest,
    ) -> DomainResult<AssetSummary> {
        Ok(asset_summary("asset-imported"))
    }

    fn batch_accept(
        &self,
        _request: crate::BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>> {
        Ok(Vec::new())
    }

    fn reject(
        &self,
        _library_path: &Path,
        _suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()> {
        Ok(())
    }

    fn batch_reject(
        &self,
        _library_path: &Path,
        _suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()> {
        Ok(())
    }

    fn list_history(
        &self,
        _library_path: &Path,
        _asset_id: &AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        Ok(Vec::new())
    }

    fn get_review_draft_detail(
        &self,
        _library_path: &Path,
        _suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<crate::ReviewDraftDetailView> {
        Err(DomainError::Database {
            message: "review draft detail is not part of the in-memory test fixture".to_string(),
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

pub struct InMemoryFileStore {
    pub source_bytes: Vec<u8>,
    pub version_bytes: Vec<u8>,
    pub writes: RefCell<Vec<(String, Vec<u8>)>>,
}

impl Default for InMemoryFileStore {
    fn default() -> Self {
        Self {
            source_bytes: vec![9, 9, 9],
            version_bytes: vec![8, 8, 8],
            writes: RefCell::new(Vec::new()),
        }
    }
}

impl ManagedFileStore for InMemoryFileStore {
    fn read_source_bytes(&self, _source_path: &Path) -> DomainResult<Vec<u8>> {
        Ok(self.source_bytes.clone())
    }

    fn import_original(
        &self,
        _library_path: &Path,
        source_path: &Path,
        mime_type_override: Option<&str>,
    ) -> DomainResult<ManagedFileImport> {
        Ok(ManagedFileImport {
            version_id: AssetVersionId("managed-version".to_string()),
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
        _library_path: &Path,
        _version_id: &AssetVersionId,
    ) -> DomainResult<Vec<u8>> {
        Ok(self.version_bytes.clone())
    }

    fn write_generated_bytes(
        &self,
        _library_path: &Path,
        mime_type: &str,
        bytes: &[u8],
    ) -> DomainResult<PathBuf> {
        self.writes
            .borrow_mut()
            .push((mime_type.to_string(), bytes.to_vec()));
        Ok(PathBuf::from(format!(
            "/tmp/generated-{}.bin",
            self.writes.borrow().len()
        )))
    }
}

pub struct StaticImageProvider {
    pub fail: Option<DomainError>,
    pub supports_image_to_image: bool,
    pub calls: Cell<u32>,
}

impl Default for StaticImageProvider {
    fn default() -> Self {
        Self {
            fail: None,
            supports_image_to_image: true,
            calls: Cell::new(0),
        }
    }
}

impl ImageGenerationProvider for StaticImageProvider {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn supports_operation(&self, operation: GenerationOperation) -> bool {
        matches!(operation, GenerationOperation::TextToImage) || self.supports_image_to_image
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

impl StaticImageProvider {
    fn result(&self) -> DomainResult<GenerationResult> {
        self.calls.set(self.calls.get() + 1);
        if let Some(error) = &self.fail {
            return Err(error.clone());
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

fn generation_event_summary(
    request: CreateGenerationEventRequest,
    sequence: usize,
) -> GenerationEventSummary {
    GenerationEventSummary {
        id: GenerationEventId(format!("event-{sequence}")),
        asset_id: request.asset_id,
        output_version_id: request.output_version_id,
        provider: request.provider,
        provider_model: request.provider_model,
        operation_type: request.operation_type,
        prompt: request.prompt,
        parameters_json: request.parameters_json,
        status: request.status,
    }
}
