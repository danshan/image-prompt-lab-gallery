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
        "parametersJson": "{\"steps\":24}"
    }))
    .expect("draft");

    let task = generation_draft_to_daemon_task(draft).expect("task");

    assert_eq!(task.input["promptVersionId"], "prompt-version-1");
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
    assert_eq!(lineage.prompt_id, "prompt-1");
    assert_eq!(lineage.prompt_name, "Lighting study");
    assert_eq!(lineage.prompt_version_id, "prompt-version-1");
    assert_eq!(lineage.prompt_version_number, 3);
    assert_eq!(lineage.prompt_version_name, "p3");
}
