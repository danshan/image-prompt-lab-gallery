use super::*;
use crate::GenerationService;
use crate::MetadataReviewService;
use crate::{
    AlbumId, AlbumKind, AlbumService, AppendTaskOutputRequest, AssetService,
    BatchAddAssetsToAlbumRequest, BatchCreateTasksRequest, BatchReviewMetadataSuggestionRequest,
    CreatePromptDocumentRequest, CreateTaskInput, GalleryAlbumFilter, GalleryQuery,
    GalleryReadService, GallerySort, GenerateImageRequest, GeneratedImage, GenerationResult,
    ImageProvider, ImportAssetRequest, ListPromptVersionsRequest, PromptVersionId,
    ReorderAlbumItemsRequest, ReorderAlbumsRequest, ReviewMetadataSuggestionRequest,
    ReviewStatusFilter, SavePromptVersionRequest, SearchQuery, SearchService, TaskOutputType,
    TaskService, TaskStatus, TaskType, UpdateAssetMetadataRequest, UpdatePromptDraftRequest,
    UpdateTaskStatusRequest,
};
use std::time::Instant;
use uuid::Uuid;

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("imglab-core-{name}-{}", Uuid::new_v4()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove old test directory");
    }
    root
}

fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes
}

fn move_version_to_legacy_state(
    root: &Path,
    version_id: &AssetVersionId,
    current_relative: &Path,
    legacy_relative: &Path,
) {
    let current = root.join(current_relative);
    let legacy = root.join(legacy_relative);
    fs::create_dir_all(legacy.parent().expect("legacy parent")).expect("create legacy parent");
    fs::rename(&current, &legacy).expect("move to legacy path");
    let md5 = file_digest(&legacy, CHECKSUM_MD5).expect("md5");
    let connection = Connection::open(LocalLibraryService::database_path(root)).expect("open db");
    connection
        .execute(
            "
            UPDATE asset_versions
            SET file_path = ?1,
                sha256 = ?2,
                checksum_algorithm = 'MD5',
                checksum = ?2,
                width = NULL,
                height = NULL
            WHERE id = ?3
            ",
            params![legacy_relative.to_string_lossy(), md5, version_id.0],
        )
        .expect("update legacy version");
}

#[test]
fn prompt_template_rendering_requires_declared_values() {
    use crate::domain::prompt::{render_prompt_template, PromptTemplateVariable};

    let variables = vec![PromptTemplateVariable {
        name: "subject".to_string(),
        label: Some("Subject".to_string()),
        required: true,
        default_value: None,
    }];
    let values = serde_json::json!({});

    let error = render_prompt_template("A {{subject}} study", &variables, &values)
        .expect_err("missing required variable should fail");

    assert!(error.to_string().contains("subject"));
}

#[test]
fn prompt_template_rendering_rejects_undeclared_variables() {
    use crate::domain::prompt::render_prompt_template;

    let error = render_prompt_template("A {{subject}} study", &[], &serde_json::json!({}))
        .expect_err("undeclared variable should fail");

    assert!(error.to_string().contains("subject"));
}

#[test]
fn prompt_template_rendering_uses_runtime_values() {
    use crate::domain::prompt::{render_prompt_template, PromptTemplateVariable};

    let variables = vec![PromptTemplateVariable {
        name: "subject".to_string(),
        label: Some("Subject".to_string()),
        required: true,
        default_value: Some("orchid".to_string()),
    }];
    let rendered = render_prompt_template(
        "A {{subject}} study",
        &variables,
        &serde_json::json!({ "subject": "fern" }),
    )
    .expect("render prompt");

    assert_eq!(rendered, "A fern study");
}

#[test]
fn migration_adds_prompt_workspace_schema_without_backfilling_documents() {
    let root = test_root("prompt-workspace-migration");
    let registry = test_root("prompt-workspace-migration-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Prompt Migration".to_string(),
        })
        .expect("init library");

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let prompt_table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('prompt_documents', 'prompt_versions')",
            [],
            |row| row.get(0),
        )
        .expect("prompt tables");
    assert_eq!(prompt_table_count, 2);

    let prompt_link_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('generation_events') WHERE name = 'prompt_version_id'",
            [],
            |row| row.get(0),
        )
        .expect("prompt link column");
    assert_eq!(prompt_link_count, 1);

    let prompt_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM prompt_documents", [], |row| {
            row.get(0)
        })
        .expect("prompt count");
    assert_eq!(prompt_count, 0);
}

#[test]
fn prompt_repository_save_version_keeps_previous_version_immutable() {
    use crate::application::ports::PromptRepository;

    let root = test_root("prompt-version-immutable");
    let registry = test_root("prompt-version-immutable-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Prompt Versions".to_string(),
        })
        .expect("init library");

    let created = service
        .create_prompt_document(CreatePromptDocumentRequest {
            library_path: root.clone(),
            name: "Botanical".to_string(),
            draft_body: "A {{subject}} study".to_string(),
            draft_negative_prompt: Some("blur".to_string()),
            draft_style_prompt: Some("macro".to_string()),
            variables_schema_json: r#"[{"name":"subject","required":true}]"#.to_string(),
            default_values_json: r#"{"subject":"orchid"}"#.to_string(),
            parameter_preset_json: r#"{"provider":"fake"}"#.to_string(),
            notes: Some("first draft".to_string()),
        })
        .expect("create prompt");

    let first_version = service
        .save_prompt_version(SavePromptVersionRequest {
            library_path: root.clone(),
            prompt_id: created.id.0.clone(),
        })
        .expect("save first version");

    service
        .update_prompt_draft(UpdatePromptDraftRequest {
            library_path: root.clone(),
            prompt_id: created.id.0.clone(),
            name: "Botanical Revised".to_string(),
            draft_body: "A {{subject}} field study".to_string(),
            draft_negative_prompt: Some("noise".to_string()),
            draft_style_prompt: Some("editorial".to_string()),
            variables_schema_json: r#"[{"name":"subject","required":true}]"#.to_string(),
            default_values_json: r#"{"subject":"fern"}"#.to_string(),
            parameter_preset_json: r#"{"provider":"fake","model":"v2"}"#.to_string(),
            notes: Some("second draft".to_string()),
        })
        .expect("update draft");

    let second_version = service
        .save_prompt_version(SavePromptVersionRequest {
            library_path: root.clone(),
            prompt_id: created.id.0.clone(),
        })
        .expect("save second version");

    let versions = service
        .list_prompt_versions(ListPromptVersionsRequest {
            library_path: root,
            prompt_id: created.id.0.clone(),
        })
        .expect("list versions");

    assert_eq!(first_version.version_number, 1);
    assert_eq!(second_version.version_number, 2);
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version_number, 2);
    assert_eq!(versions[1].id, first_version.id);
    assert_eq!(versions[1].body, "A {{subject}} study");
    assert_eq!(versions[1].negative_prompt.as_deref(), Some("blur"));
    assert_eq!(versions[1].style_prompt.as_deref(), Some("macro"));
    assert_eq!(versions[1].default_values_json, r#"{"subject":"orchid"}"#);
    assert_eq!(versions[1].parameter_preset_json, r#"{"provider":"fake"}"#);
    assert_eq!(versions[1].notes.as_deref(), Some("first draft"));
}

#[test]
fn prompt_repository_rejects_invalid_json_without_persisting_document() {
    use crate::application::ports::PromptRepository;

    let root = test_root("prompt-invalid-json");
    let registry = test_root("prompt-invalid-json-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Prompt Invalid JSON".to_string(),
        })
        .expect("init library");

    let error = service
        .create_prompt_document(CreatePromptDocumentRequest {
            library_path: root.clone(),
            name: "Broken".to_string(),
            draft_body: "A {{subject}} study".to_string(),
            draft_negative_prompt: None,
            draft_style_prompt: None,
            variables_schema_json: "{not-json".to_string(),
            default_values_json: "{}".to_string(),
            parameter_preset_json: "{}".to_string(),
            notes: None,
        })
        .expect_err("invalid json should fail");

    assert!(error.to_string().contains("variables_schema_json"));

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let prompt_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM prompt_documents", [], |row| {
            row.get(0)
        })
        .expect("prompt count");
    assert_eq!(prompt_count, 0);
}

#[test]
fn generation_event_round_trips_prompt_version_id_through_lineage() {
    use crate::application::ports::AssetRepository;

    let root = test_root("prompt-generation-event-link");
    let registry = test_root("prompt-generation-event-link-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Prompt Event Link".to_string(),
        })
        .expect("init library");

    let source = test_root("prompt-generation-event-link-source").join("image.png");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, png_bytes(32, 32)).expect("write png");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    let prompt_version_id = PromptVersionId(Uuid::new_v4().to_string());
    let event = AssetService::record_generation_event(
        &service,
        CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(asset.id.clone()),
            output_version_id: Some(version.id.clone()),
            provider: "fake".to_string(),
            provider_model: "fake-model".to_string(),
            operation_type: GenerationOperation::TextToImage,
            prompt: "linked prompt".to_string(),
            negative_prompt: None,
            input_asset_version_id: None,
            prompt_version_id: Some(prompt_version_id.clone()),
            parameters_json: "{}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        },
    )
    .expect("record generation event");
    service
        .mark_version_generated(&root, &asset.id, &version.id, &event.id)
        .expect("mark generated");

    let lineage = service
        .get_lineage(&root, &version.id)
        .expect("load lineage");

    assert_eq!(
        lineage[0]
            .generation_event
            .as_ref()
            .and_then(|event| event.prompt_version_id.as_ref()),
        Some(&prompt_version_id)
    );
}

#[test]
fn creates_managed_library_layout_and_registry() {
    let root = test_root("create-library");
    let registry = test_root("registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);

    let summary = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Test Library".to_string(),
        })
        .expect("create library");

    assert_eq!(summary.name, "Test Library");
    assert!(LocalLibraryService::manifest_path(&root).is_file());
    assert!(LocalLibraryService::database_path(&root).is_file());

    for relative in REQUIRED_DIRS {
        assert!(root.join(relative).is_dir(), "missing {relative}");
    }

    let libraries = service.list_libraries(false).expect("list libraries");
    assert_eq!(libraries.len(), 1);
    assert_eq!(libraries[0].id, summary.id);
}

