mod app_logs;
mod commands;
mod daemon_client;
mod errors;
mod metadata_generation;
mod paths;
mod services;
mod view_mappers;
mod views;

use app_logs::{AppLogContentView, AppLogView, ReadAppLogInput};
use daemon_client::{
    BatchCreateTasksInput, DaemonSidecar, DaemonTask, DaemonTaskAttempt, DaemonTaskDetail,
    DaemonTaskEvent, DaemonTaskInput, DaemonTaskOutput,
};
use errors::*;
use imglab_core::{
    prepare_generation_request, AlbumId, AlbumService, AssetId, AssetService,
    BatchAddAssetsToAlbumRequest, BatchReviewMetadataSuggestionRequest, CreateLibraryRequest,
    CreateMetadataSuggestionRequest, CreateSmartAlbumRequest, DomainError,
    ExportLibraryBackupRequest, ExportLibraryRequest, GalleryQuery, GalleryReadService,
    GallerySort, GenerateImageRequest, GenerationOperation, GenerationRequestInput,
    GenerationService, ImageProvider, ImportAssetRequest, ImportLibraryBackupRequest, LibraryId,
    LibraryService, LocalGenerationService, LocalLibraryService, MetadataReviewService,
    MetadataSuggestionId, RenameLibraryAliasRequest, ReorderAlbumItemsRequest,
    ReorderAlbumsRequest, RepairLibraryRequest, ReviewMetadataSuggestionRequest,
    ReviewStatusFilter, SearchQuery, SearchService, UpdateAssetMetadataRequest,
};
use imglab_provider_codex::CodexCliImageProvider;
use metadata_generation::{
    CodexCliMetadataGenerator, GenerateReviewFieldInput, GeneratedReviewFieldView, ReviewField,
};
use paths::*;
use serde::{Deserialize, Serialize};
use services::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;
use view_mappers::*;
use views::*;

#[derive(Clone, Default)]
struct DesktopState {
    daemon_sidecar: Arc<Mutex<Option<DaemonSidecar>>>,
}

pub fn run() {
    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = _app.get_webview_window("main") {
                    window.set_background_color(Some(tauri::window::Color(32, 37, 39, 255)))?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::health,
            commands::library::create_library,
            commands::library::list_libraries,
            commands::library::open_library,
            commands::library::library_status,
            commands::library::studio_overview,
            commands::library::diagnostics_overview,
            commands::library::repair_library,
            commands::library::hide_library,
            commands::library::rename_library_alias,
            commands::library::unregister_library,
            commands::library::import_asset,
            commands::library::export_library,
            commands::library::export_library_backup_zip,
            commands::library::import_library_backup_zip,
            commands::library::reveal_library_folder,
            commands::gallery::search_assets,
            commands::gallery::query_gallery,
            commands::gallery::get_asset_detail,
            commands::gallery::get_asset_inspector_detail,
            commands::generation::generate_image,
            commands::daemon::daemon_health,
            commands::daemon::enqueue_generation_tasks,
            commands::daemon::list_daemon_tasks,
            commands::daemon::get_daemon_task_detail,
            commands::daemon::reorder_daemon_tasks,
            commands::daemon::cancel_daemon_task,
            commands::daemon::retry_daemon_task,
            commands::daemon::duplicate_daemon_task,
            commands::metadata::update_asset_metadata,
            commands::metadata::add_tag_to_asset,
            commands::albums::list_albums,
            commands::albums::create_manual_album,
            commands::albums::create_smart_album,
            commands::albums::add_asset_to_album,
            commands::albums::batch_add_assets_to_album,
            commands::albums::remove_asset_from_album,
            commands::albums::rename_album,
            commands::albums::delete_album,
            commands::albums::reorder_albums,
            commands::albums::reorder_album_items,
            commands::review::create_suggestion,
            commands::review::list_pending_suggestions,
            commands::review::get_review_draft_detail,
            commands::review::accept_suggestion,
            commands::review::batch_accept_suggestions,
            commands::review::reject_suggestion,
            commands::review::batch_reject_suggestions,
            commands::review::list_suggestion_history,
            commands::review::generate_review_field,
            commands::review::regenerate_suggestion,
            commands::logs::list_app_logs,
            commands::logs::read_app_log,
            commands::updater::check_for_update,
            commands::updater::install_update,
            commands::updater::restart_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run desktop application");
}

#[cfg(test)]
mod tests;
