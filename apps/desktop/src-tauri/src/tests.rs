use super::*;
use crate::commands::daemon::generation_draft_to_daemon_task;

#[test]
fn maps_gallery_sort_input() {
    assert!(matches!(
        gallery_sort_from_input(Some("ratingDesc")).expect("sort"),
        GallerySort::RatingDesc
    ));
    let error = gallery_sort_from_input(Some("unknown")).expect_err("invalid sort");
    assert_eq!(error.code, "InvalidGalleryQuery");
    assert!(error.recoverable);
}

#[test]
fn maps_legacy_album_id_to_explicit_album_filter() {
    let query = gallery_query_from_input(QueryGalleryInput {
        library_path: PathBuf::from("/tmp/library"),
        text: None,
        providers: None,
        min_rating: None,
        review_status: None,
        tags: None,
        album_filter: None,
        album_id: Some("album-1".to_string()),
        sort: None,
    })
    .expect("query");

    assert_eq!(
        query.album_filter,
        GalleryAlbumFilter::InAny(vec![AlbumId("album-1".to_string())])
    );
    assert_eq!(query.album_id, None);
}

#[test]
fn explicit_album_filter_ignores_stale_legacy_album_id() {
    let query = gallery_query_from_input(QueryGalleryInput {
        library_path: PathBuf::from("/tmp/library"),
        text: None,
        providers: None,
        min_rating: None,
        review_status: None,
        tags: None,
        album_filter: Some(GalleryAlbumFilterInput {
            mode: "any".to_string(),
            album_ids: None,
        }),
        album_id: Some("album-1".to_string()),
        sort: None,
    })
    .expect("query");

    assert_eq!(query.album_filter, GalleryAlbumFilter::Any);
    assert_eq!(query.album_id, None);
}

#[test]
fn maps_provider_capability_error_as_recoverable() {
    let error: CommandError = DomainError::UnsupportedProviderCapability {
        provider: "codex-cli".to_string(),
        capability: "image_to_image".to_string(),
    }
    .into();

    assert_eq!(error.code, "UnsupportedProviderCapability");
    assert!(error.recoverable);
}

#[test]
fn maps_archived_content_type_and_summary_views() {
    assert!(matches!(
        archived_content_type_from_input("asset").expect("asset"),
        imglab_core::ArchivedContentType::Asset
    ));
    assert!(archived_content_type_from_input("unknown").is_err());

    let archived = archived_content_view(imglab_core::ArchivedContentSummary {
        id: "asset-1".to_string(),
        item_type: imglab_core::ArchivedContentType::Asset,
        title: "Archived asset".to_string(),
        archived_at: "1000".to_string(),
        dependency_summary: "1 version(s)".to_string(),
        file_count: 1,
        file_size_bytes: 42,
    });
    assert_eq!(archived.item_type, "asset");
    assert_eq!(archived.file_size_bytes, 42);

    let delete = permanent_delete_summary_view(imglab_core::PermanentDeleteSummary {
        item_id: "asset-1".to_string(),
        item_type: imglab_core::ArchivedContentType::Asset,
        sqlite_row_count: 3,
        file_count: 1,
        file_size_bytes: 42,
        warnings: vec!["warning".to_string()],
    });
    assert_eq!(delete.item_type, "asset");
    assert_eq!(delete.sqlite_row_count, 3);
}

#[test]
fn maps_merge_library_summary_view() {
    let view = merge_library_summary_view(imglab_core::MergeLibrarySummary {
        source_library_id: imglab_core::LibraryId("source".to_string()),
        target_library_id: imglab_core::LibraryId("target".to_string()),
        asset_count: 1,
        version_count: 2,
        prompt_count: 3,
        prompt_version_count: 4,
        album_count: 5,
        tag_count: 6,
        generation_event_count: 7,
        metadata_suggestion_count: 8,
        skipped_runtime_row_count: 9,
        file_count: 10,
        file_size_bytes: 11,
        warnings: vec!["skipped runtime rows".to_string()],
    });

    assert_eq!(view.source_library_id, "source");
    assert_eq!(view.target_library_id, "target");
    assert_eq!(view.version_count, 2);
    assert_eq!(view.skipped_runtime_row_count, 9);
}

#[test]
fn expands_home_relative_library_path() {
    let normalized = normalize_library_root_path(PathBuf::from("~/Documents/image-prompt-lab"))
        .expect("normalized path");

    assert!(normalized.is_absolute());
    assert!(normalized.ends_with("Documents/image-prompt-lab"));
    assert!(!normalized.to_string_lossy().starts_with('~'));
}