#[test]
fn studio_overview_summarizes_library_workflow_state() {
    let root = test_root("studio-overview");
    let registry = test_root("studio-overview-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Studio".to_string(),
        })
        .expect("create library");
    let source = test_root("studio-overview-source").join("image.png");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, png_bytes(64, 48)).expect("write png");
    let (asset, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    service
        .create_suggestion(CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: asset.id,
            source: "test".to_string(),
            suggested_title: Some("Suggested title".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec!["tag".to_string()],
            suggested_category: None,
            confidence_json: "{}".to_string(),
        })
        .expect("create suggestion");
    let tasks = service
        .create_tasks(BatchCreateTasksRequest {
            library_path: root.clone(),
            library_id: library.id.clone(),
            tasks: vec![
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 1,
                    concurrency_group: Some("fake".to_string()),
                    max_attempts: 3,
                    input_json: "{}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::MetadataSuggestionGeneration,
                    provider: None,
                    operation: None,
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 1,
                    input_json: "{}".to_string(),
                },
            ],
        })
        .expect("create tasks");
    service
        .update_task_status(UpdateTaskStatusRequest {
            library_path: root.clone(),
            task_id: tasks[1].id.clone(),
            status: TaskStatus::FailedFinal,
            next_retry_at: None,
            last_error_code: Some("provider_error".to_string()),
            last_error_message: Some("failed".to_string()),
            error_classification: None,
            wait_reason: None,
        })
        .expect("fail task");

    let overview = service.studio_overview(&root).expect("studio overview");

    assert_eq!(overview.library.id, library.id);
    assert_eq!(overview.status.integrity_status, "healthy");
    assert_eq!(overview.registered_library_count, 1);
    assert_eq!(overview.missing_library_count, 0);
    assert_eq!(overview.review_pending_count, 1);
    assert_eq!(overview.task_summary.active_count, 1);
    assert_eq!(overview.task_summary.queued_count, 1);
    assert_eq!(overview.task_summary.failed_count, 1);
    assert!(overview
        .provider_health
        .iter()
        .any(|provider| provider.provider == "codex-cli"));
}

#[test]
fn hides_registered_library_without_deleting_files() {
    let root = test_root("hide-library");
    let registry = test_root("hide-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let summary = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Hidden Library".to_string(),
        })
        .expect("create library");

    service.hide_library(&summary.id).expect("hide library");

    assert!(root.exists());
    assert!(service.list_libraries(false).expect("visible").is_empty());
    assert_eq!(service.list_libraries(true).expect("all").len(), 1);
}

#[test]
fn rejects_future_schema_manifest() {
    let root = test_root("future-schema");
    let registry = test_root("future-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Future".to_string(),
        })
        .expect("create library");

    let mut manifest = LocalLibraryService::read_manifest(&root).expect("read manifest");
    manifest.schema_version = CURRENT_SCHEMA_VERSION + 1;
    LocalLibraryService::write_manifest(&root, &manifest).expect("write manifest");

    let error = service
        .open_library(&root)
        .expect_err("future schema should fail");
    assert!(matches!(error, DomainError::SchemaMismatch { .. }));
}

#[test]
fn migrates_legacy_asset_versions_to_numeric_versions() {
    let root = test_root("legacy-version-number");
    let registry = test_root("legacy-version-number-registry").join("registry.sqlite");
    let source_dir = test_root("legacy-version-number-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    let generated = source_dir.join("generated.png");
    fs::write(&source, b"input bytes").expect("write source");
    fs::write(&generated, b"generated bytes").expect("write generated");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Legacy Version Number".to_string(),
        })
        .expect("create library");
    let (asset, parent) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import parent");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: parent.id.clone(),
            generation_event_id: None,
            source_path: generated,
            mime_type: "image/png".to_string(),
            version_label: Some("variant".to_string()),
        })
        .expect("create child");

    {
        let connection =
            Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
        connection
            .execute(
                "DROP INDEX IF EXISTS idx_asset_versions_asset_version_number",
                [],
            )
            .expect("drop version index");
        connection
            .execute("ALTER TABLE asset_versions DROP COLUMN version_number", [])
            .expect("drop version number column");
        connection
            .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION - 1)
            .expect("downgrade user version");
    }

    service.open_library(&root).expect("migrate library");
    let migrated_parent = service
        .get_asset_detail(&root, &asset.id, Some(&parent.id))
        .expect("parent detail");
    let migrated_child = migrated_parent
        .versions
        .iter()
        .find(|version| version.id == child.id)
        .expect("child version");
    let migrated_parent_version = migrated_parent
        .versions
        .iter()
        .find(|version| version.id == parent.id)
        .expect("parent version");

    assert_eq!(migrated_parent_version.version_number, 1);
    assert_eq!(migrated_parent_version.version_name, "v1");
    assert_eq!(migrated_child.version_number, 2);
    assert_eq!(migrated_child.version_name, "v2");
    assert_eq!(migrated_child.parent_version_id.as_ref(), Some(&parent.id));
}

#[test]
fn imports_asset_into_managed_originals() {
    let root = test_root("import-asset");
    let registry = test_root("import-registry").join("registry.sqlite");
    let source_dir = test_root("import-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"not really a png").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Import".to_string(),
        })
        .expect("create library");

    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    assert_eq!(asset.status, "imported");
    assert_eq!(version.mime_type, "image/png");
    assert!(version.generation_event_id.is_none());
    assert_eq!(version.version_number, 1);
    assert_eq!(version.version_name, "v1");
    assert!(root.join(&version.file_path).is_file());
    assert_eq!(version.checksum_algorithm, CHECKSUM_SHA256);
    assert_eq!(
        version.checksum,
        "e90137d39de304eefbbe788bc535c7e82f27abbf8069505fbbd8a9dcdc4f2024"
    );
    let path_parts = version
        .file_path
        .iter()
        .map(|part| part.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(path_parts[0], "originals");
    assert_eq!(path_parts[1].len(), 4);
    assert_eq!(path_parts[2].len(), 2);
    assert!(path_parts[3].ends_with(".png"));
    assert!(Uuid::parse_str(path_parts[3].trim_end_matches(".png")).is_ok());

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let asset_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))
        .expect("asset count");
    let version_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM asset_versions", [], |row| row.get(0))
        .expect("version count");
    assert_eq!(asset_count, 1);
    assert_eq!(version_count, 1);
}

#[test]
fn duplicate_source_filenames_get_distinct_uuid_paths() {
    let root = test_root("duplicate-filenames");
    let registry = test_root("duplicate-filenames-registry").join("registry.sqlite");
    let first_dir = test_root("duplicate-source-a");
    let second_dir = test_root("duplicate-source-b");
    fs::create_dir_all(&first_dir).expect("create first source dir");
    fs::create_dir_all(&second_dir).expect("create second source dir");
    let first = first_dir.join("input.png");
    let second = second_dir.join("input.png");
    fs::write(&first, b"first").expect("write first");
    fs::write(&second, b"second").expect("write second");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Duplicates".to_string(),
        })
        .expect("create library");

    let (_, first_version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: first,
        })
        .expect("import first");
    let (_, second_version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: second,
        })
        .expect("import second");

    assert_ne!(first_version.file_path, second_version.file_path);
    assert!(root.join(first_version.file_path).is_file());
    assert!(root.join(second_version.file_path).is_file());
}

#[test]
fn imports_png_dimensions_into_file_context() {
    let root = test_root("import-dimensions");
    let registry = test_root("import-dimensions-registry").join("registry.sqlite");
    let source_dir = test_root("import-dimensions-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(640, 480)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Dimensions".to_string(),
        })
        .expect("create library");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&version.id))
        .expect("detail");
    let file = detail.file.expect("file");
    assert_eq!(file.width, Some(640));
    assert_eq!(file.height, Some(480));

    let gallery = service
        .query_gallery(&root, GalleryQuery::default())
        .expect("gallery");
    assert_eq!(gallery[0].width, Some(640));
    assert_eq!(gallery[0].height, Some(480));
}

#[test]
fn unknown_binary_keeps_dimensions_empty() {
    let root = test_root("unknown-dimensions");
    let registry = test_root("unknown-dimensions-registry").join("registry.sqlite");
    let source_dir = test_root("unknown-dimensions-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"not really a png").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Unknown Dimensions".to_string(),
        })
        .expect("create library");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&version.id))
        .expect("detail");
    let file = detail.file.expect("file");
    assert_eq!(file.width, None);
    assert_eq!(file.height, None);
}

#[test]
fn repair_library_dry_run_reports_legacy_path_checksum_and_dimensions() {
    let root = test_root("repair-dry-run");
    let registry = test_root("repair-dry-run-registry").join("registry.sqlite");
    let source_dir = test_root("repair-dry-run-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(640, 480)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Repair".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    let old_relative = PathBuf::from("originals")
        .join("imported")
        .join(format!("{}.png", version.id.0));
    move_version_to_legacy_state(&root, &version.id, &version.file_path, &old_relative);

    let summary = service
        .repair_library(RepairLibraryRequest {
            library_path: root.clone(),
            dry_run: true,
        })
        .expect("repair dry run");

    assert_eq!(summary.scanned_versions, 1);
    assert_eq!(summary.files_moved, 1);
    assert_eq!(summary.paths_updated, 1);
    assert_eq!(summary.checksums_updated, 1);
    assert_eq!(summary.dimensions_updated, 1);
    assert!(summary.issues.is_empty());
    assert!(root.join(&old_relative).is_file());
    assert!(!root.join(&version.file_path).is_file());
}

#[test]
fn repair_library_applies_legacy_path_checksum_and_dimensions() {
    let root = test_root("repair-apply");
    let registry = test_root("repair-apply-registry").join("registry.sqlite");
    let source_dir = test_root("repair-apply-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(800, 600)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Repair".to_string(),
        })
        .expect("create library");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    let old_relative = PathBuf::from("originals")
        .join("generated")
        .join(format!("{}.png", version.id.0));
    move_version_to_legacy_state(&root, &version.id, &version.file_path, &old_relative);

    let summary = service
        .repair_library(RepairLibraryRequest {
            library_path: root.clone(),
            dry_run: false,
        })
        .expect("repair apply");

    assert_eq!(summary.files_moved, 1);
    assert_eq!(summary.paths_updated, 1);
    assert_eq!(summary.checksums_updated, 1);
    assert_eq!(summary.dimensions_updated, 1);
    assert!(summary.issues.is_empty());
    assert!(!root.join(&old_relative).exists());
    assert!(root.join(&version.file_path).is_file());

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&version.id))
        .expect("detail");
    let file = detail.file.expect("file");
    assert_eq!(file.relative_location, version.file_path);
    assert_eq!(file.checksum_algorithm, CHECKSUM_SHA256);
    assert_eq!(file.width, Some(800));
    assert_eq!(file.height, Some(600));
    assert!(service
        .check_integrity(&root)
        .expect("integrity")
        .is_empty());
}

#[test]
fn repair_library_reports_missing_files_without_deleting_records() {
    let root = test_root("repair-missing");
    let registry = test_root("repair-missing-registry").join("registry.sqlite");
    let source_dir = test_root("repair-missing-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(16, 16)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Repair Missing".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    fs::remove_file(root.join(&version.file_path)).expect("remove managed file");

    let summary = service
        .repair_library(RepairLibraryRequest {
            library_path: root.clone(),
            dry_run: false,
        })
        .expect("repair apply");

    assert_eq!(summary.scanned_versions, 1);
    assert_eq!(summary.issues.len(), 1);
    assert_eq!(summary.issues[0].version_id, version.id);

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM asset_versions", [], |row| row.get(0))
        .expect("count versions");
    assert_eq!(count, 1);
}

#[test]
fn exports_imported_asset_with_sidecar() {
    let root = test_root("export-library");
    let registry = test_root("export-registry").join("registry.sqlite");
    let export_root = test_root("export-output");
    let source_dir = test_root("export-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.jpg");
    fs::write(&source, b"jpg bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Export".to_string(),
        })
        .expect("create library");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    let summary = service
        .export_library(ExportLibraryRequest {
            library_path: root,
            output_path: export_root.clone(),
            album_id: None,
        })
        .expect("export library");

    assert_eq!(summary.exported_files, 1);
    assert_eq!(summary.exported_sidecars, 1);
    assert!(export_root
        .join("originals")
        .join(format!("{}.jpg", version.id.0))
        .is_file());
    assert!(export_root
        .join("sidecars")
        .join(format!("{}.json", asset.id.0))
        .is_file());
}

