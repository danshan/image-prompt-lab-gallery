use crate::application::ports::LibraryRepository;
use crate::{
    CreateLibraryRequest, DiagnosticsOverviewView, DomainResult, ExportLibraryBackupRequest,
    ExportLibraryRequest, ExportSummary, ImportLibraryBackupRequest, IntegrityIssue,
    LibraryBackupSummary, LibraryId, LibraryStatusView, LibrarySummary, RenameLibraryAliasRequest,
    RepairLibraryRequest, RepairSummary, StudioOverviewView,
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
        }
    }
}
