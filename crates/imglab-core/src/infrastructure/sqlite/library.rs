use crate::application::ports::LibraryRepository;
use crate::library::LocalLibraryService;
use crate::{
    CreateLibraryRequest, DomainResult, ExportLibraryBackupRequest, ExportLibraryRequest,
    ExportSummary, ImportLibraryBackupRequest, IntegrityIssue, LibraryBackupSummary, LibraryId,
    LibraryService, LibraryStatusView, LibrarySummary, RenameLibraryAliasRequest,
    RepairLibraryRequest, RepairSummary, StudioOverviewView,
};
use std::path::Path;

impl LibraryRepository for LocalLibraryService {
    fn create_library(&self, request: CreateLibraryRequest) -> DomainResult<LibrarySummary> {
        LibraryService::create_library(self, request)
    }

    fn open_library(&self, root_path: &Path) -> DomainResult<LibrarySummary> {
        LibraryService::open_library(self, root_path)
    }

    fn list_libraries(&self, include_hidden: bool) -> DomainResult<Vec<LibrarySummary>> {
        LibraryService::list_libraries(self, include_hidden)
    }

    fn rename_library_alias(
        &self,
        request: RenameLibraryAliasRequest,
    ) -> DomainResult<LibrarySummary> {
        LibraryService::rename_library_alias(self, request)
    }

    fn unregister_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        LibraryService::unregister_library(self, library_id)
    }

    fn export_library(&self, request: ExportLibraryRequest) -> DomainResult<ExportSummary> {
        LibraryService::export_library(self, request)
    }

    fn export_library_backup_zip(&self, request: ExportLibraryBackupRequest) -> DomainResult<()> {
        LibraryService::export_library_backup_zip(self, request)
    }

    fn import_library_backup_zip(
        &self,
        request: ImportLibraryBackupRequest,
    ) -> DomainResult<LibraryBackupSummary> {
        LibraryService::import_library_backup_zip(self, request)
    }

    fn repair_library(&self, request: RepairLibraryRequest) -> DomainResult<RepairSummary> {
        LibraryService::repair_library(self, request)
    }

    fn check_integrity(&self, root_path: &Path) -> DomainResult<Vec<IntegrityIssue>> {
        LibraryService::check_integrity(self, root_path)
    }

    fn library_status(&self, root_path: &Path) -> DomainResult<LibraryStatusView> {
        LibraryService::library_status(self, root_path)
    }

    fn studio_overview(&self, root_path: &Path) -> DomainResult<StudioOverviewView> {
        LibraryService::studio_overview(self, root_path)
    }
}