#[test]
fn renames_registry_alias_without_changing_manifest() {
    let root = test_root("rename-library-alias");
    let registry = test_root("rename-library-alias-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let created = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Manifest Name".to_string(),
        })
        .expect("create library");
    let original_manifest =
        LocalLibraryService::read_manifest(&root).expect("read original manifest");

    let renamed = service
        .rename_library_alias(RenameLibraryAliasRequest {
            library_id: created.id.clone(),
            alias: "Local Alias".to_string(),
        })
        .expect("rename alias");

    assert_eq!(renamed.name, "Local Alias");
    let manifest = LocalLibraryService::read_manifest(&root).expect("read manifest");
    assert_eq!(manifest.name, original_manifest.name);
    assert_eq!(manifest.id, original_manifest.id);
    let libraries = service.list_libraries(false).expect("list libraries");
    assert_eq!(libraries[0].name, "Local Alias");
}

#[test]
fn rejects_empty_registry_alias() {
    let root = test_root("rename-empty-library-alias");
    let registry = test_root("rename-empty-library-alias-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let created = service
        .create_library(CreateLibraryRequest {
            root_path: root,
            name: "Alias".to_string(),
        })
        .expect("create library");

    let error = service
        .rename_library_alias(RenameLibraryAliasRequest {
            library_id: created.id,
            alias: "   ".to_string(),
        })
        .expect_err("empty alias rejected");

    assert_eq!(error.code(), "InvalidLibraryAlias");
}

#[test]
fn unregisters_library_without_deleting_files() {
    let root = test_root("unregister-library");
    let registry = test_root("unregister-library-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let created = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Unregister".to_string(),
        })
        .expect("create library");

    service
        .unregister_library(&created.id)
        .expect("unregister library");

    assert!(root.join(MANIFEST_FILE).is_file());
    assert!(root.join(DATABASE_FILE).is_file());
    assert!(service
        .list_libraries(true)
        .expect("list libraries")
        .is_empty());
}

#[test]
fn unregister_unknown_library_returns_not_found() {
    let registry = test_root("unregister-missing-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);

    let error = service
        .unregister_library(&LibraryId("missing".to_string()))
        .expect_err("missing library");

    assert_eq!(error.code(), "LibraryNotFound");
}

#[test]
fn exports_and_imports_library_backup_zip() {
    let root = test_root("backup-export-library");
    let registry = test_root("backup-export-registry").join("registry.sqlite");
    let zip_path = test_root("backup-export-output").join("library.zip");
    let import_root = test_root("backup-import-library");
    let import_registry = test_root("backup-import-registry").join("registry.sqlite");
    let source_dir = test_root("backup-export-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(10, 10)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Backup".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    service
        .export_library_backup_zip(ExportLibraryBackupRequest {
            library_path: root.clone(),
            output_zip_path: zip_path.clone(),
        })
        .expect("export backup");

    assert!(zip_path.is_file());

    let import_service = LocalLibraryService::new(import_registry);
    let summary = import_service
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path,
            destination_path: import_root.clone(),
        })
        .expect("import backup");

    assert!(!summary.cloned);
    assert!(import_root.join(MANIFEST_FILE).is_file());
    assert!(import_root.join(DATABASE_FILE).is_file());
    assert!(import_root.join(version.file_path).is_file());
    import_service
        .open_library(&import_root)
        .expect("open imported library");
}

#[test]
fn import_backup_zip_clones_conflicting_library_id() {
    let root = test_root("backup-conflict-library");
    let registry = test_root("backup-conflict-registry").join("registry.sqlite");
    let zip_path = test_root("backup-conflict-output").join("library.zip");
    let import_root = test_root("backup-conflict-import");
    let service = LocalLibraryService::new(registry);
    let created = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Conflict".to_string(),
        })
        .expect("create library");
    service
        .export_library_backup_zip(ExportLibraryBackupRequest {
            library_path: root,
            output_zip_path: zip_path.clone(),
        })
        .expect("export backup");

    let summary = service
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path,
            destination_path: import_root.clone(),
        })
        .expect("import backup");

    assert!(summary.cloned);
    assert_ne!(summary.library.id, created.id);
    let manifest = LocalLibraryService::read_manifest(&import_root).expect("read manifest");
    assert_eq!(manifest.id, summary.library.id.0);
    assert!(manifest.name.ends_with(" Copy"));
}

#[test]
fn import_invalid_backup_zip_does_not_register_library() {
    let registry = test_root("backup-invalid-registry").join("registry.sqlite");
    let zip_dir = test_root("backup-invalid-zip");
    let zip_path = zip_dir.join("invalid.zip");
    let import_root = test_root("backup-invalid-import");
    fs::create_dir_all(&zip_dir).expect("create zip dir");
    fs::write(&zip_path, b"not a zip").expect("write invalid zip");

    let service = LocalLibraryService::new(registry);
    let error = service
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path,
            destination_path: import_root,
        })
        .expect_err("invalid zip");

    assert_eq!(error.code(), "InvalidLibraryBackup");
    assert!(service
        .list_libraries(true)
        .expect("list libraries")
        .is_empty());
}

#[test]
fn import_backup_zip_rejects_non_empty_destination() {
    let registry = test_root("backup-non-empty-registry").join("registry.sqlite");
    let zip_path = test_root("backup-non-empty-zip").join("library.zip");
    let import_root = test_root("backup-non-empty-import");
    fs::create_dir_all(&import_root).expect("create import dir");
    fs::write(import_root.join("existing.txt"), b"existing").expect("write existing");

    let service = LocalLibraryService::new(registry);
    let error = service
        .import_library_backup_zip(ImportLibraryBackupRequest {
            zip_path,
            destination_path: import_root,
        })
        .expect_err("non-empty destination");

    assert_eq!(error.code(), "ImportDestinationNotEmpty");
}

#[test]
fn reports_missing_managed_file() {
    let root = test_root("integrity-missing");
    let registry = test_root("integrity-registry").join("registry.sqlite");
    let source_dir = test_root("integrity-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.webp");
    fs::write(&source, b"webp bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Integrity".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    fs::remove_file(root.join(&version.file_path)).expect("remove managed file");

    let issues = service.check_integrity(&root).expect("check integrity");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].version_id, version.id);
    assert_eq!(issues[0].kind, IntegrityIssueKind::MissingFile);
}

#[test]
fn reports_sha256_mismatch_for_modified_file() {
    let root = test_root("integrity-sha256-mismatch");
    let registry = test_root("integrity-sha256-mismatch-registry").join("registry.sqlite");
    let source_dir = test_root("integrity-sha256-mismatch-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.webp");
    fs::write(&source, b"webp bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Integrity".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");

    fs::write(root.join(&version.file_path), b"changed").expect("modify managed file");

    let issues = service.check_integrity(&root).expect("check integrity");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].version_id, version.id);
    assert_eq!(issues[0].kind, IntegrityIssueKind::HashMismatch);
    assert!(issues[0].message.contains(CHECKSUM_SHA256));
}

#[test]
fn legacy_md5_versions_remain_readable_before_repair() {
    let root = test_root("legacy-md5");
    let registry = test_root("legacy-md5-registry").join("registry.sqlite");
    let source_dir = test_root("legacy-md5-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"legacy bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Legacy".to_string(),
        })
        .expect("create library");
    let (asset, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    let md5 = file_digest(&root.join(&version.file_path), CHECKSUM_MD5).expect("md5");
    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    connection
        .execute(
            "
            UPDATE asset_versions
            SET sha256 = ?1,
                checksum_algorithm = 'MD5',
                checksum = ?1
            WHERE id = ?2
            ",
            params![md5, version.id.0],
        )
        .expect("make legacy row");

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&version.id))
        .expect("detail");
    let file = detail.file.expect("file context");
    assert_eq!(file.checksum_algorithm, CHECKSUM_MD5);
    assert_eq!(file.integrity_status, "verified");
    assert!(service
        .check_integrity(&root)
        .expect("integrity")
        .is_empty());
}

#[test]
fn records_event_and_child_version_lineage() {
    let root = test_root("lineage");
    let registry = test_root("lineage-registry").join("registry.sqlite");
    let source_dir = test_root("lineage-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    let generated = source_dir.join("generated.png");
    fs::write(&source, b"input bytes").expect("write source");
    fs::write(&generated, b"generated bytes").expect("write generated");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Lineage".to_string(),
        })
        .expect("create library");
    let (asset, parent) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import parent");
    let event = service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(asset.id.clone()),
            output_version_id: None,
            provider: "codex".to_string(),
            provider_model: "gpt-image-2".to_string(),
            operation_type: GenerationOperation::ImageToImage,
            prompt: "make a variant".to_string(),
            negative_prompt: None,
            input_asset_version_id: Some(parent.id.clone()),
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
            raw_request_json: Some("{\"prompt\":\"make a variant\"}".to_string()),
            raw_response_json: Some("{\"ok\":true}".to_string()),
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("record event");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id,
            parent_version_id: parent.id.clone(),
            generation_event_id: Some(event.id.clone()),
            source_path: generated,
            mime_type: "image/png".to_string(),
            version_label: Some("variant".to_string()),
        })
        .expect("create child");
    assert_eq!(parent.version_number, 1);
    assert_eq!(parent.version_name, "v1");
    assert_eq!(child.version_number, 2);
    assert_eq!(child.version_name, "v2");

    let lineage = service.get_lineage(&root, &child.id).expect("lineage");
    assert_eq!(lineage.len(), 2);
    assert_eq!(lineage[0].version.id, child.id);
    assert_eq!(
        lineage[0].generation_event.as_ref().expect("event").id,
        event.id
    );
    assert_eq!(
        lineage[0]
            .generation_event
            .as_ref()
            .expect("event")
            .output_version_id
            .as_ref(),
        Some(&child.id)
    );
    assert_eq!(lineage[1].version.id, parent.id);
    assert_eq!(lineage[1].version.version_number, 1);
    assert!(lineage[1].generation_event.is_none());
}

