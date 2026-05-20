#[tauri::command]
fn health() -> &'static str {
    "ok"
}

#[tauri::command]
fn create_library(input: CreateLibraryInput) -> Result<LibraryView, CommandError> {
    let root_path = normalize_library_root_path(input.root_path)?;
    service()
        .create_library(CreateLibraryRequest {
            root_path,
            name: input.name,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn list_libraries(include_hidden: bool) -> Result<Vec<LibraryView>, CommandError> {
    service()
        .list_libraries(include_hidden)
        .map(|libraries| libraries.into_iter().map(library_view).collect())
        .map_err(Into::into)
}

#[tauri::command]
fn open_library(root_path: PathBuf) -> Result<LibraryView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .open_library(&root_path)
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn library_status(root_path: PathBuf) -> Result<LibraryStatusView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .library_status(&root_path)
        .map(library_status_view)
        .map_err(Into::into)
}

#[tauri::command]
fn studio_overview(root_path: PathBuf) -> Result<StudioOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .studio_overview(&root_path)
        .map(studio_overview_view)
        .map_err(Into::into)
}

#[tauri::command]
fn diagnostics_overview(root_path: PathBuf) -> Result<DiagnosticsOverviewView, CommandError> {
    let root_path = normalize_library_root_path(root_path)?;
    service()
        .diagnostics_overview(&root_path)
        .map(diagnostics_overview_view)
        .map_err(Into::into)
}

#[tauri::command]
fn repair_library(input: RepairLibraryInput) -> Result<RepairSummaryView, CommandError> {
    service()
        .repair_library(RepairLibraryRequest {
            library_path: input.library_path,
            dry_run: input.dry_run,
        })
        .map(repair_summary_view)
        .map_err(Into::into)
}

#[tauri::command]
fn hide_library(library_id: String) -> Result<(), CommandError> {
    service()
        .hide_library(&LibraryId(library_id))
        .map_err(Into::into)
}

#[tauri::command]
fn rename_library_alias(input: RenameLibraryAliasInput) -> Result<LibraryView, CommandError> {
    service()
        .rename_library_alias(RenameLibraryAliasRequest {
            library_id: LibraryId(input.library_id),
            alias: input.alias,
        })
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
fn unregister_library(library_id: String) -> Result<(), CommandError> {
    service()
        .unregister_library(&LibraryId(library_id))
        .map_err(Into::into)
}

#[tauri::command]
fn import_asset(input: ImportAssetInput) -> Result<(AssetView, VersionView), CommandError> {
    service()
        .import_asset(ImportAssetRequest {
            library_path: input.library_path,
            source_path: input.source_path,
        })
        .map(|(asset, version)| (asset_view(asset), version_view(version)))
        .map_err(Into::into)
}

#[tauri::command]
fn export_library(input: ExportLibraryInput) -> Result<serde_json::Value, CommandError> {
    service()
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
fn export_library_backup_zip(input: ExportLibraryBackupInput) -> Result<(), CommandError> {
    let library_path = normalize_library_root_path(input.library_path)?;
    service()
        .export_library_backup_zip(ExportLibraryBackupRequest {
            library_path,
            output_zip_path: input.output_zip_path,
        })
        .map_err(Into::into)
}

#[tauri::command]
fn import_library_backup_zip(
    input: ImportLibraryBackupInput,
) -> Result<LibraryBackupView, CommandError> {
    let destination_path = normalize_library_root_path(input.destination_path)?;
    service()
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
fn reveal_library_folder(root_path: PathBuf) -> Result<(), CommandError> {
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
