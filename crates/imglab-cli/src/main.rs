use imglab_core::application::use_cases::library::LibraryUseCase;
use imglab_core::{
    prepare_generation_request, AssetId, CreateLibraryRequest, CreateMetadataSuggestionRequest,
    DomainError, ExportLibraryRequest, GenerateImageRequest, GenerationOperation,
    GenerationRequestInput, ImportAssetRequest, LocalLibraryService, MetadataSuggestionId,
    RepairLibraryRequest, ReviewMetadataSuggestionRequest, SearchQuery,
};
use imglab_provider_codex::CodexCliImageProvider;
use serde_json::json;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        print_error(&error);
        std::process::exit(exit_code(&error));
    }
}

fn run(args: Vec<String>) -> Result<(), DomainError> {
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };

    let app = imglab_core::infrastructure::composition::sqlite_application(
        default_registry_path(),
        imglab_core::FakeImageProvider::success("fake"),
    );
    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "init" => init(app.library_lifecycle(), &args[1..]),
        "library" => library(app.library_lifecycle(), &args[1..]),
        "import" => import(app.assets(), &args[1..]),
        "export" => export(app.library_lifecycle(), &args[1..]),
        "search" => search(app.library_lifecycle(), app.search(), &args[1..]),
        "generate" => generate(&args[1..]),
        "tag" => tag(app.library(), &args[1..]),
        "rate" => rate(app.albums(), &args[1..]),
        "album" => album(app.library_lifecycle(), app.albums(), &args[1..]),
        "suggestion" => suggestion(app.library_lifecycle(), app.metadata_review(), &args[1..]),
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported command: {command}"),
        }),
    }
}

fn init(service: &LibraryUseCase<LocalLibraryService>, args: &[String]) -> Result<(), DomainError> {
    let path = positional(args, 0, "library path")?;
    let name = option_value(args, "--name").unwrap_or_else(|| "Image Prompt Lab".to_string());
    let dry_run = has_flag(args, "--dry-run");
    if dry_run {
        print_json(json!({"dry_run": true, "path": path, "name": name}));
        return Ok(());
    }

    let summary = service.create_library(CreateLibraryRequest {
        root_path: PathBuf::from(path),
        name,
    })?;
    print_json(json!({
        "id": summary.id.0,
        "name": summary.name,
        "root_path": summary.root_path,
        "hidden": summary.hidden,
        "schema_version": summary.schema_version
    }));
    Ok(())
}

fn library(
    service: &LibraryUseCase<LocalLibraryService>,
    args: &[String],
) -> Result<(), DomainError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(DomainError::InvalidGenerationParameters {
            message: "library subcommand is required".to_string(),
        });
    };

    match subcommand {
        "list" => {
            let include_hidden = has_flag(args, "--include-hidden");
            let libraries = service.list_libraries(include_hidden)?;
            print_json(json!({
                "libraries": libraries.into_iter().map(|library| {
                    json!({
                        "id": library.id.0,
                        "name": library.name,
                        "root_path": library.root_path,
                        "hidden": library.hidden,
                        "schema_version": library.schema_version
                    })
                }).collect::<Vec<_>>()
            }));
            Ok(())
        }
        "open" => {
            let path = positional(&args[1..], 0, "library path")?;
            let summary = service.open_library(&PathBuf::from(path))?;
            print_json(
                json!({"id": summary.id.0, "name": summary.name, "root_path": summary.root_path}),
            );
            Ok(())
        }
        "hide" => {
            let id = positional(&args[1..], 0, "library id")?;
            if has_flag(args, "--dry-run") {
                print_json(json!({"dry_run": true, "id": id, "hidden": true}));
                return Ok(());
            }

            service.hide_library(&imglab_core::LibraryId(id))?;
            print_json(json!({"hidden": true}));
            Ok(())
        }
        "repair" => {
            let library_path = required_option(args, "--library")?;
            let dry_run = !has_flag(args, "--apply");
            let summary = service.repair_library(RepairLibraryRequest {
                library_path: PathBuf::from(library_path),
                dry_run,
            })?;
            print_json(json!({
                "dry_run": summary.dry_run,
                "scanned_versions": summary.scanned_versions,
                "files_moved": summary.files_moved,
                "paths_updated": summary.paths_updated,
                "checksums_updated": summary.checksums_updated,
                "dimensions_updated": summary.dimensions_updated,
                "issues": summary.issues.into_iter().map(|issue| {
                    json!({
                        "version_id": issue.version_id.0,
                        "path": issue.path,
                        "message": issue.message
                    })
                }).collect::<Vec<_>>()
            }));
            Ok(())
        }
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported library subcommand: {subcommand}"),
        }),
    }
}