#[test]
fn generation_service_saves_fake_provider_output() {
    let root = test_root("generation-flow");
    let registry = test_root("generation-registry").join("registry.sqlite");
    let library = LocalLibraryService::new(registry);
    let library_summary = library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Generation".to_string(),
        })
        .expect("create library");

    let generation = LocalGenerationService::new(crate::FakeImageProvider::success("fake"));
    let versions = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: None,
            input_bytes: None,
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "fake".to_string(),
                model: "fake-image".to_string(),
                prompt: "make a test image".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::TextToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect("generate");

    assert_eq!(versions.len(), 1);
    assert!(root.join(&versions[0].file_path).is_file());
    assert!(versions[0].generation_event_id.is_some());
    assert_eq!(versions[0].version_number, 1);
    assert_eq!(versions[0].version_name, "v1");

    let gallery = library
        .query_gallery(
            &root,
            GalleryQuery {
                text: Some("test image".to_string()),
                providers: vec!["fake".to_string()],
                min_rating: None,
                review_status: ReviewStatusFilter::Any,
                tags: vec![],
                album_filter: GalleryAlbumFilter::Any,
                album_id: None,
                sort: GallerySort::Newest,
            },
        )
        .expect("gallery");
    assert_eq!(gallery.len(), 1);
    assert_eq!(gallery[0].id, versions[0].asset_id);
    assert_eq!(gallery[0].title.as_deref(), Some("Make Test Image"));
    assert_eq!(gallery[0].provider.as_deref(), Some("fake"));
    assert_eq!(gallery[0].model_label.as_deref(), Some("fake-image"));
    assert_eq!(gallery[0].prompt.as_deref(), Some("make a test image"));
    assert_eq!(gallery[0].review_pending_count, 1);

    let suggestions = library
        .list_pending(&root, &library_summary.id)
        .expect("pending suggestions");
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].asset_id, versions[0].asset_id);
    assert_eq!(
        suggestions[0].suggested_title.as_deref(),
        Some("Make Test Image")
    );
    assert!(suggestions[0]
        .suggested_schema_prompt
        .as_deref()
        .unwrap_or_default()
        .contains("\"primary_subject\": \"make a test image\""));

    let detail = library
        .get_asset_detail(&root, &versions[0].asset_id, Some(&versions[0].id))
        .expect("detail");
    assert_eq!(detail.provider.as_deref(), Some("fake"));
    assert_eq!(detail.title.as_deref(), Some("Make Test Image"));
    assert_eq!(detail.model_label.as_deref(), Some("fake-image"));
    assert_eq!(detail.prompt.as_deref(), Some("make a test image"));
    assert_eq!(detail.parameters_json.as_deref(), Some("{}"));
    assert_eq!(detail.review_pending_count, 1);
}

#[test]
fn generation_service_imports_uploaded_reference_as_separate_asset() {
    let root = test_root("uploaded-reference-generation");
    let registry = test_root("uploaded-reference-generation-registry").join("registry.sqlite");
    let source_dir = test_root("uploaded-reference-generation-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let reference_file = source_dir.join("reference.png");
    fs::write(&reference_file, png_bytes(32, 24)).expect("write reference");

    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Uploaded Reference".to_string(),
        })
        .expect("create library");

    let generation = LocalGenerationService::new(crate::FakeImageProvider::success("fake"));
    let versions = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: Some(reference_file),
            input_bytes: Some(png_bytes(32, 24)),
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "fake".to_string(),
                model: "fake-image".to_string(),
                prompt: "make a reference variant".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::ImageToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect("generate");

    assert_eq!(versions.len(), 1);
    let output = &versions[0];
    assert_eq!(output.version_number, 1);
    assert_eq!(output.version_name, "v1");
    assert!(output.parent_version_id.is_none());

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let reference_version_id: String = connection
        .query_row(
            "
            SELECT av.id
            FROM asset_versions av
            INNER JOIN assets a ON a.id = av.asset_id
            WHERE a.status = 'reference'
            ",
            [],
            |row| row.get(0),
        )
        .expect("reference version");
    let output_event_input_id: String = connection
        .query_row(
            "
            SELECT input_asset_version_id
            FROM generation_events
            WHERE output_version_id = ?1
            ",
            params![output.id.0],
            |row| row.get(0),
        )
        .expect("event input");
    assert_eq!(output_event_input_id, reference_version_id);

    let gallery = library
        .query_gallery(&root, GalleryQuery::default())
        .expect("gallery");
    assert_eq!(gallery.len(), 1);
    assert_eq!(gallery[0].id, output.asset_id);
    assert_eq!(gallery[0].current_version_number, Some(1));
    assert_eq!(gallery[0].current_version_name.as_deref(), Some("v1"));

    let detail = library
        .get_asset_detail(&root, &output.asset_id, Some(&output.id))
        .expect("output detail");
    let source_reference = detail.source_reference.expect("source reference");
    assert_eq!(source_reference.version_id.0, reference_version_id);
    assert_eq!(source_reference.version_number, 1);
    assert_eq!(source_reference.version_name, "v1");
}

#[test]
fn generation_service_creates_existing_version_variation() {
    let root = test_root("existing-version-generation");
    let registry = test_root("existing-version-generation-registry").join("registry.sqlite");
    let source_dir = test_root("existing-version-generation-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, png_bytes(16, 16)).expect("write source");

    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Existing Version".to_string(),
        })
        .expect("create library");
    let (asset, parent) = library
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import parent");

    let generation = LocalGenerationService::new(crate::FakeImageProvider::success("fake"));
    let versions = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: None,
            input_bytes: Some(png_bytes(16, 16)),
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "fake".to_string(),
                model: "fake-image".to_string(),
                prompt: "make a variation".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::ImageToImage,
                input_version_id: Some(parent.id.clone()),
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect("generate");

    assert_eq!(versions.len(), 1);
    let child = &versions[0];
    assert_eq!(child.asset_id, asset.id);
    assert_eq!(child.parent_version_id.as_ref(), Some(&parent.id));
    assert_eq!(child.version_number, 2);
    assert_eq!(child.version_name, "v2");
}

#[test]
fn uploaded_reference_provider_failure_keeps_reference_without_output() {
    let root = test_root("uploaded-reference-failure");
    let registry = test_root("uploaded-reference-failure-registry").join("registry.sqlite");
    let source_dir = test_root("uploaded-reference-failure-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let reference_file = source_dir.join("reference.png");
    fs::write(&reference_file, png_bytes(32, 24)).expect("write reference");

    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Uploaded Reference Failure".to_string(),
        })
        .expect("create library");

    let generation = LocalGenerationService::new(crate::FakeImageProvider::failure("fake", "boom"));
    let error = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: Some(reference_file),
            input_bytes: Some(png_bytes(32, 24)),
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "fake".to_string(),
                model: "fake-image".to_string(),
                prompt: "make a reference variant".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::ImageToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect_err("provider should fail");
    assert!(matches!(error, DomainError::GenerationFailed { .. }));

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let reference_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE status = 'reference'",
            [],
            |row| row.get(0),
        )
        .expect("reference count");
    let output_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE status = 'generated'",
            [],
            |row| row.get(0),
        )
        .expect("output count");
    let failed_event_count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM generation_events
            WHERE status = 'failed'
              AND input_asset_version_id IS NOT NULL
              AND output_version_id IS NULL
            ",
            [],
            |row| row.get(0),
        )
        .expect("failed event count");
    assert_eq!(reference_count, 1);
    assert_eq!(output_count, 0);
    assert_eq!(failed_event_count, 1);
}

#[derive(Debug, Clone)]
struct PngProvider;

impl ImageProvider for PngProvider {
    fn name(&self) -> &'static str {
        "png-provider"
    }

    fn validate_parameters(&self, _parameters: &crate::GenerationParameters) -> DomainResult<()> {
        Ok(())
    }

    fn generate_from_text(
        &self,
        _parameters: &crate::GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        Ok(GenerationResult {
            images: vec![GeneratedImage {
                bytes: png_bytes(320, 240),
                mime_type: "image/png".to_string(),
                provider_metadata_json: "{}".to_string(),
            }],
            raw_request_json: "{}".to_string(),
            raw_response_json: "{}".to_string(),
        })
    }

    fn generate_from_image(
        &self,
        _parameters: &crate::GenerationParameters,
        _input: &[u8],
    ) -> DomainResult<GenerationResult> {
        self.generate_from_text(_parameters)
    }
}

#[test]
fn generation_service_persists_output_dimensions() {
    let root = test_root("generation-dimensions");
    let registry = test_root("generation-dimensions-registry").join("registry.sqlite");
    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Generation Dimensions".to_string(),
        })
        .expect("create library");

    let generation = LocalGenerationService::new(PngProvider);
    let versions = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: None,
            input_bytes: None,
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "png-provider".to_string(),
                model: "png-image".to_string(),
                prompt: "make a png".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::TextToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect("generate");

    let detail = library
        .get_asset_detail(&root, &versions[0].asset_id, Some(&versions[0].id))
        .expect("detail");
    let file = detail.file.expect("file");
    assert_eq!(file.width, Some(320));
    assert_eq!(file.height, Some(240));
}

#[test]
fn metadata_suggestions_are_review_first() {
    let root = test_root("metadata-review");
    let registry = test_root("metadata-registry").join("registry.sqlite");
    let source_dir = test_root("metadata-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"image bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Metadata".to_string(),
        })
        .expect("create library");
    let (asset, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import asset");
    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            title: None,
            description: None,
            schema_prompt: None,
            rating: None,
            category: Some("category-a".to_string()),
            status: None,
        })
        .expect("seed category");

    let suggestion = service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            source: "fake".to_string(),
            suggested_title: Some("Suggested title".to_string()),
            suggested_description: Some("Suggested description".to_string()),
            suggested_schema_prompt: Some("{\"OUTPUT\":{\"mood\":\"edited\"}}".to_string()),
            suggested_tags: vec!["tag-a".to_string(), "tag-b".to_string()],
            suggested_category: Some("category-a".to_string()),
            confidence_json: "{}".to_string(),
        })
        .expect("create suggestion");

    let before = load_asset_summary(
        &Connection::open(LocalLibraryService::database_path(&root)).expect("open db"),
        &asset.id,
    )
    .expect("load asset");
    assert!(before.title.is_none());

    let pending = service
        .list_pending(&root, &library.id)
        .expect("list pending");
    assert_eq!(pending.len(), 1);

    let accepted = service
        .accept(ReviewMetadataSuggestionRequest {
            library_path: root.clone(),
            suggestion_id: suggestion.id,
            title: Some("Edited title".to_string()),
            description: Some("Edited description".to_string()),
            schema_prompt: Some("{\"OUTPUT\":{\"mood\":\"edited\"}}".to_string()),
            tags: vec!["tag-a".to_string()],
            category: Some("category-a".to_string()),
        })
        .expect("accept");

    assert_eq!(accepted.title.as_deref(), Some("Edited title"));
    let detail = service
        .get_asset_detail(&root, &asset.id, None)
        .expect("detail");
    assert_eq!(detail.description.as_deref(), Some("Edited description"));
    assert_eq!(
        detail.schema_prompt.as_deref(),
        Some("{\"OUTPUT\":{\"mood\":\"edited\"}}")
    );
    assert!(service
        .list_pending(&root, &library.id)
        .expect("pending")
        .is_empty());
}

