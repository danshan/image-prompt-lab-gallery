mod app_logs;
mod daemon_client;
mod metadata_generation;

use app_logs::{AppLogContentView, AppLogView, ReadAppLogInput};
use daemon_client::{
    BatchCreateTasksInput, DaemonSidecar, DaemonTask, DaemonTaskAttempt, DaemonTaskDetail,
    DaemonTaskEvent, DaemonTaskInput, DaemonTaskOutput,
};
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
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Default)]
struct DesktopState {
    daemon_sidecar: Arc<Mutex<Option<DaemonSidecar>>>,
}

include!("errors.rs");
include!("views.rs");
include!("paths.rs");
include!("view_mappers.rs");
include!("services.rs");
include!("commands/library.rs");
include!("commands/gallery.rs");
include!("commands/generation.rs");
include!("commands/daemon.rs");
include!("commands/metadata.rs");
include!("commands/albums.rs");
include!("commands/review.rs");
include!("commands/logs.rs");
include!("commands/updater.rs");

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
            health,
            create_library,
            list_libraries,
            open_library,
            library_status,
            studio_overview,
            diagnostics_overview,
            repair_library,
            hide_library,
            rename_library_alias,
            unregister_library,
            import_asset,
            export_library,
            export_library_backup_zip,
            import_library_backup_zip,
            reveal_library_folder,
            search_assets,
            query_gallery,
            get_asset_detail,
            get_asset_inspector_detail,
            generate_image,
            daemon_health,
            enqueue_generation_tasks,
            list_daemon_tasks,
            get_daemon_task_detail,
            reorder_daemon_tasks,
            cancel_daemon_task,
            retry_daemon_task,
            duplicate_daemon_task,
            update_asset_metadata,
            add_tag_to_asset,
            list_albums,
            create_manual_album,
            create_smart_album,
            add_asset_to_album,
            batch_add_assets_to_album,
            remove_asset_from_album,
            rename_album,
            delete_album,
            reorder_albums,
            reorder_album_items,
            create_suggestion,
            list_pending_suggestions,
            get_review_draft_detail,
            accept_suggestion,
            batch_accept_suggestions,
            reject_suggestion,
            batch_reject_suggestions,
            list_suggestion_history,
            generate_review_field,
            regenerate_suggestion,
            list_app_logs,
            read_app_log,
            check_for_update,
            install_update,
            restart_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run desktop application");
}

#[cfg(test)]
mod tests {
    use super::*;

    include!("tests.rs");
}