fn import(
    service: &imglab_core::infrastructure::composition::SqliteAssetUseCase,
    args: &[String],
) -> Result<(), DomainError> {
    let library_path =
        option_value(args, "--library").ok_or_else(|| DomainError::LibraryNotFound {
            path: "--library".to_string(),
        })?;
    let file = positional(args, 0, "file path")?;

    if has_flag(args, "--dry-run") {
        print_json(json!({"dry_run": true, "library": library_path, "files": [file]}));
        return Ok(());
    }

    let (asset, version) = service.import_asset(ImportAssetRequest {
        library_path: PathBuf::from(library_path),
        source_path: PathBuf::from(file),
    })?;
    print_json(json!({
        "asset_id": asset.id.0,
        "version_id": version.id.0,
        "version_number": version.version_number,
        "version_name": version.version_name,
        "file_path": version.file_path,
        "checksum_algorithm": version.checksum_algorithm,
        "checksum": version.checksum
    }));
    Ok(())
}

fn export(
    service: &LibraryUseCase<LocalLibraryService>,
    args: &[String],
) -> Result<(), DomainError> {
    let library_path = required_option(args, "--library")?;
    let output_path = required_option(args, "--out")?;
    let album_id = option_value(args, "--album").map(imglab_core::AlbumId);
    if has_flag(args, "--dry-run") {
        print_json(json!({
            "dry_run": true,
            "library": library_path,
            "output": output_path,
            "album_id": album_id.map(|id| id.0)
        }));
        return Ok(());
    }

    let summary = service.export_library(ExportLibraryRequest {
        library_path: PathBuf::from(library_path),
        output_path: PathBuf::from(output_path),
        album_id,
    })?;
    print_json(
        json!({"exported_files": summary.exported_files, "exported_sidecars": summary.exported_sidecars}),
    );
    Ok(())
}

fn search(
    library_service: &LibraryUseCase<LocalLibraryService>,
    search: &imglab_core::application::use_cases::albums::SearchUseCase<LocalLibraryService>,
    args: &[String],
) -> Result<(), DomainError> {
    let library_path = required_option(args, "--library")?;
    let query_text = option_value(args, "--query");
    let library = library_service.open_library(&PathBuf::from(library_path))?;
    let results = search.execute(
        &library.id,
        SearchQuery {
            text: query_text,
            tags: vec![],
            min_rating: None,
            provider: None,
            status: None,
            category: None,
        },
    )?;
    print_json(json!({
        "assets": results.into_iter().map(|asset| {
            json!({"id": asset.id.0, "title": asset.title, "category": asset.category, "rating": asset.rating, "status": asset.status})
        }).collect::<Vec<_>>()
    }));
    Ok(())
}

fn generate(args: &[String]) -> Result<(), DomainError> {
    let library_path = required_option(args, "--library")?;
    let prompt = required_option(args, "--prompt")?;
    let provider_name = option_value(args, "--provider").unwrap_or_else(|| "codex-cli".to_string());
    let prepared = prepare_generation_request(GenerationRequestInput {
        library_path: PathBuf::from(&library_path),
        provider: provider_name,
        prompt,
        negative_prompt: option_value(args, "--negative-prompt"),
        model: None,
        operation: None,
        input_file: option_value(args, "--input-file").map(PathBuf::from),
        input_version_id: option_value(args, "--input-version").map(imglab_core::AssetVersionId),
        parameters_json: option_value(args, "--parameters"),
    })?;

    if has_flag(args, "--dry-run") {
        print_json(json!({
            "dry_run": true,
            "library": library_path,
            "provider": prepared.provider,
            "operation": operation_name(prepared.request.parameters.operation),
            "prompt": prepared.request.parameters.prompt
        }));
        return Ok(());
    }

    match prepared.provider.as_str() {
        "codex" | "codex-cli" => generate_with_provider(
            CodexCliImageProvider::new("codex", &library_path),
            prepared.request,
        ),
        "fake" => generate_with_provider(
            imglab_core::FakeImageProvider::success("fake"),
            prepared.request,
        ),
        _ => unreachable!("provider is normalized before dispatch"),
    }
}