#[test]
fn manages_manual_album_and_searches_assets() {
    let root = test_root("albums-search");
    let registry = test_root("albums-registry").join("registry.sqlite");
    let source_dir = test_root("albums-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"image bytes").expect("write source");

    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Albums".to_string(),
        })
        .expect("create library");
    let (asset, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import");

    assert!(service
        .list_albums(&library.id)
        .expect("list empty albums")
        .is_empty());

    let album = service
        .create_manual_album(&library.id, "Favorites")
        .expect("album");
    let albums = service.list_albums(&library.id).expect("list albums");
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].id, album.id);
    assert_eq!(albums[0].name, "Favorites");
    assert_eq!(albums[0].kind, AlbumKind::Manual);
    assert_eq!(albums[0].item_count, 0);

    service.add_asset(&album.id, &asset.id).expect("add asset");
    service
        .add_asset(&album.id, &asset.id)
        .expect("duplicate add is no-op");
    let albums = service.list_albums(&library.id).expect("list albums");
    assert_eq!(albums[0].item_count, 1);
    let tasks = service
        .create_tasks(BatchCreateTasksRequest {
            library_path: root.clone(),
            library_id: library.id.clone(),
            tasks: vec![CreateTaskInput {
                task_type: TaskType::ImageGeneration,
                provider: Some("fake".to_string()),
                operation: Some(GenerationOperation::TextToImage),
                priority: 0,
                concurrency_group: Some("fake".to_string()),
                max_attempts: 1,
                input_json: "{}".to_string(),
            }],
        })
        .expect("create task");
    service
        .append_task_output(AppendTaskOutputRequest {
            library_path: root.clone(),
            task_id: tasks[0].id.clone(),
            output_type: TaskOutputType::Asset,
            target_id: asset.id.0.clone(),
            payload_json: None,
        })
        .expect("append output");

    let album_results = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_id: Some(album.id.clone()),
                ..GalleryQuery::default()
            },
        )
        .expect("query album gallery");
    assert_eq!(album_results.len(), 1);
    assert_eq!(album_results[0].id, asset.id);
    assert_eq!(album_results[0].albums.len(), 1);
    assert_eq!(album_results[0].albums[0].id, album.id);
    assert_eq!(
        album_results[0]
            .album_context
            .as_ref()
            .expect("album context")
            .id,
        album.id
    );
    assert_eq!(
        album_results[0]
            .task_origin
            .as_ref()
            .expect("task origin")
            .task_id,
        tasks[0].id
    );

    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            title: None,
            description: None,
            schema_prompt: None,
            rating: Some(5),
            category: Some("icons".to_string()),
            status: Some("curated".to_string()),
        })
        .expect("update metadata");

    let results = service
        .search(
            &library.id,
            SearchQuery {
                text: None,
                tags: vec![],
                min_rating: Some(4),
                provider: None,
                status: Some("curated".to_string()),
                category: Some("icons".to_string()),
            },
        )
        .expect("search");

    assert_eq!(results.len(), 1);

    service
        .add_tag_to_asset(&root, &asset.id, "favorite")
        .expect("tag asset");
    service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(asset.id.clone()),
            output_version_id: None,
            provider: "fake".to_string(),
            provider_model: "fake-image".to_string(),
            operation_type: GenerationOperation::TextToImage,
            prompt: "tiny icon sheet".to_string(),
            negative_prompt: None,
            input_asset_version_id: None,
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("record event");

    let prompt_results = service
        .search(
            &library.id,
            SearchQuery {
                text: Some("icon sheet".to_string()),
                tags: vec![],
                min_rating: None,
                provider: Some("fake".to_string()),
                status: None,
                category: None,
            },
        )
        .expect("search by prompt");
    assert_eq!(prompt_results.len(), 1);

    let tag_text_results = service
        .search(
            &library.id,
            SearchQuery {
                text: Some("favor".to_string()),
                tags: vec![],
                min_rating: None,
                provider: None,
                status: None,
                category: None,
            },
        )
        .expect("search by tag text");
    assert_eq!(tag_text_results.len(), 1);
}

#[test]
fn gallery_query_filters_and_sorts_cards() {
    let root = test_root("gallery-query");
    let registry = test_root("gallery-registry").join("registry.sqlite");
    let source_dir = test_root("gallery-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let first_source = source_dir.join("first.png");
    let second_source = source_dir.join("second.png");
    fs::write(&first_source, b"first image").expect("write first");
    fs::write(&second_source, b"second image").expect("write second");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Gallery".to_string(),
        })
        .expect("create library");
    let (first, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: first_source,
        })
        .expect("import first");
    let (second, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: second_source,
        })
        .expect("import second");

    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: first.id.clone(),
            title: None,
            description: None,
            schema_prompt: None,
            rating: Some(5),
            category: Some("botanical".to_string()),
            status: Some("curated".to_string()),
        })
        .expect("update first");
    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: second.id,
            title: None,
            description: None,
            schema_prompt: None,
            rating: Some(2),
            category: Some("city".to_string()),
            status: Some("imported".to_string()),
        })
        .expect("update second");
    service
        .add_tag_to_asset(&root, &first.id, "neon")
        .expect("tag first");
    service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(first.id.clone()),
            output_version_id: None,
            provider: "codex-cli".to_string(),
            provider_model: "codex-imagegen".to_string(),
            operation_type: GenerationOperation::TextToImage,
            prompt: "neon botanical study".to_string(),
            negative_prompt: None,
            input_asset_version_id: None,
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("record event");
    service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: first.id.clone(),
            source: "fake".to_string(),
            suggested_title: Some("Neon Botanical Study".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec!["neon".to_string()],
            suggested_category: Some("botanical".to_string()),
            confidence_json: "{}".to_string(),
        })
        .expect("create suggestion");

    let results = service
        .query_gallery(
            &root,
            GalleryQuery {
                text: Some("botanical".to_string()),
                providers: vec!["codex-cli".to_string()],
                min_rating: Some(4),
                review_status: ReviewStatusFilter::Pending,
                tags: vec!["neon".to_string()],
                album_filter: GalleryAlbumFilter::Any,
                album_id: None,
                sort: GallerySort::RatingDesc,
            },
        )
        .expect("query gallery");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, first.id);
    assert_eq!(results[0].provider.as_deref(), Some("codex-cli"));
    assert_eq!(results[0].review_pending_count, 1);
    assert_eq!(results[0].tags, vec!["neon".to_string()]);
}

#[test]
fn gallery_query_aggregates_versions_events_tags_and_review_counts() {
    let root = test_root("gallery-aggregate");
    let registry = test_root("gallery-aggregate-registry").join("registry.sqlite");
    let source_dir = test_root("gallery-aggregate-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("source.png");
    let child_source = source_dir.join("child.png");
    fs::write(&source, png_bytes(12, 8)).expect("write source");
    fs::write(&child_source, png_bytes(24, 16)).expect("write child");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Gallery Aggregate".to_string(),
        })
        .expect("create library");
    let (asset, parent) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import");
    service
        .add_tag_to_asset(&root, &asset.id, "alpha")
        .expect("tag alpha");
    service
        .add_tag_to_asset(&root, &asset.id, "beta")
        .expect("tag beta");
    let event = service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(asset.id.clone()),
            output_version_id: None,
            provider: "codex-cli".to_string(),
            provider_model: "codex-imagegen".to_string(),
            operation_type: GenerationOperation::ImageToImage,
            prompt: "aggregate prompt".to_string(),
            negative_prompt: None,
            input_asset_version_id: Some(parent.id.clone()),
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("event");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: parent.id,
            generation_event_id: Some(event.id),
            source_path: child_source,
            mime_type: "image/png".to_string(),
            version_label: Some("variant".to_string()),
        })
        .expect("child");
    service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            source: "test".to_string(),
            suggested_title: Some("Aggregate".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec!["alpha".to_string()],
            suggested_category: None,
            confidence_json: "{}".to_string(),
        })
        .expect("suggestion");

    let items = service
        .query_gallery(
            &root,
            GalleryQuery {
                text: Some("aggregate".to_string()),
                providers: vec!["codex-cli".to_string()],
                tags: vec!["alpha".to_string(), "beta".to_string()],
                review_status: ReviewStatusFilter::Pending,
                ..GalleryQuery::default()
            },
        )
        .expect("query gallery");

    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.id, asset.id);
    assert_eq!(item.current_version_id.as_ref(), Some(&child.id));
    assert_eq!(item.provider.as_deref(), Some("codex-cli"));
    assert_eq!(item.model_label.as_deref(), Some("codex-imagegen"));
    assert_eq!(item.prompt.as_deref(), Some("aggregate prompt"));
    assert_eq!(item.tags, vec!["alpha".to_string(), "beta".to_string()]);
    assert_eq!(item.review_pending_count, 1);
    assert_eq!(item.version_count, 2);
    assert_eq!(item.version_label.as_deref(), Some("variant"));
    assert_eq!(item.width, Some(24));
    assert_eq!(item.height, Some(16));
}

