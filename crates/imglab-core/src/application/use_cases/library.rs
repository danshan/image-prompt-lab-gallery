use crate::application::ports::LibraryRepository;
use crate::{
    ArchiveAssetRequest, ArchivePromptDocumentRequest, ArchivedContentSummary,
    CreateLibraryRequest, DiagnosticsOverviewView, DomainResult, ExportLibraryBackupRequest,
    ExportLibraryRequest, ExportSummary, ImportLibraryBackupRequest, IntegrityIssue,
    LibraryBackupSummary, LibraryId, LibraryStatusView, LibrarySummary, ListArchivedContentRequest,
    MergeLibraryRequest, MergeLibrarySummary, PermanentDeleteArchivedContentRequest,
    PermanentDeleteSummary, RenameLibraryAliasRequest, RepairLibraryRequest, RepairSummary,
    RestoreAssetRequest, RestorePromptDocumentRequest, StudioOverviewView,
};
use std::path::Path;

pub struct LibraryUseCase<R> {
    repository: R,
}

impl<R> LibraryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> LibraryUseCase<R>
where
    R: LibraryRepository,
{
    pub fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary> {
        self.repository.create_library(request)
    }

    pub fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
        self.repository.open_library(root_path)
    }

    pub fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>> {
        self.repository.list_libraries(include_hidden)
    }

    pub fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        self.repository.hide_library(library_id)
    }

    pub fn rename_library_alias(
        &self,
        request: RenameLibraryAliasRequest,
    ) -> DomainResult<LibrarySummary> {
        self.repository.rename_library_alias(request)
    }

    pub fn unregister_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        self.repository.unregister_library(library_id)
    }

    pub fn export_library(&self, request: ExportLibraryRequest) -> DomainResult<ExportSummary> {
        self.repository.export_library(request)
    }

    pub fn export_library_backup_zip(
        &self,
        request: ExportLibraryBackupRequest,
    ) -> DomainResult<()> {
        self.repository.export_library_backup_zip(request)
    }

    pub fn import_library_backup_zip(
        &self,
        request: ImportLibraryBackupRequest,
    ) -> DomainResult<LibraryBackupSummary> {
        self.repository.import_library_backup_zip(request)
    }

    pub fn dry_run_merge_library(
        &self,
        request: MergeLibraryRequest,
    ) -> DomainResult<MergeLibrarySummary> {
        self.repository.dry_run_merge_library(request)
    }

    pub fn merge_library(&self, request: MergeLibraryRequest) -> DomainResult<MergeLibrarySummary> {
        self.repository.merge_library(request)
    }

    pub fn archive_asset(&self, request: ArchiveAssetRequest) -> DomainResult<()> {
        self.repository.archive_asset(request)
    }

    pub fn restore_asset(&self, request: RestoreAssetRequest) -> DomainResult<()> {
        self.repository.restore_asset(request)
    }

    pub fn archive_prompt_document(
        &self,
        request: ArchivePromptDocumentRequest,
    ) -> DomainResult<()> {
        self.repository.archive_prompt_document(request)
    }

    pub fn restore_prompt_document(
        &self,
        request: RestorePromptDocumentRequest,
    ) -> DomainResult<()> {
        self.repository.restore_prompt_document(request)
    }

    pub fn list_archived_content(
        &self,
        request: ListArchivedContentRequest,
    ) -> DomainResult<Vec<ArchivedContentSummary>> {
        self.repository.list_archived_content(request)
    }

    pub fn dry_run_permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        self.repository
            .dry_run_permanent_delete_archived_content(request)
    }

    pub fn permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        self.repository.permanent_delete_archived_content(request)
    }

    pub fn repair_library(&self, request: RepairLibraryRequest) -> DomainResult<RepairSummary> {
        self.repository.repair_library(request)
    }

    pub fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
        self.repository.check_integrity(root_path)
    }

    pub fn library_status(&self, root_path: &Path) -> DomainResult<LibraryStatusView> {
        self.repository.library_status(root_path)
    }

    pub fn studio_overview(&self, root_path: &Path) -> DomainResult<StudioOverviewView> {
        self.repository.studio_overview(root_path)
    }

    pub fn diagnostics_overview(&self, root_path: &Path) -> DomainResult<DiagnosticsOverviewView> {
        self.repository.diagnostics_overview(root_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, LibraryId};
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(Default)]
    struct RecordingLibraryRepository {
        created: RefCell<Vec<CreateLibraryRequest>>,
        opened: RefCell<Vec<PathBuf>>,
        listed: RefCell<Vec<bool>>,
    }

    impl LibraryRepository for RecordingLibraryRepository {
        fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary> {
            self.created.borrow_mut().push(request.clone());
            Ok(summary(request.root_path, request.name))
        }

        fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
            self.opened.borrow_mut().push(root_path.to_path_buf());
            Ok(summary(root_path.to_path_buf(), "Opened".to_string()))
        }

        fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>> {
            self.listed.borrow_mut().push(include_hidden);
            Ok(vec![summary(
                PathBuf::from("/tmp/library"),
                "Listed".to_string(),
            )])
        }

        fn hide_library(&self, _library_id: &LibraryId) -> DomainResult<()> {
            Ok(())
        }

        fn rename_library_alias(
            &self,
            _request: RenameLibraryAliasRequest,
        ) -> DomainResult<LibrarySummary> {
            Ok(summary(
                PathBuf::from("/tmp/library"),
                "Renamed".to_string(),
            ))
        }

        fn unregister_library(&self, _library_id: &LibraryId) -> DomainResult<()> {
            Ok(())
        }

        fn export_library(&self, _request: ExportLibraryRequest) -> DomainResult<ExportSummary> {
            Ok(ExportSummary {
                exported_files: 0,
                exported_sidecars: 0,
            })
        }

        fn export_library_backup_zip(
            &self,
            _request: ExportLibraryBackupRequest,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn import_library_backup_zip(
            &self,
            _request: ImportLibraryBackupRequest,
        ) -> DomainResult<LibraryBackupSummary> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn dry_run_merge_library(
            &self,
            _request: MergeLibraryRequest,
        ) -> DomainResult<MergeLibrarySummary> {
            Ok(MergeLibrarySummary {
                source_library_id: LibraryId("source".to_string()),
                target_library_id: LibraryId("target".to_string()),
                asset_count: 0,
                version_count: 0,
                prompt_count: 0,
                prompt_version_count: 0,
                album_count: 0,
                tag_count: 0,
                generation_event_count: 0,
                metadata_suggestion_count: 0,
                skipped_runtime_row_count: 0,
                file_count: 0,
                file_size_bytes: 0,
                warnings: Vec::new(),
            })
        }

        fn merge_library(&self, request: MergeLibraryRequest) -> DomainResult<MergeLibrarySummary> {
            self.dry_run_merge_library(request)
        }

        fn archive_asset(&self, _request: ArchiveAssetRequest) -> DomainResult<()> {
            Ok(())
        }

        fn restore_asset(&self, _request: RestoreAssetRequest) -> DomainResult<()> {
            Ok(())
        }

        fn archive_prompt_document(
            &self,
            _request: ArchivePromptDocumentRequest,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn restore_prompt_document(
            &self,
            _request: RestorePromptDocumentRequest,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn list_archived_content(
            &self,
            _request: ListArchivedContentRequest,
        ) -> DomainResult<Vec<ArchivedContentSummary>> {
            Ok(Vec::new())
        }

        fn dry_run_permanent_delete_archived_content(
            &self,
            _request: PermanentDeleteArchivedContentRequest,
        ) -> DomainResult<PermanentDeleteSummary> {
            Ok(PermanentDeleteSummary {
                item_id: "item".to_string(),
                item_type: crate::ArchivedContentType::Asset,
                sqlite_row_count: 0,
                file_count: 0,
                file_size_bytes: 0,
                warnings: Vec::new(),
            })
        }

        fn permanent_delete_archived_content(
            &self,
            request: PermanentDeleteArchivedContentRequest,
        ) -> DomainResult<PermanentDeleteSummary> {
            self.dry_run_permanent_delete_archived_content(request)
        }

        fn repair_library(&self, _request: RepairLibraryRequest) -> DomainResult<RepairSummary> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn check_integrity(&self, _root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
            Ok(Vec::new())
        }

        fn library_status(&self, _root_path: &Path) -> DomainResult<LibraryStatusView> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn studio_overview(&self, _root_path: &Path) -> DomainResult<StudioOverviewView> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }

        fn diagnostics_overview(&self, _root_path: &Path) -> DomainResult<DiagnosticsOverviewView> {
            Err(DomainError::Database {
                message: "not implemented in recording repository".to_string(),
            })
        }
    }

    #[test]
    fn library_use_case_delegates_create_open_and_list() {
        let repository = RecordingLibraryRepository::default();
        let use_case = LibraryUseCase::new(repository);

        let created = use_case
            .create_library(CreateLibraryRequest {
                root_path: PathBuf::from("/tmp/created"),
                name: "Created".to_string(),
            })
            .expect("create library");
        let opened = use_case
            .open_library(Path::new("/tmp/opened"))
            .expect("open library");
        let listed = use_case.list_libraries(true).expect("list libraries");

        assert_eq!(created.name, "Created");
        assert_eq!(opened.root_path, PathBuf::from("/tmp/opened"));
        assert_eq!(listed.len(), 1);
        assert_eq!(use_case.repository.created.borrow().len(), 1);
        assert_eq!(use_case.repository.opened.borrow().len(), 1);
        assert_eq!(*use_case.repository.listed.borrow(), vec![true]);
    }

    fn summary(root_path: PathBuf, name: String) -> LibrarySummary {
        LibrarySummary {
            id: LibraryId("library-1".to_string()),
            name,
            root_path,
            hidden: false,
            schema_version: 1,
            automation_enabled: false,
        }
    }
}