#[test]
fn rejects_relative_library_path() {
    let error =
        normalize_library_root_path(PathBuf::from("relative/image-prompt-lab")).expect_err("error");

    assert_eq!(error.code, "InvalidPath");
    assert!(error.recoverable);
}

#[test]
fn generation_draft_preserves_prompt_version_link() {
    let draft: GenerationTaskDraftInput = serde_json::from_value(serde_json::json!({
        "provider": "fake",
        "prompt": "rendered prompt",
        "negativePrompt": "avoid blur",
        "promptVersionId": "prompt-version-1",
        "model": "fake-model-v2",
        "valuesJson": "{\"subject\":\"fern\"}",
        "parametersJson": "{\"steps\":24}"
    }))
    .expect("draft");

    let task = generation_draft_to_daemon_task(draft).expect("task");

    assert_eq!(task.input["promptVersionId"], "prompt-version-1");
    assert_eq!(task.input["model"], "fake-model-v2");
    assert_eq!(task.input["valuesJson"], "{\"subject\":\"fern\"}");
    assert_eq!(task.input["prompt"], "rendered prompt");
    assert_eq!(task.input["negativePrompt"], "avoid blur");
    assert_eq!(task.input["parametersJson"]["steps"], 24);
}

#[test]
fn generation_draft_without_prompt_version_remains_legacy_compatible() {
    let draft: GenerationTaskDraftInput = serde_json::from_value(serde_json::json!({
        "provider": "fake",
        "prompt": "legacy prompt"
    }))
    .expect("draft");

    let task = generation_draft_to_daemon_task(draft).expect("task");

    assert!(task.input["promptVersionId"].is_null());
    assert_eq!(task.input["prompt"], "legacy prompt");
}

#[test]
fn maps_prompt_lineage_view() {
    let view = prompt_lineage_view(imglab_core::PromptLineageView {
        prompt_id: imglab_core::PromptId("prompt-1".to_string()),
        prompt_name: "Lighting study".to_string(),
        prompt_version_id: imglab_core::PromptVersionId("prompt-version-1".to_string()),
        prompt_version_number: 3,
        prompt_version_name: "v3".to_string(),
    });

    assert_eq!(view.prompt_id, "prompt-1");
    assert_eq!(view.prompt_name, "Lighting study");
    assert_eq!(view.prompt_version_id, "prompt-version-1");
    assert_eq!(view.prompt_version_number, 3);
    assert_eq!(view.prompt_version_name, "v3");
}

#[test]
fn maps_asset_detail_prompt_lineage() {
    let view = asset_detail_view(
        imglab_core::AssetDetailView {
            id: imglab_core::AssetId("asset-1".to_string()),
            title: None,
            description: None,
            schema_prompt: None,
            category: None,
            rating: None,
            status: "generated".to_string(),
            created_at: "2026-05-24T00:00:00Z".to_string(),
            updated_at: "2026-05-24T00:00:00Z".to_string(),
            prompt: None,
            prompt_generation_event_id: Some(imglab_core::GenerationEventId(
                "generation-event-1".to_string(),
            )),
            negative_prompt: None,
            provider: None,
            model_label: None,
            parameters_json: None,
            tags: Vec::new(),
            albums: Vec::new(),
            review_pending_count: 0,
            current_version_id: None,
            current_version_number: None,
            current_version_name: None,
            focused_version_id: None,
            focused_version_tree_name: None,
            focused_version: None,
            versions: Vec::new(),
            version_tree: Vec::new(),
            version_tree_issues: Vec::new(),
            lineage: Vec::new(),
            prompt_lineage: Some(imglab_core::PromptLineageView {
                prompt_id: imglab_core::PromptId("prompt-1".to_string()),
                prompt_name: "Lighting study".to_string(),
                prompt_version_id: imglab_core::PromptVersionId("prompt-version-1".to_string()),
                prompt_version_number: 3,
                prompt_version_name: "p3".to_string(),
            }),
            source_reference: None,
            promoted_from: None,
            file: None,
        },
        std::path::Path::new("/tmp/library"),
    );

    let lineage = view.prompt_lineage.expect("prompt lineage");
    assert_eq!(
        view.prompt_generation_event_id.as_deref(),
        Some("generation-event-1")
    );
    assert_eq!(lineage.prompt_id, "prompt-1");
    assert_eq!(lineage.prompt_name, "Lighting study");
    assert_eq!(lineage.prompt_version_id, "prompt-version-1");
    assert_eq!(lineage.prompt_version_number, 3);
    assert_eq!(lineage.prompt_version_name, "p3");
}
