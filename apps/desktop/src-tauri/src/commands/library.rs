use crate::*;

#[tauri::command]
pub(crate) fn health() -> &'static str {
    "ok"
}

#[tauri::command]
pub(crate) fn create_library(input: CreateLibraryInput) -> Result<LibraryView, CommandError> {
    let root_path = normalize_library_root_path(input.root_path)?;
    desktop_app()
        .library_lifecycle()
        .create_library(CreateLibraryRequest {
            root_path,
            name: input.name,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn list_libraries(include_hidden: bool) -> Result<Vec<LibraryView>, CommandError> {
    desktop_app()
        .library_lifecycle()
        .list_libraries(include_hidden)
        .map(|libraries| libraries.into_iter().map(library_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn open_library(root_path: PathBuf) -> Result<LibraryView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    desktop_app()
        .library_lifecycle()
        .open_library(&root_path)
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn library_status(root_path: PathBuf) -> Result<LibraryStatusView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    desktop_app()
        .library_lifecycle()
        .library_status(&root_path)
        .map(library_status_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn studio_overview(root_path: PathBuf) -> Result<StudioOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    desktop_app()
        .library_lifecycle()
        .studio_overview(&root_path)
        .map(studio_overview_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn diagnostics_overview(
    root_path: PathBuf,
) -> Result<DiagnosticsOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    desktop_app()
        .library_lifecycle()
        .diagnostics_overview(&root_path)
        .map(diagnostics_overview_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn repair_library(input: RepairLibraryInput) -> Result<RepairSummaryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .repair_library(RepairLibraryRequest {
            library_path: input.library_path,
            dry_run: input.dry_run,
        })
        .map(repair_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn hide_library(library_id: String) -> Result<(), CommandError> {
    desktop_app()
        .library_lifecycle()
        .hide_library(&LibraryId(library_id))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn rename_library_alias(
    input: RenameLibraryAliasInput,
) -> Result<LibraryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .rename_library_alias(RenameLibraryAliasRequest {
            library_id: LibraryId(input.library_id),
            alias: input.alias,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn unregister_library(library_id: String) -> Result<(), CommandError> {
    desktop_app()
        .library_lifecycle()
        .unregister_library(&LibraryId(library_id))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn import_asset(
    input: ImportAssetInput,
) -> Result<(AssetView, VersionView), CommandError> {
    desktop_app()
        .assets()
        .import_asset(ImportAssetRequest {
            library_path: input.library_path,
            source_path: input.source_path,
        })
        .map(|(asset, version)| (asset_view(asset), version_view(version)))
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn export_library(input: ExportLibraryInput) -> Result<serde_json::Value, CommandError> {
    desktop_app()
        .library_lifecycle()
        .export_library(ExportLibraryRequest {
            library_path: input.library_path,
            output_path: input.output_path,
            album_id: input.album_id.map(imglab_core::AlbumId),
        })
        .map(|summary| {
            serde_json::json!({
                "exportedFiles": summary.exported_files,
                "exportedSidecars": summary.exported_sidecars
            })
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn export_library_backup_zip(
    input: ExportLibraryBackupInput,
) -> Result<(), CommandError> {
    let library_path = normalize_library_root_path(input.library_path)?;
    desktop_app()
        .library_lifecycle()
        .export_library_backup_zip(ExportLibraryBackupRequest {
            library_path,
            output_zip_path: input.output_zip_path,
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn import_library_backup_zip(
    input: ImportLibraryBackupInput,
) -> Result<LibraryBackupView, CommandError> {
    let destination_path = normalize_library_root_path(input.destination_path)?;
    desktop_app()
        .library_lifecycle()
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path: input.zip_path,
            destination_path,
        })
        .map(|summary| LibraryBackupView {
            library: library_view(summary.library),
            cloned: summary.cloned,
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn list_archived_content(
    input: ListArchivedContentInput,
) -> Result<Vec<ArchivedContentView>, CommandError> {
    let item_type = input
        .item_type
        .as_deref()
        .map(archived_content_type_from_input)
        .transpose()?;
    desktop_app()
        .library_lifecycle()
        .list_archived_content(imglab_core::ListArchivedContentRequest {
            library_path: input.library_path,
            item_type,
        })
        .map(|items| items.into_iter().map(archived_content_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn restore_asset(input: RestoreAssetInput) -> Result<(), CommandError> {
    desktop_app()
        .library_lifecycle()
        .restore_asset(imglab_core::RestoreAssetRequest {
            library_path: input.library_path,
            asset_id: imglab_core::AssetId(input.asset_id),
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn restore_prompt_document(
    input: RestorePromptDocumentInput,
) -> Result<(), CommandError> {
    desktop_app()
        .library_lifecycle()
        .restore_prompt_document(imglab_core::RestorePromptDocumentRequest {
            library_path: input.library_path,
            prompt_id: imglab_core::PromptId(input.prompt_id),
        })
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn dry_run_permanent_delete_archived_content(
    input: PermanentDeleteArchivedContentInput,
) -> Result<PermanentDeleteSummaryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .dry_run_permanent_delete_archived_content(
            imglab_core::PermanentDeleteArchivedContentRequest {
                library_path: input.library_path,
                item_type: archived_content_type_from_input(&input.item_type)?,
                id: input.id,
            },
        )
        .map(permanent_delete_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn permanent_delete_archived_content(
    input: PermanentDeleteArchivedContentInput,
) -> Result<PermanentDeleteSummaryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .permanent_delete_archived_content(imglab_core::PermanentDeleteArchivedContentRequest {
            library_path: input.library_path,
            item_type: archived_content_type_from_input(&input.item_type)?,
            id: input.id,
        })
        .map(permanent_delete_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn dry_run_merge_library(
    input: MergeLibraryInput,
) -> Result<MergeLibrarySummaryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .dry_run_merge_library(imglab_core::MergeLibraryRequest {
            target_library_path: input.target_library_path,
            source_library_path: input.source_library_path,
        })
        .map(merge_library_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn merge_library(
    input: MergeLibraryInput,
) -> Result<MergeLibrarySummaryView, CommandError> {
    desktop_app()
        .library_lifecycle()
        .merge_library(imglab_core::MergeLibraryRequest {
            target_library_path: input.target_library_path,
            source_library_path: input.source_library_path,
        })
        .map(merge_library_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn reveal_library_folder(root_path: PathBuf) -> Result<(), CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    if !root_path.is_dir() {
        return Err(CommandError {
            code: "LibraryNotFound".to_string(),
            message: format!("library folder is missing: {}", root_path.display()),
            recoverable: true,
        });
    }
    reveal_path(&root_path)
}
