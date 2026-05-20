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
        let error = normalize_library_root_path(PathBuf::from("relative/image-prompt-lab"))
            .expect_err("error");

        assert_eq!(error.code, "InvalidPath");
        assert!(error.recoverable);
    }