fn generate_with_provider<P>(provider: P, request: GenerateImageRequest) -> Result<(), DomainError>
where
    P: imglab_core::application::ports::ImageGenerationProvider,
{
    let library_path = request.library_path.clone();
    let app = imglab_core::infrastructure::composition::sqlite_application(
        default_registry_path(),
        provider,
    );
    let versions = app.generation().execute(request)?;
    print_json(json!({
        "versions": versions.into_iter().map(|version| {
            let source_reference = app.gallery()
                .get_asset_detail(&library_path, &version.asset_id, Some(&version.id))
                .ok()
                .and_then(|detail| detail.source_reference)
                .map(|reference| {
                    json!({
                        "asset_id": reference.asset_id.0,
                        "asset_title": reference.asset_title,
                        "asset_status": reference.asset_status,
                        "version_id": reference.version_id.0,
                        "version_number": reference.version_number,
                        "version_name": reference.version_name,
                        "file_path": reference.file_path
                    })
                });
            json!({
                "id": version.id.0,
                "asset_id": version.asset_id.0,
                "parent_version_id": version.parent_version_id.map(|id| id.0),
                "generation_event_id": version.generation_event_id.map(|id| id.0),
                "version_number": version.version_number,
                "version_name": version.version_name,
                "file_path": version.file_path,
                "checksum_algorithm": version.checksum_algorithm,
                "checksum": version.checksum,
                "mime_type": version.mime_type,
                "source_reference": source_reference
            })
        }).collect::<Vec<_>>()
    }));
    Ok(())
}

fn rate(
    albums: &imglab_core::application::use_cases::albums::AlbumUseCase<LocalLibraryService>,
    args: &[String],
) -> Result<(), DomainError> {
    let library_path = required_option(args, "--library")?;
    let asset_id = positional(args, 0, "asset id")?;
    let rating = positional(args, 1, "rating")?
        .parse::<u8>()
        .map_err(|error| DomainError::InvalidGenerationParameters {
            message: format!("invalid rating: {error}"),
        })?;
    if has_flag(args, "--dry-run") {
        print_json(json!({"dry_run": true, "asset_id": asset_id, "rating": rating}));
        return Ok(());
    }

    let asset = albums.update_asset_metadata(imglab_core::UpdateAssetMetadataRequest {
        library_path: PathBuf::from(library_path),
        asset_id: imglab_core::AssetId(asset_id),
        title: None,
        description: None,
        schema_prompt: None,
        rating: Some(rating),
        category: None,
        status: None,
    })?;
    print_json(json!({"id": asset.id.0, "rating": asset.rating}));
    Ok(())
}

fn tag(service: &LocalLibraryService, args: &[String]) -> Result<(), DomainError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(DomainError::InvalidGenerationParameters {
            message: "tag subcommand is required".to_string(),
        });
    };

    match subcommand {
        "add" => {
            let library_path = required_option(args, "--library")?;
            let asset_id = positional(&args[1..], 0, "asset id")?;
            let tag = positional(&args[1..], 1, "tag")?;
            if has_flag(args, "--dry-run") {
                print_json(json!({"dry_run": true, "asset_id": asset_id, "tag": tag}));
                return Ok(());
            }

            service.add_tag_to_asset(
                &PathBuf::from(library_path),
                &AssetId(asset_id.clone()),
                &tag,
            )?;
            print_json(json!({"asset_id": asset_id, "tag": tag}));
            Ok(())
        }
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported tag subcommand: {subcommand}"),
        }),
    }
}