#[test]
#[ignore = "synthetic SQLite sufficiency checkpoint for 10k gallery/search assets"]
fn synthetic_gallery_search_checkpoint_10k_assets() {
    let root = test_root("gallery-checkpoint-10k");
    let registry = test_root("gallery-checkpoint-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Gallery Checkpoint".to_string(),
        })
        .expect("create library");
    let database_path = LocalLibraryService::database_path(&root);
    let mut connection = Connection::open(database_path).expect("open database");
    let now = "2026-05-18T00:00:00Z";

    let seed_start = Instant::now();
    let transaction = connection.transaction().expect("begin transaction");
    for index in 0..8 {
        transaction
            .execute(
                "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, NULL, ?3)",
                params![format!("tag-{index}"), format!("tag-{index}"), now],
            )
            .expect("insert tag");
    }
    transaction
        .execute(
            "INSERT INTO albums (id, name, description, kind, smart_query_json, sort_order, created_at, updated_at)
             VALUES ('album-1', 'Synthetic', NULL, 'manual', NULL, 1, ?1, ?1)",
            params![now],
        )
        .expect("insert album");

    for index in 0..10_000 {
        let asset_id = format!("asset-{index:05}");
        let version_id = format!("version-{index:05}");
        let event_id = format!("event-{index:05}");
        let tag_a = format!("tag-{}", index % 8);
        let tag_b = format!("tag-{}", (index + 3) % 8);
        let provider = if index % 2 == 0 { "codex-cli" } else { "fake" };
        let created_at = format!(
            "2026-05-18T{:02}:{:02}:{:02}Z",
            index / 3600,
            (index / 60) % 60,
            index % 60
        );

        transaction
            .execute(
                "INSERT INTO assets (
                    id, library_id, media_type, title, description, schema_prompt, category,
                    rating, status, created_at, updated_at, captured_at
                 ) VALUES (?1, ?2, 'image', ?3, NULL, NULL, ?4, ?5, 'generated', ?6, ?6, NULL)",
                params![
                    asset_id,
                    library.id.0,
                    format!("Synthetic Asset {index:05}"),
                    format!("category-{}", index % 4),
                    (index % 5) + 1,
                    created_at
                ],
            )
            .expect("insert asset");
        transaction
            .execute(
                "INSERT INTO asset_versions (
                    id, asset_id, parent_version_id, generation_event_id, file_path, sha256,
                    checksum_algorithm, checksum, width, height, mime_type, version_number, version_label, created_at
                 ) VALUES (?1, ?2, NULL, ?3, ?4, 'sha', 'SHA-256', 'sha', 1024, 1024, 'image/png', 1, 'v1', ?5)",
                params![
                    version_id,
                    asset_id,
                    event_id,
                    format!("originals/{index:05}.png"),
                    created_at
                ],
            )
            .expect("insert version");
        transaction
            .execute(
                "INSERT INTO generation_events (
                    id, asset_id, output_version_id, provider, provider_model, operation_type, prompt,
                    negative_prompt, input_asset_version_id, parameters_json, raw_request_json,
                    raw_response_json, status, started_at, completed_at, error_code, error_message
                 ) VALUES (?1, ?2, ?3, ?4, 'synthetic-model', 'text_to_image', ?5, NULL, NULL, '{}', NULL, NULL, 'completed', ?6, ?6, NULL, NULL)",
                params![
                    event_id,
                    asset_id,
                    version_id,
                    provider,
                    format!("synthetic botanical prompt {index:05}"),
                    created_at
                ],
            )
            .expect("insert event");
        for tag_id in [tag_a, tag_b] {
            transaction
                .execute(
                    "INSERT INTO asset_tags (asset_id, tag_id, source, confirmed_at) VALUES (?1, ?2, 'synthetic', ?3)",
                    params![asset_id, tag_id, now],
                )
                .expect("insert asset tag");
        }
        if index % 5 == 0 {
            transaction
                .execute(
                    "INSERT INTO metadata_suggestions (
                        id, asset_id, source, suggested_title, suggested_description,
                        suggested_schema_prompt, suggested_tags_json, suggested_category,
                        confidence_json, status, created_at, reviewed_at
                     ) VALUES (?1, ?2, 'synthetic', NULL, NULL, NULL, '[]', NULL, '{}', 'pending_review', ?3, NULL)",
                    params![format!("suggestion-{index:05}"), asset_id, created_at],
                )
                .expect("insert suggestion");
        }
        if index % 2 == 0 {
            transaction
                .execute(
                    "INSERT INTO album_items (album_id, asset_id, sort_order, added_at) VALUES ('album-1', ?1, ?2, ?3)",
                    params![asset_id, index, created_at],
                )
                .expect("insert album item");
        }
    }
    transaction.commit().expect("commit synthetic data");
    eprintln!("synthetic seed 10k assets: {:?}", seed_start.elapsed());

    let gallery_start = Instant::now();
    let gallery = service
        .query_gallery(&root, GalleryQuery::default())
        .expect("query gallery");
    let gallery_elapsed = gallery_start.elapsed();
    assert_eq!(gallery.len(), 10_000);

    let search_start = Instant::now();
    let search = service
        .search(
            &library.id,
            SearchQuery {
                text: Some("botanical".to_string()),
                tags: vec!["tag-0".to_string()],
                min_rating: None,
                provider: Some("codex-cli".to_string()),
                status: None,
                category: None,
            },
        )
        .expect("search assets");
    let search_elapsed = search_start.elapsed();
    assert!(!search.is_empty());

    let album_start = Instant::now();
    let album = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_id: Some(AlbumId("album-1".to_string())),
                sort: GallerySort::AlbumOrder,
                ..GalleryQuery::default()
            },
        )
        .expect("query album gallery");
    let album_elapsed = album_start.elapsed();
    assert_eq!(album.len(), 5_000);

    eprintln!("synthetic gallery 10k assets: {gallery_elapsed:?}");
    eprintln!("synthetic search 10k assets: {search_elapsed:?}");
    eprintln!("synthetic album order 5k assets: {album_elapsed:?}");
}

#[test]
fn asset_detail_aggregates_lineage_and_file_context() {
    let root = test_root("asset-detail");
    let registry = test_root("asset-detail-registry").join("registry.sqlite");
    let source_dir = test_root("asset-detail-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    let generated = source_dir.join("generated.png");
    fs::write(&source, b"input bytes").expect("write source");
    fs::write(&generated, b"generated bytes").expect("write generated");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Detail".to_string(),
        })
        .expect("create library");
    let (asset, parent) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import parent");
    service
        .add_tag_to_asset(&root, &asset.id, "study")
        .expect("tag");
    let event = service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(asset.id.clone()),
            output_version_id: None,
            provider: "fake".to_string(),
            provider_model: "fake-image".to_string(),
            operation_type: GenerationOperation::ImageToImage,
            prompt: "make a variant".to_string(),
            negative_prompt: Some("blur".to_string()),
            input_asset_version_id: Some(parent.id.clone()),
            prompt_version_id: None,
            parameters_json: "{\"seed\":7}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("event");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: parent.id,
            generation_event_id: Some(event.id),
            source_path: generated,
            mime_type: "image/png".to_string(),
            version_label: Some("variant".to_string()),
        })
        .expect("child");

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&child.id))
        .expect("detail");

    assert_eq!(detail.id, asset.id);
    assert_eq!(detail.prompt.as_deref(), Some("make a variant"));
    assert_eq!(detail.negative_prompt.as_deref(), Some("blur"));
    assert_eq!(detail.provider.as_deref(), Some("fake"));
    assert_eq!(detail.tags, vec!["study".to_string()]);
    assert_eq!(detail.versions.len(), 2);
    assert_eq!(detail.lineage.len(), 2);
    assert_eq!(
        detail.file.as_ref().expect("file").integrity_status,
        "verified"
    );
}

#[derive(Debug, Clone)]
struct TextOnlyProvider;

impl ImageProvider for TextOnlyProvider {
    fn name(&self) -> &'static str {
        "text-only"
    }

    fn validate_parameters(&self, _parameters: &crate::GenerationParameters) -> DomainResult<()> {
        Ok(())
    }

    fn generate_from_text(
        &self,
        _parameters: &crate::GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        Ok(GenerationResult {
            images: vec![GeneratedImage {
                bytes: b"image".to_vec(),
                mime_type: "image/png".to_string(),
                provider_metadata_json: "{}".to_string(),
            }],
            raw_request_json: "{}".to_string(),
            raw_response_json: "{}".to_string(),
        })
    }

    fn generate_from_image(
        &self,
        _parameters: &crate::GenerationParameters,
        _input: &[u8],
    ) -> DomainResult<GenerationResult> {
        unreachable!("capability check should reject image-to-image first")
    }
}

#[test]
fn generation_service_rejects_unsupported_provider_capability() {
    let root = test_root("unsupported-capability");
    let registry = test_root("unsupported-capability-registry").join("registry.sqlite");
    let source_dir = test_root("unsupported-capability-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("input.png");
    fs::write(&source, b"input bytes").expect("write source");

    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Capability".to_string(),
        })
        .expect("create library");
    let (_, version) = library
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import");

    let generation = LocalGenerationService::new(TextOnlyProvider);
    let error = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: None,
            input_bytes: Some(b"input bytes".to_vec()),
            parameters: crate::GenerationParameters {
                library_path: Some(root),
                provider: "text-only".to_string(),
                model: "text-only-image".to_string(),
                prompt: "make a variant".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::ImageToImage,
                input_version_id: Some(version.id),
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect_err("unsupported capability");

    assert!(matches!(
        error,
        DomainError::UnsupportedProviderCapability { .. }
    ));
}

#[test]
fn uploaded_reference_capability_failure_does_not_import_reference() {
    let root = test_root("unsupported-uploaded-reference");
    let registry = test_root("unsupported-uploaded-reference-registry").join("registry.sqlite");
    let source_dir = test_root("unsupported-uploaded-reference-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let reference_file = source_dir.join("reference.png");
    fs::write(&reference_file, png_bytes(16, 16)).expect("write reference");

    let library = LocalLibraryService::new(registry);
    library
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Unsupported Reference".to_string(),
        })
        .expect("create library");

    let generation = LocalGenerationService::new(TextOnlyProvider);
    let error = generation
        .generate(GenerateImageRequest {
            library_path: root.clone(),
            input_file: Some(reference_file),
            input_bytes: Some(png_bytes(16, 16)),
            parameters: crate::GenerationParameters {
                library_path: Some(root.clone()),
                provider: "text-only".to_string(),
                model: "text-only-image".to_string(),
                prompt: "make a variant".to_string(),
                negative_prompt: None,
                operation: GenerationOperation::ImageToImage,
                input_version_id: None,
                prompt_version_id: None,
                parameters_json: "{}".to_string(),
            },
        })
        .expect_err("unsupported capability");

    assert!(matches!(
        error,
        DomainError::UnsupportedProviderCapability { .. }
    ));

    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");
    let reference_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE status = 'reference'",
            [],
            |row| row.get(0),
        )
        .expect("reference count");
    assert_eq!(reference_count, 0);
}

#[test]
fn rejects_unsupported_smart_album_field() {
    let root = test_root("smart-album");
    let registry = test_root("smart-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Smart".to_string(),
        })
        .expect("create library");

    let error = service
        .create_smart_album(crate::CreateSmartAlbumRequest {
            library_path: root,
            name: "Bad".to_string(),
            smart_query_json: "{\"unknown\":true}".to_string(),
        })
        .expect_err("invalid smart query");

    assert!(matches!(error, DomainError::InvalidSmartAlbumQuery { .. }));
}

#[test]
fn reorders_albums_and_manual_album_items() {
    let root = test_root("album-reorder");
    let registry = test_root("album-reorder-registry").join("registry.sqlite");
    let source_dir = test_root("album-reorder-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_a = source_dir.join("a.png");
    let source_b = source_dir.join("b.png");
    fs::write(&source_a, png_bytes(10, 10)).expect("write a");
    fs::write(&source_b, png_bytes(10, 10)).expect("write b");
    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Albums".to_string(),
        })
        .expect("create library");
    let album_a = service
        .create_manual_album(&library.id, "A")
        .expect("album a");
    let album_b = service
        .create_manual_album(&library.id, "B")
        .expect("album b");

    service
        .reorder_albums(ReorderAlbumsRequest {
            library_path: root.clone(),
            album_ids: vec![album_b.id.clone(), album_a.id.clone()],
        })
        .expect("reorder albums");
    let albums = service.list_albums(&library.id).expect("list albums");
    assert_eq!(albums[0].id, album_b.id);

    let (asset_a, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_a,
        })
        .expect("import a");
    let (asset_b, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_b,
        })
        .expect("import b");
    let source_c = source_dir.join("c.png");
    fs::write(&source_c, png_bytes(10, 10)).expect("write c");
    let (asset_c, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_c,
        })
        .expect("import c");
    service
        .batch_add_assets(BatchAddAssetsToAlbumRequest {
            album_id: album_a.id.clone(),
            asset_ids: vec![asset_a.id.clone(), asset_b.id.clone()],
        })
        .expect("batch add");
    service
        .reorder_album_items(ReorderAlbumItemsRequest {
            album_id: album_a.id.clone(),
            asset_ids: vec![asset_b.id.clone(), asset_a.id.clone()],
        })
        .expect("reorder items");
    let items = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_id: Some(album_a.id),
                sort: GallerySort::AlbumOrder,
                ..GalleryQuery::default()
            },
        )
        .expect("album order query");
    assert_eq!(items[0].id, asset_b.id);
    assert_eq!(items[1].id, asset_a.id);
    assert!(!items.iter().any(|item| item.id == asset_c.id));
}

