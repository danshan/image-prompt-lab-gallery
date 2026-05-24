use crate::application::ports::{AssetRepository, ManagedFileStore};
use crate::domain::asset::{ensure_same_asset_parent, next_version_number};
use crate::{
    AddAssetTagRequest, AssetSummary, CreateChildVersionRequest, DomainResult, ImportAssetRequest,
    PersistAssetVersionRequest, PersistImportedAssetRequest, PromoteAssetVersionRequest,
    PromoteAssetVersionSummary, VersionSummary,
};

pub struct AssetUseCase<R, F> {
    repository: R,
    files: F,
}

impl<R, F> AssetUseCase<R, F> {
    pub fn new(repository: R, files: F) -> Self {
        Self { repository, files }
    }
}

impl<R, F> AssetUseCase<R, F>
where
    R: AssetRepository + Clone,
    F: ManagedFileStore + Clone,
{
    pub fn import_asset(
        &self,
        request: ImportAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        ImportAssetUseCase::new(self.repository.clone(), self.files.clone()).execute(request)
    }

    pub fn create_child_version(
        &self,
        request: CreateChildVersionRequest,
    ) -> DomainResult<VersionSummary> {
        CreateChildVersionUseCase::new(self.repository.clone(), self.files.clone()).execute(request)
    }

    pub fn promote_version_as_asset(
        &self,
        request: PromoteAssetVersionRequest,
    ) -> DomainResult<PromoteAssetVersionSummary> {
        self.repository.promote_version_as_asset(request)
    }
}

impl<R, F> AssetUseCase<R, F>
where
    R: AssetRepository,
{
    pub fn add_tag(&self, request: AddAssetTagRequest) -> DomainResult<()> {
        self.repository.add_tag_to_asset(request)
    }
}

pub struct ImportAssetUseCase<R, F> {
    repository: R,
    files: F,
}

impl<R, F> ImportAssetUseCase<R, F> {
    pub fn new(repository: R, files: F) -> Self {
        Self { repository, files }
    }
}

impl<R, F> ImportAssetUseCase<R, F>
where
    R: AssetRepository,
    F: ManagedFileStore,
{
    pub fn execute(
        &self,
        request: ImportAssetRequest,
    ) -> DomainResult<(AssetSummary, VersionSummary)> {
        let imported =
            self.files
                .import_original(&request.library_path, &request.source_path, None)?;
        self.repository
            .persist_imported_asset(PersistImportedAssetRequest {
                library_path: request.library_path,
                version_id: imported.version_id,
                file: imported.metadata,
                status: "imported".to_string(),
                version_number: 1,
                version_label: "import".to_string(),
            })
    }
}

pub struct CreateChildVersionUseCase<R, F> {
    repository: R,
    files: F,
}

impl<R, F> CreateChildVersionUseCase<R, F> {
    pub fn new(repository: R, files: F) -> Self {
        Self { repository, files }
    }
}