fn album(
    library_service: &LibraryUseCase<LocalLibraryService>,
    albums: &imglab_core::application::use_cases::albums::AlbumUseCase<LocalLibraryService>,
    args: &[String],
) -> Result<(), DomainError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(DomainError::InvalidGenerationParameters {
            message: "album subcommand is required".to_string(),
        });
    };

    match subcommand {
        "create" => {
            let library_path = required_option(args, "--library")?;
            let name = positional(&args[1..], 0, "album name")?;
            let library = library_service.open_library(&PathBuf::from(library_path))?;
            if has_flag(args, "--dry-run") {
                print_json(json!({"dry_run": true, "name": name}));
                return Ok(());
            }

            let album = albums.create_manual_album(&library.id, &name)?;
            print_json(json!({"id": album.id.0, "name": album.name, "kind": "manual"}));
            Ok(())
        }
        "add" => {
            let album_id = positional(&args[1..], 0, "album id")?;
            let asset_id = positional(&args[1..], 1, "asset id")?;
            if has_flag(args, "--dry-run") {
                print_json(json!({"dry_run": true, "album_id": album_id, "asset_id": asset_id}));
                return Ok(());
            }

            albums.add_asset(
                &imglab_core::AlbumId(album_id.clone()),
                &AssetId(asset_id.clone()),
            )?;
            print_json(json!({"album_id": album_id, "asset_id": asset_id}));
            Ok(())
        }
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported album subcommand: {subcommand}"),
        }),
    }
}

fn suggestion(
    library_service: &LibraryUseCase<LocalLibraryService>,
    metadata_review: &imglab_core::application::use_cases::metadata_review::ReviewMetadataSuggestionUseCase<
        LocalLibraryService,
    >,
    args: &[String],
) -> Result<(), DomainError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(DomainError::InvalidGenerationParameters {
            message: "suggestion subcommand is required".to_string(),
        });
    };

    match subcommand {
        "list" => {
            let library_path = required_option(args, "--library")?;
            let library = library_service.open_library(&PathBuf::from(&library_path))?;
            let suggestions =
                metadata_review.list_pending(&PathBuf::from(library_path), &library.id)?;
            print_json(json!({
                "suggestions": suggestions.into_iter().map(|suggestion| {
                    json!({
                        "id": suggestion.id.0,
                        "asset_id": suggestion.asset_id.0,
                        "title": suggestion.suggested_title,
                        "description": suggestion.suggested_description,
                        "schema_prompt": suggestion.suggested_schema_prompt,
                        "tags": suggestion.suggested_tags,
                        "category": suggestion.suggested_category,
                        "status": suggestion.status
                    })
                }).collect::<Vec<_>>()
            }));
            Ok(())
        }
        "create" => {
            let library_path = required_option(args, "--library")?;
            let asset_id = positional(&args[1..], 0, "asset id")?;
            let tags = option_values(args, "--tag");
            if has_flag(args, "--dry-run") {
                print_json(json!({
                    "dry_run": true,
                    "asset_id": asset_id,
                    "title": option_value(args, "--title"),
                    "description": option_value(args, "--description"),
                    "schema_prompt": option_value(args, "--schema-prompt"),
                    "tags": tags,
                    "category": option_value(args, "--category")
                }));
                return Ok(());
            }

            let suggestion =
                metadata_review.create_suggestion(CreateMetadataSuggestionRequest {
                    library_path: PathBuf::from(library_path),
                    asset_id: AssetId(asset_id),
                    source: "cli".to_string(),
                    suggested_title: option_value(args, "--title"),
                    suggested_description: option_value(args, "--description"),
                    suggested_schema_prompt: option_value(args, "--schema-prompt"),
                    suggested_tags: tags,
                    suggested_category: option_value(args, "--category"),
                    confidence_json: option_value(args, "--confidence")
                        .unwrap_or_else(|| "{}".to_string()),
                })?;
            print_json(json!({"id": suggestion.id.0, "status": suggestion.status}));
            Ok(())
        }
        "accept" => {
            let library_path = required_option(args, "--library")?;
            let suggestion_id = positional(&args[1..], 0, "suggestion id")?;
            let tags = option_values(args, "--tag");
            if has_flag(args, "--dry-run") {
                print_json(json!({
                    "dry_run": true,
                    "suggestion_id": suggestion_id,
                    "title": option_value(args, "--title"),
                    "description": option_value(args, "--description"),
                    "schema_prompt": option_value(args, "--schema-prompt"),
                    "tags": tags,
                    "category": option_value(args, "--category")
                }));
                return Ok(());
            }

            let asset = metadata_review.accept(ReviewMetadataSuggestionRequest {
                library_path: PathBuf::from(library_path),
                suggestion_id: MetadataSuggestionId(suggestion_id),
                title: option_value(args, "--title"),
                description: option_value(args, "--description"),
                schema_prompt: option_value(args, "--schema-prompt"),
                tags,
                category: option_value(args, "--category"),
            })?;
            print_json(json!({
                "id": asset.id.0,
                "title": asset.title,
                "category": asset.category,
                "rating": asset.rating,
                "status": asset.status
            }));
            Ok(())
        }
        "reject" => {
            let library_path = required_option(args, "--library")?;
            let suggestion_id = positional(&args[1..], 0, "suggestion id")?;
            if has_flag(args, "--dry-run") {
                print_json(json!({"dry_run": true, "id": suggestion_id, "status": "rejected"}));
                return Ok(());
            }

            metadata_review.reject(
                &PathBuf::from(library_path),
                &MetadataSuggestionId(suggestion_id.clone()),
            )?;
            print_json(json!({"id": suggestion_id, "status": "rejected"}));
            Ok(())
        }
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported suggestion subcommand: {subcommand}"),
        }),
    }
}