#[test]
fn gallery_album_filter_supports_union_unassigned_and_validation() {
    let root = test_root("gallery-album-filter");
    let registry = test_root("gallery-album-filter-registry").join("registry.sqlite");
    let source_dir = test_root("gallery-album-filter-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let service = LocalLibraryService::new(registry);
    let library = service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Gallery Album Filter".to_string(),
        })
        .expect("create library");
    let album_a = service
        .create_manual_album(&library.id, "A")
        .expect("album a");
    let album_b = service
        .create_manual_album(&library.id, "B")
        .expect("album b");

    let import_named = |name: &str| {
        let source = source_dir.join(format!("{name}.png"));
        fs::write(&source, png_bytes(10, 10)).expect("write source");
        service
            .import_asset(ImportAssetRequest {
                library_path: root.clone(),
                source_path: source,
            })
            .expect("import")
            .0
    };

    let asset_a = import_named("a");
    let asset_b = import_named("b");
    let shared = import_named("shared");
    let smart_only = import_named("smart-only");
    let unassigned = import_named("unassigned");

    service
        .batch_add_assets(BatchAddAssetsToAlbumRequest {
            album_id: album_a.id.clone(),
            asset_ids: vec![asset_a.id.clone(), shared.id.clone()],
        })
        .expect("batch add a");
    service
        .batch_add_assets(BatchAddAssetsToAlbumRequest {
            album_id: album_b.id.clone(),
            asset_ids: vec![asset_b.id.clone(), shared.id.clone()],
        })
        .expect("batch add b");
    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: smart_only.id.clone(),
            title: Some("smart-only".to_string()),
            description: None,
            schema_prompt: None,
            rating: None,
            category: None,
            status: None,
        })
        .expect("title smart-only");
    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: unassigned.id.clone(),
            title: Some("unassigned".to_string()),
            description: None,
            schema_prompt: None,
            rating: None,
            category: None,
            status: None,
        })
        .expect("title unassigned");
    service
        .create_smart_album(crate::CreateSmartAlbumRequest {
            library_path: root.clone(),
            name: "Smart Only".to_string(),
            smart_query_json: r#"{"text":"smart-only"}"#.to_string(),
        })
        .expect("smart album");

    let default_items = service
        .query_gallery(&root, GalleryQuery::default())
        .expect("default query");
    assert_eq!(default_items.len(), 5);

    let single = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::InAny(vec![album_a.id.clone()]),
                ..GalleryQuery::default()
            },
        )
        .expect("single album query");
    assert_eq!(single.len(), 2);
    assert!(single.iter().any(|item| item.id == asset_a.id));
    assert!(single.iter().any(|item| item.id == shared.id));
    assert!(single
        .iter()
        .all(|item| item.album_context.as_ref().map(|album| &album.id) == Some(&album_a.id)));

    let union = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::InAny(vec![
                    album_a.id.clone(),
                    album_b.id.clone(),
                    album_a.id.clone(),
                ]),
                ..GalleryQuery::default()
            },
        )
        .expect("multi album query");
    assert_eq!(union.len(), 3);
    assert!(union.iter().any(|item| item.id == asset_a.id));
    assert!(union.iter().any(|item| item.id == asset_b.id));
    assert!(union.iter().any(|item| item.id == shared.id));
    assert!(union.iter().all(|item| item.album_context.is_none()));

    let empty_filter = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::InAny(Vec::new()),
                ..GalleryQuery::default()
            },
        )
        .expect("empty album filter");
    assert_eq!(empty_filter.len(), default_items.len());

    let unassigned_items = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::Unassigned,
                ..GalleryQuery::default()
            },
        )
        .expect("unassigned query");
    assert_eq!(unassigned_items.len(), 1);
    assert_eq!(unassigned_items[0].id, unassigned.id);

    let unknown_error = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::InAny(vec![AlbumId("missing".to_string())]),
                ..GalleryQuery::default()
            },
        )
        .expect_err("unknown album id");
    assert!(matches!(
        unknown_error,
        DomainError::InvalidGalleryQuery { .. }
    ));

    let multi_album_order_error = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::InAny(vec![album_a.id, album_b.id]),
                sort: GallerySort::AlbumOrder,
                ..GalleryQuery::default()
            },
        )
        .expect_err("album_order rejects multi album");
    assert!(matches!(
        multi_album_order_error,
        DomainError::InvalidGalleryQuery { .. }
    ));

    let unassigned_album_order_error = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_filter: GalleryAlbumFilter::Unassigned,
                sort: GallerySort::AlbumOrder,
                ..GalleryQuery::default()
            },
        )
        .expect_err("album_order rejects unassigned");
    assert!(matches!(
        unassigned_album_order_error,
        DomainError::InvalidGalleryQuery { .. }
    ));
}

#[test]
fn album_path_helpers_do_not_depend_on_registry_lookup() {
    let root = test_root("album-path-helpers");
    let registry = test_root("album-path-registry").join("registry.sqlite");
    let detached_registry = test_root("album-path-detached-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Album Path Helpers".to_string(),
        })
        .expect("create library");

    let detached = LocalLibraryService::new(detached_registry);
    let album = detached
        .create_manual_album_in_library(&root, "Detached Album")
        .expect("create album by path");
    let albums = detached
        .list_albums_in_library(&root)
        .expect("list albums by path");

    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].id, album.id);
    assert_eq!(albums[0].name, "Detached Album");
}

#[test]
fn smart_album_supports_created_range_and_album_order_validation() {
    let root = test_root("smart-range");
    let registry = test_root("smart-range-registry").join("registry.sqlite");
    let source_dir = test_root("smart-range-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("asset.png");
    fs::write(&source, png_bytes(10, 10)).expect("write source");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Smart Range".to_string(),
        })
        .expect("create library");
    let (asset, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import");
    let smart = service
        .create_smart_album(crate::CreateSmartAlbumRequest {
            library_path: root.clone(),
            name: "Recent".to_string(),
            smart_query_json: "{\"createdAtFrom\":\"0\",\"createdAtTo\":\"999999999999999\"}"
                .to_string(),
        })
        .expect("smart album");
    let items = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_id: Some(smart.id),
                ..GalleryQuery::default()
            },
        )
        .expect("smart query");
    assert_eq!(items[0].id, asset.id);

    let error = service
        .query_gallery(
            &root,
            GalleryQuery {
                sort: GallerySort::AlbumOrder,
                ..GalleryQuery::default()
            },
        )
        .expect_err("album order needs album");
    assert!(matches!(error, DomainError::InvalidGalleryQuery { .. }));
}

#[test]
fn smart_album_uses_shared_gallery_filters() {
    let root = test_root("smart-shared-filters");
    let registry = test_root("smart-shared-filters-registry").join("registry.sqlite");
    let source_dir = test_root("smart-shared-filters-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_a = source_dir.join("a.png");
    let source_b = source_dir.join("b.png");
    fs::write(&source_a, png_bytes(10, 10)).expect("write a");
    fs::write(&source_b, png_bytes(10, 10)).expect("write b");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Smart Shared Filters".to_string(),
        })
        .expect("create library");
    let (matching, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_a,
        })
        .expect("import matching");
    let (excluded, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_b,
        })
        .expect("import excluded");

    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: matching.id.clone(),
            title: Some("Neon Botanical Study".to_string()),
            description: None,
            schema_prompt: None,
            rating: Some(5),
            category: Some("botanical".to_string()),
            status: Some("curated".to_string()),
        })
        .expect("update matching");
    service
        .update_asset_metadata(UpdateAssetMetadataRequest {
            library_path: root.clone(),
            asset_id: excluded.id,
            title: Some("Muted City Study".to_string()),
            description: None,
            schema_prompt: None,
            rating: Some(2),
            category: Some("city".to_string()),
            status: Some("imported".to_string()),
        })
        .expect("update excluded");
    service
        .add_tag_to_asset(&root, &matching.id, "neon")
        .expect("tag matching");
    service
        .record_generation_event(CreateGenerationEventRequest {
            library_path: root.clone(),
            asset_id: Some(matching.id.clone()),
            output_version_id: None,
            provider: "codex-cli".to_string(),
            provider_model: "codex-imagegen".to_string(),
            operation_type: GenerationOperation::TextToImage,
            prompt: "neon botanical study".to_string(),
            negative_prompt: None,
            input_asset_version_id: None,
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
            raw_request_json: None,
            raw_response_json: None,
            status: "completed".to_string(),
            error_code: None,
            error_message: None,
        })
        .expect("record generation");
    service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: matching.id.clone(),
            source: "fake".to_string(),
            suggested_title: Some("Neon Botanical Study".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec!["neon".to_string()],
            suggested_category: Some("botanical".to_string()),
            confidence_json: "{}".to_string(),
        })
        .expect("create suggestion");

    let smart = service
        .create_smart_album(crate::CreateSmartAlbumRequest {
            library_path: root.clone(),
            name: "Focused".to_string(),
            smart_query_json: r#"{
                "text":"botanical",
                "tags":["neon"],
                "providers":["codex-cli"],
                "minRating":4,
                "reviewStatus":"pending",
                "category":"botanical",
                "status":"curated",
                "createdAtFrom":"0",
                "createdAtTo":"999999999999999",
                "sort":"ratingDesc"
            }"#
            .to_string(),
        })
        .expect("smart album");

    let items = service
        .query_gallery(
            &root,
            GalleryQuery {
                album_id: Some(smart.id),
                ..GalleryQuery::default()
            },
        )
        .expect("query smart album");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, matching.id);
    assert_eq!(items[0].provider.as_deref(), Some("codex-cli"));
    assert_eq!(items[0].rating, Some(5));
    assert_eq!(items[0].review_pending_count, 1);
    assert_eq!(items[0].category.as_deref(), Some("botanical"));
    assert_eq!(items[0].status, "curated");
    assert_eq!(items[0].tags, vec!["neon".to_string()]);
}