impl<R, F> CreateChildVersionUseCase<R, F>
where
    R: AssetRepository,
    F: ManagedFileStore,
{
    pub fn execute(&self, request: CreateChildVersionRequest) -> DomainResult<VersionSummary> {
        let parent = self
            .repository
            .load_version(&request.library_path, &request.parent_version_id)?;
        ensure_same_asset_parent(&request.asset_id, &parent.asset_id)?;
        let versions = self
            .repository
            .list_versions_for_asset(&request.library_path, &request.asset_id)?;
        let current_max = versions.iter().map(|version| version.version_number).max();
        let imported = self.files.import_original(
            &request.library_path,
            &request.source_path,
            Some(&request.mime_type),
        )?;
        self.repository
            .persist_asset_version(PersistAssetVersionRequest {
                library_path: request.library_path,
                asset_id: request.asset_id,
                parent_version_id: Some(request.parent_version_id),
                generation_event_id: request.generation_event_id,
                version_id: imported.version_id,
                file: imported.metadata,
                version_number: next_version_number(current_max),
                version_label: request.version_label,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetId, AssetVersionId, CreateGenerationEventRequest, GenerationEventSummary,
        GenerationOperation, ImportAssetRequest, ManagedFileImport, ManagedFileMetadata,
        PersistAssetVersionRequest, PromoteAssetVersionSummary,
    };
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(Default)]
    struct FakeAssetRepository {
        persisted_versions: RefCell<Vec<PersistAssetVersionRequest>>,
        existing_versions: RefCell<Vec<VersionSummary>>,
        added_tags: RefCell<Vec<AddAssetTagRequest>>,
    }

    impl AssetRepository for FakeAssetRepository {
        fn load_version(
            &self,
            _library_path: &std::path::Path,
            version_id: &AssetVersionId,
        ) -> DomainResult<VersionSummary> {
            Ok(VersionSummary {
                id: version_id.clone(),
                ..version_summary(1)
            })
        }

        fn list_versions_for_asset(
            &self,
            _library_path: &std::path::Path,
            _asset_id: &AssetId,
        ) -> DomainResult<Vec<VersionSummary>> {
            Ok(self.existing_versions.borrow().clone())
        }

        fn persist_imported_asset(
            &self,
            request: PersistImportedAssetRequest,
        ) -> DomainResult<(AssetSummary, VersionSummary)> {
            Ok((asset_summary(), version_summary(request.version_number)))
        }

        fn persist_asset_version(
            &self,
            request: PersistAssetVersionRequest,
        ) -> DomainResult<VersionSummary> {
            let version_number = request.version_number;
            self.persisted_versions.borrow_mut().push(request);
            Ok(version_summary(version_number))
        }

        fn promote_version_as_asset(
            &self,
            _request: PromoteAssetVersionRequest,
        ) -> DomainResult<PromoteAssetVersionSummary> {
            unimplemented!("promote use case is delegated to repository integration tests")
        }

        fn record_generation_event(
            &self,
            request: CreateGenerationEventRequest,
        ) -> DomainResult<GenerationEventSummary> {
            let output_version_id = request.output_version_id.clone();
            Ok(GenerationEventSummary {
                id: request
                    .output_version_id
                    .map(|id| crate::GenerationEventId(id.0))
                    .unwrap_or_else(|| crate::GenerationEventId("event-1".to_string())),
                asset_id: request.asset_id,
                output_version_id,
                provider: request.provider,
                provider_model: request.provider_model,
                operation_type: request.operation_type,
                prompt: request.prompt,
                prompt_version_id: request.prompt_version_id,
                parameters_json: request.parameters_json,
                status: request.status,
            })
        }

        fn mark_version_generated(
            &self,
            _library_path: &std::path::Path,
            _asset_id: &AssetId,
            _version_id: &AssetVersionId,
            _generation_event_id: &crate::GenerationEventId,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn add_tag_to_asset(&self, request: AddAssetTagRequest) -> DomainResult<()> {
            self.added_tags.borrow_mut().push(request);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeFileStore {
        imported_paths: RefCell<Vec<PathBuf>>,
    }

    impl ManagedFileStore for FakeFileStore {
        fn read_source_bytes(&self, _source_path: &std::path::Path) -> DomainResult<Vec<u8>> {
            Ok(Vec::new())
        }

        fn import_original(
            &self,
            _library_path: &std::path::Path,
            source_path: &std::path::Path,
            mime_type_override: Option<&str>,
        ) -> DomainResult<ManagedFileImport> {
            self.imported_paths
                .borrow_mut()
                .push(source_path.to_path_buf());
            Ok(ManagedFileImport {
                version_id: AssetVersionId("version-imported".to_string()),
                metadata: ManagedFileMetadata {
                    file_path: PathBuf::from("originals/test.png"),
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
            Ok(Vec::new())
        }

        fn write_generated_bytes(
            &self,
            _library_path: &std::path::Path,
            _mime_type: &str,
            _bytes: &[u8],
        ) -> DomainResult<PathBuf> {
            Ok(PathBuf::from("/tmp/generated.png"))
        }
    }

    fn asset_summary() -> AssetSummary {
        AssetSummary {
            id: AssetId("asset-1".to_string()),
            title: None,
            category: None,
            rating: None,
            status: "imported".to_string(),
        }
    }

    fn version_summary(version_number: u32) -> VersionSummary {
        VersionSummary {
            id: AssetVersionId(format!("version-{version_number}")),
            asset_id: AssetId("asset-1".to_string()),
            parent_version_id: None,
            generation_event_id: None,
            version_number,
            version_name: crate::version_name(version_number),
            file_path: PathBuf::from("originals/test.png"),
            checksum_algorithm: "SHA-256".to_string(),
            checksum: "abc".to_string(),
            mime_type: "image/png".to_string(),
        }
    }

    #[test]
    fn import_asset_use_case_delegates_to_repository_port() {
        let repository = FakeAssetRepository::default();
        let files = FakeFileStore::default();
        let use_case = ImportAssetUseCase::new(repository, files);

        let (_, version) = use_case
            .execute(ImportAssetRequest {
                library_path: PathBuf::from("/tmp/library"),
                source_path: PathBuf::from("/tmp/source.png"),
            })
            .expect("import");

        assert_eq!(version.version_number, 1);
        assert_eq!(use_case.files.imported_paths.borrow().len(), 1);
    }

    #[test]
    fn create_child_version_use_case_delegates_to_repository_port() {
        let repository = FakeAssetRepository {
            existing_versions: RefCell::new(vec![version_summary(1), version_summary(2)]),
            ..FakeAssetRepository::default()
        };
        let files = FakeFileStore::default();
        let use_case = CreateChildVersionUseCase::new(repository, files);

        let version = use_case
            .execute(CreateChildVersionRequest {
                library_path: PathBuf::from("/tmp/library"),
                asset_id: AssetId("asset-1".to_string()),
                parent_version_id: AssetVersionId("version-1".to_string()),
                generation_event_id: None,
                source_path: PathBuf::from("/tmp/generated.png"),
                mime_type: "image/png".to_string(),
                version_label: Some("generated".to_string()),
            })
            .expect("child version");

        assert_eq!(version.version_number, 3);
        let persisted = use_case.repository.persisted_versions.borrow();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].version_number, 3);
        assert_eq!(
            persisted[0].parent_version_id,
            Some(AssetVersionId("version-1".to_string()))
        );
        assert_eq!(persisted[0].file.mime_type, "image/png");
    }

    #[test]
    fn asset_use_case_delegates_tag_mutation_to_repository_port() {
        let use_case = AssetUseCase::new(FakeAssetRepository::default(), FakeFileStore::default());

        use_case
            .add_tag(AddAssetTagRequest {
                library_path: PathBuf::from("/tmp/library"),
                asset_id: AssetId("asset-1".to_string()),
                tag: "favorite".to_string(),
            })
            .expect("tag add");

        let tags = use_case.repository.added_tags.borrow();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].asset_id, AssetId("asset-1".to_string()));
        assert_eq!(tags[0].tag, "favorite");
    }

    #[test]
    fn fake_repository_keeps_generation_event_contract_shape() {
        let repository = FakeAssetRepository::default();
        let event = repository
            .record_generation_event(CreateGenerationEventRequest {
                library_path: PathBuf::from("/tmp/library"),
                asset_id: Some(AssetId("asset-1".to_string())),
                output_version_id: Some(AssetVersionId("version-1".to_string())),
                provider: "fake".to_string(),
                provider_model: "fake-image".to_string(),
                operation_type: GenerationOperation::TextToImage,
                prompt: "prompt".to_string(),
                negative_prompt: None,
                input_asset_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
                raw_request_json: None,
                raw_response_json: None,
                status: "completed".to_string(),
                error_code: None,
                error_message: None,
            })
            .expect("event");

        assert_eq!(event.provider, "fake");
        assert_eq!(event.status, "completed");
    }
}