fn default_registry_path() -> PathBuf {
    std::env::var_os("IMGLAB_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-registry.sqlite"))
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn option_values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .collect()
}

fn required_option(args: &[String], name: &str) -> Result<String, DomainError> {
    option_value(args, name).ok_or_else(|| DomainError::InvalidGenerationParameters {
        message: format!("{name} is required"),
    })
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn positional(args: &[String], index: usize, label: &str) -> Result<String, DomainError> {
    positional_values(args)
        .into_iter()
        .nth(index)
        .cloned()
        .ok_or_else(|| DomainError::InvalidGenerationParameters {
            message: format!("{label} is required"),
        })
}

fn positional_values(args: &[String]) -> Vec<&String> {
    let mut values = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg.starts_with("--") {
            if option_takes_value(arg) {
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        values.push(arg);
        index += 1;
    }
    values
}

fn option_takes_value(name: &str) -> bool {
    matches!(
        name,
        "--name"
            | "--library"
            | "--out"
            | "--album"
            | "--query"
            | "--provider"
            | "--prompt"
            | "--negative-prompt"
            | "--parameters"
            | "--input-file"
            | "--input-version"
            | "--title"
            | "--description"
            | "--tag"
            | "--category"
            | "--confidence"
    )
}

fn operation_name(operation: GenerationOperation) -> &'static str {
    match operation {
        GenerationOperation::TextToImage => "text-to-image",
        GenerationOperation::ImageToImage => "image-to-image",
    }
}

fn print_json(value: serde_json::Value) {
    println!("{value}");
}

fn print_error(error: &DomainError) {
    eprintln!(
        "{}",
        json!({
            "code": error.code(),
            "message": error.to_string(),
            "details": {},
            "recoverable": error.recoverable()
        })
    );
}

fn exit_code(error: &DomainError) -> i32 {
    match error {
        DomainError::LibraryNotFound { .. } => 3,
        DomainError::SchemaMismatch { .. } => 4,
        DomainError::InvalidGenerationParameters { .. } => 2,
        _ => 1,
    }
}

fn print_help() {
    println!("imglab: local AI image prompt lab CLI");
    println!(
        "commands: init, library, import, export, search, generate, tag, rate, album, suggestion"
    );
}