#[test]
fn asset_detail_builds_version_tree_with_path_labels_and_gallery_summary() {
    let root = test_root("version-tree-labels");
    let registry = test_root("version-tree-labels-registry").join("registry.sqlite");
    let source_dir = test_root("version-tree-labels-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("root.png");
    let child_a_source = source_dir.join("child-a.png");
    let child_b_source = source_dir.join("child-b.png");
    let grandchild_source = source_dir.join("grandchild.png");
    fs::write(&source, png_bytes(10, 10)).expect("write root");
    fs::write(&child_a_source, png_bytes(11, 11)).expect("write child a");
    fs::write(&child_b_source, png_bytes(12, 12)).expect("write child b");
    fs::write(&grandchild_source, png_bytes(13, 13)).expect("write grandchild");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Version Tree".to_string(),
        })
        .expect("create library");
    let (asset, root_version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import root");
    let child_a = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: root_version.id.clone(),
            generation_event_id: None,
            source_path: child_a_source,
            mime_type: "image/png".to_string(),
            version_label: Some("child-a".to_string()),
        })
        .expect("create child a");
    let child_b = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: root_version.id.clone(),
            generation_event_id: None,
            source_path: child_b_source,
            mime_type: "image/png".to_string(),
            version_label: Some("child-b".to_string()),
        })
        .expect("create child b");
    let grandchild = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: child_a.id.clone(),
            generation_event_id: None,
            source_path: grandchild_source,
            mime_type: "image/png".to_string(),
            version_label: Some("grandchild".to_string()),
        })
        .expect("create grandchild");

    let detail = service
        .get_asset_detail(&root, &asset.id, Some(&grandchild.id))
        .expect("asset detail");

    assert_eq!(detail.focused_version_id, Some(grandchild.id.clone()));
    assert_eq!(detail.focused_version_tree_name.as_deref(), Some("v1.1.1"));
    assert_eq!(detail.version_tree.len(), 1);
    assert_eq!(detail.version_tree[0].tree_name, "v1");
    assert_eq!(detail.version_tree[0].children.len(), 2);
    assert_eq!(detail.version_tree[0].children[0].version_id, child_a.id);
    assert_eq!(detail.version_tree[0].children[0].tree_name, "v1.1");
    assert_eq!(
        detail.version_tree[0].children[0].children[0].tree_name,
        "v1.1.1"
    );
    assert_eq!(detail.version_tree[0].children[1].version_id, child_b.id);
    assert_eq!(detail.version_tree[0].children[1].tree_name, "v1.2");
    assert!(detail.version_tree_issues.is_empty());

    let gallery = service
        .query_gallery(&root, GalleryQuery::default())
        .expect("query gallery");
    assert_eq!(gallery[0].version_count, 4);
    assert_eq!(gallery[0].version_tree_branch_count, 2);
    assert!(gallery[0].current_version_tree_name.is_some());
}

#[test]
fn asset_detail_reports_degraded_version_tree_states() {
    let root = test_root("version-tree-degraded");
    let registry = test_root("version-tree-degraded-registry").join("registry.sqlite");
    let source_dir = test_root("version-tree-degraded-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_a = source_dir.join("a.png");
    let child_source = source_dir.join("child.png");
    let source_b = source_dir.join("b.png");
    fs::write(&source_a, png_bytes(10, 10)).expect("write a");
    fs::write(&child_source, png_bytes(11, 11)).expect("write child");
    fs::write(&source_b, png_bytes(12, 12)).expect("write b");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Degraded Tree".to_string(),
        })
        .expect("create library");
    let (asset_a, root_a) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_a,
        })
        .expect("import a");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset_a.id.clone(),
            parent_version_id: root_a.id.clone(),
            generation_event_id: None,
            source_path: child_source,
            mime_type: "image/png".to_string(),
            version_label: Some("child".to_string()),
        })
        .expect("create child");
    let (_, root_b) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_b,
        })
        .expect("import b");
    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");

    connection
        .execute(
            "UPDATE asset_versions SET parent_version_id = ?1 WHERE id = ?2",
            params!["missing-version", child.id.0],
        )
        .expect("set missing parent");
    let detail = service
        .get_asset_detail(&root, &asset_a.id, Some(&child.id))
        .expect("missing parent detail");
    assert!(detail
        .version_tree_issues
        .iter()
        .any(|issue| issue.kind == "missing_parent"));

    connection
        .execute(
            "UPDATE asset_versions SET parent_version_id = ?1 WHERE id = ?2",
            params![root_b.id.0, child.id.0],
        )
        .expect("set cross asset parent");
    let detail = service
        .get_asset_detail(&root, &asset_a.id, Some(&child.id))
        .expect("cross asset detail");
    assert!(detail
        .version_tree_issues
        .iter()
        .any(|issue| issue.kind == "cross_asset_parent"));

    connection
        .execute(
            "UPDATE asset_versions SET parent_version_id = ?1 WHERE id = ?2",
            params![child.id.0, root_a.id.0],
        )
        .expect("set cycle root");
    connection
        .execute(
            "UPDATE asset_versions SET parent_version_id = ?1 WHERE id = ?2",
            params![root_a.id.0, child.id.0],
        )
        .expect("set cycle child");
    let detail = service
        .get_asset_detail(&root, &asset_a.id, Some(&child.id))
        .expect("cycle detail");
    assert!(detail
        .version_tree_issues
        .iter()
        .any(|issue| issue.kind == "cycle"));
}

#[test]
fn promoted_source_schema_migration_is_idempotent() {
    let root = test_root("promoted-source-schema");
    let registry = test_root("promoted-source-schema-registry").join("registry.sqlite");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Promoted Source Schema".to_string(),
        })
        .expect("create library");
    let connection = Connection::open(LocalLibraryService::database_path(&root)).expect("open db");

    migrate_library_database(&connection).expect("migrate once");
    migrate_library_database(&connection).expect("migrate twice");

    let table_count: u32 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'asset_version_sources'",
            [],
            |row| row.get(0),
        )
        .expect("query table count");
    assert_eq!(table_count, 1);
}

#[test]
fn promote_version_creates_new_asset_root_and_promoted_source_detail() {
    let root = test_root("promote-version");
    let registry = test_root("promote-version-registry").join("registry.sqlite");
    let source_dir = test_root("promote-version-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("root.png");
    let child_source = source_dir.join("child.png");
    fs::write(&source, png_bytes(10, 10)).expect("write root");
    fs::write(&child_source, png_bytes(11, 11)).expect("write child");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Promote Version".to_string(),
        })
        .expect("create library");
    let (asset, root_version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import root");
    let child = service
        .create_child_version(CreateChildVersionRequest {
            library_path: root.clone(),
            asset_id: asset.id.clone(),
            parent_version_id: root_version.id,
            generation_event_id: None,
            source_path: child_source,
            mime_type: "image/png".to_string(),
            version_label: Some("child".to_string()),
        })
        .expect("create child");

    let promoted = service
        .promote_version_as_asset(crate::PromoteAssetVersionRequest {
            library_path: root.clone(),
            source_version_id: child.id.clone(),
        })
        .expect("promote version");
    assert_ne!(promoted.asset.id, asset.id);
    assert_eq!(promoted.version.version_number, 1);
    assert!(promoted.version.parent_version_id.is_none());

    let detail = service
        .get_asset_detail(&root, &promoted.asset.id, Some(&promoted.version.id))
        .expect("promoted detail");
    let promoted_from = detail.promoted_from.expect("promoted source");
    assert_eq!(promoted_from.source_asset_id, asset.id);
    assert_eq!(promoted_from.source_version_id, child.id);
    assert_eq!(
        promoted_from.source_version_tree_name.as_deref(),
        Some("v1.1")
    );
}

#[test]
fn promote_version_rejects_missing_or_checksum_mismatched_source_without_gallery_asset() {
    let root = test_root("promote-version-errors");
    let registry = test_root("promote-version-errors-registry").join("registry.sqlite");
    let source_dir = test_root("promote-version-errors-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source = source_dir.join("source.png");
    fs::write(&source, png_bytes(10, 10)).expect("write source");

    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Promote Version Errors".to_string(),
        })
        .expect("create library");
    let (_, version) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source,
        })
        .expect("import source");
    let initial_count = service
        .query_gallery(&root, GalleryQuery::default())
        .expect("initial gallery")
        .len();
    fs::remove_file(root.join(&version.file_path)).expect("remove managed file");

    service
        .promote_version_as_asset(crate::PromoteAssetVersionRequest {
            library_path: root.clone(),
            source_version_id: version.id.clone(),
        })
        .expect_err("missing source should fail");
    assert_eq!(
        service
            .query_gallery(&root, GalleryQuery::default())
            .expect("gallery after missing")
            .len(),
        initial_count
    );

    fs::write(root.join(&version.file_path), b"changed bytes").expect("rewrite managed file");
    service
        .promote_version_as_asset(crate::PromoteAssetVersionRequest {
            library_path: root.clone(),
            source_version_id: version.id,
        })
        .expect_err("checksum mismatch should fail");
    assert_eq!(
        service
            .query_gallery(&root, GalleryQuery::default())
            .expect("gallery after checksum mismatch")
            .len(),
        initial_count
    );
}

#[test]
fn batch_review_rolls_back_and_confidence_normalizes() {
    let root = test_root("batch-review");
    let registry = test_root("batch-review-registry").join("registry.sqlite");
    let source_dir = test_root("batch-review-source");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_a = source_dir.join("a.png");
    let source_b = source_dir.join("b.png");
    fs::write(&source_a, png_bytes(10, 10)).expect("write a");
    fs::write(&source_b, png_bytes(10, 10)).expect("write b");
    let service = LocalLibraryService::new(registry);
    service
        .create_library(CreateLibraryRequest {
            root_path: root.clone(),
            name: "Review".to_string(),
        })
        .expect("create library");
    let (asset_a, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_a,
        })
        .expect("import a");
    let (asset_b, _) = service
        .import_asset(ImportAssetRequest {
            library_path: root.clone(),
            source_path: source_b,
        })
        .expect("import b");
    let suggestion_a = service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: asset_a.id.clone(),
            source: "test".to_string(),
            suggested_title: Some("A".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec![],
            suggested_category: None,
            confidence_json: "{\"overall\":0.75,\"fields\":{\"title\":80}}".to_string(),
        })
        .expect("suggestion a");
    let suggestion_b = service
        .create_suggestion(crate::CreateMetadataSuggestionRequest {
            library_path: root.clone(),
            asset_id: asset_b.id.clone(),
            source: "test".to_string(),
            suggested_title: Some("B".to_string()),
            suggested_description: None,
            suggested_schema_prompt: None,
            suggested_tags: vec![],
            suggested_category: None,
            confidence_json: "{}".to_string(),
        })
        .expect("suggestion b");
    service
        .reject(&root, &suggestion_b.id)
        .expect("reject suggestion b");
    let error = service
        .batch_accept(BatchReviewMetadataSuggestionRequest {
            library_path: root.clone(),
            suggestions: vec![
                ReviewMetadataSuggestionRequest {
                    library_path: root.clone(),
                    suggestion_id: suggestion_a.id.clone(),
                    title: Some("Accepted A".to_string()),
                    description: None,
                    schema_prompt: None,
                    tags: vec![],
                    category: None,
                },
                ReviewMetadataSuggestionRequest {
                    library_path: root.clone(),
                    suggestion_id: suggestion_b.id,
                    title: Some("Accepted B".to_string()),
                    description: None,
                    schema_prompt: None,
                    tags: vec![],
                    category: None,
                },
            ],
        })
        .expect_err("batch should fail");
    assert!(matches!(error, DomainError::InvalidAssetReference { .. }));
    let asset_a_after = load_asset_summary(
        &Connection::open(LocalLibraryService::database_path(&root)).expect("open db"),
        &asset_a.id,
    )
    .expect("load asset");
    assert!(asset_a_after.title.is_none());

    let history = service.list_history(&root, &asset_a.id).expect("history");
    assert_eq!(history.len(), 1);
    let confidence = service.normalize_confidence(&history[0].confidence_json);
    assert_eq!(confidence.overall, Some(75));
    assert_eq!(confidence.title, Some(80));
    assert_eq!(service.normalize_confidence("not json").overall, None);
}
