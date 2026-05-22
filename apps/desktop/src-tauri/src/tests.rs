use super::*;

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
