use crate::application::ports::LibraryRepository;
use crate::library::LocalLibraryService;
use crate::{
    ArchiveAssetRequest, ArchivePromptDocumentRequest, ArchivedContentSummary,
    CreateLibraryRequest, DiagnosticsOverviewView, DomainResult, ExportLibraryBackupRequest,
    ExportLibraryRequest, ExportSummary, ImportLibraryBackupRequest, IntegrityIssue,
    LibraryBackupSummary, LibraryId, LibraryService, LibraryStatusView, LibrarySummary,
    ListArchivedContentRequest, PermanentDeleteArchivedContentRequest, PermanentDeleteSummary,
    RenameLibraryAliasRequest, RepairLibraryRequest, RepairSummary, RestoreAssetRequest,
    RestorePromptDocumentRequest, StudioOverviewView,
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

    fn hide_library(&self, library_id: &LibraryId) -> DomainResult<()> {
        LibraryService::hide_library(self, library_id)
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

    fn dry_run_merge_library(
        &self,
        request: crate::MergeLibraryRequest,
    ) -> DomainResult<crate::MergeLibrarySummary> {
        LibraryService::dry_run_merge_library(self, request)
    }

    fn merge_library(
        &self,
        request: crate::MergeLibraryRequest,
    ) -> DomainResult<crate::MergeLibrarySummary> {
        LibraryService::merge_library(self, request)
    }

    fn archive_asset(&self, request: ArchiveAssetRequest) -> DomainResult<()> {
        LibraryService::archive_asset(self, request)
    }

    fn restore_asset(&self, request: RestoreAssetRequest) -> DomainResult<()> {
        LibraryService::restore_asset(self, request)
    }

    fn archive_prompt_document(&self, request: ArchivePromptDocumentRequest) -> DomainResult<()> {
        LibraryService::archive_prompt_document(self, request)
    }

    fn restore_prompt_document(&self, request: RestorePromptDocumentRequest) -> DomainResult<()> {
        LibraryService::restore_prompt_document(self, request)
    }

    fn list_archived_content(
        &self,
        request: ListArchivedContentRequest,
    ) -> DomainResult<Vec<ArchivedContentSummary>> {
        LibraryService::list_archived_content(self, request)
    }

    fn dry_run_permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        LibraryService::dry_run_permanent_delete_archived_content(self, request)
    }

    fn permanent_delete_archived_content(
        &self,
        request: PermanentDeleteArchivedContentRequest,
    ) -> DomainResult<PermanentDeleteSummary> {
        LibraryService::permanent_delete_archived_content(self, request)
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

    fn diagnostics_overview(&self, root_path: &Path) -> DomainResult<DiagnosticsOverviewView> {
        LibraryService::diagnostics_overview(self, root_path)
    }
}
