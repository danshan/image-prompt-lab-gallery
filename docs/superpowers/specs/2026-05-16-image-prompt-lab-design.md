# Mac AI Image Prompt Lab Design

## Summary

Build a macOS-native application for managing AI image generation prompts, generated images, metadata, albums, and asset lineage. The first phase is GUI-first, with a CLI for automation and batch workflows. The system is single-user and local-first. Each resource library is stored as an independent local directory with its own SQLite database and managed image files.

The selected architecture is SwiftUI shell + Rust core + SQLite. SwiftUI provides the native Mac experience. The CLI and GUI both call the same Rust core. The Rust core exposes service-shaped interfaces from the start, so the project can later evolve into a local daemon or IPC API without rewriting repository logic.

## Goals

- Provide a native macOS GUI for browsing, generating, reviewing, tagging, rating, albuming, and version-tracking AI image assets.
- Provide a CLI for core automation: resource library creation, generation, import, export, tagging, rating, album operations, and search.
- Support phase-one image generation through Codex `gpt-image-2` and Grok image generation.
- Support text-to-image and image-to-image generation.
- Preserve generation provenance: provider, model, prompt, parameters, source image version, raw request, and raw response.
- Support AI-assisted metadata suggestions for title, tags, category, and description, with human review before canonical metadata is changed.
- Support multiple independent resource libraries, including create, open, switch, hide, and minimal import/export.
- Use local filesystem storage with per-library SQLite metadata.

## Non-Goals

- No multi-user collaboration.
- No cloud sync.
- No full resource library encryption in phase one.
- No advanced backup and migration system in phase one.
- No Photoshop-style image editor.
- No graph-style lineage visualization in phase one.
- No daemon, IPC, or local HTTP API in phase one, although the Rust core API should keep that evolution path open.

## Architecture

The first phase uses four main boundaries:

```text
SwiftUI App
  - macOS-native UI
  - library switcher
  - gallery, album, review, generation, and asset-detail workflows
  - ViewModel layer adapting Rust core responses into SwiftUI state

CLI
  - automation entrypoint
  - init, library, generate, import, export, tag, rate, album, search
  - JSON output, dry-run support, and stable exit codes

Rust Core
  - service boundary for all business operations
  - repository management
  - SQLite schema and migrations
  - file layout and integrity checks
  - asset lineage and generation events
  - provider abstraction
  - AI metadata suggestion workflow

Local Library
  - one directory per resource library
  - one SQLite database per library
  - managed originals, thumbnails, previews, sidecars, and exports
```

The Rust core is the only business source of truth. GUI and CLI code must not implement separate write paths for repository operations. They call the same core services for creating libraries, importing files, generating images, accepting suggestions, updating tags, changing ratings, and modifying albums.

The Rust core should be designed around service-shaped interfaces. Phase one links the SwiftUI app and CLI directly against the core, but the interface should avoid UI-specific assumptions. A later phase can expose the same services through IPC or a local API.

## GUI Product Design

The GUI should use a three-column workbench:

```text
Library Sidebar | Workspace | Inspector
```

The left sidebar contains resource library navigation and primary sections:

- Gallery
- Albums
- Review Inbox
- Generation Queue
- Settings

The central workspace changes based on the selected section:

- Gallery grid
- Album view
- Review inbox
- Generation composer
- Generation queue
- Search results

The right inspector shows details for the selected asset or version:

- Title, rating, status, and dates
- Prompt, negative prompt, provider, model, and generation parameters
- Tags and album membership
- AI metadata suggestions and review actions
- Asset versions and parent-child lineage
- File information and integrity status

This shape fits a prompt lab better than a mode-only image gallery. Users can look at an image, inspect the prompt, compare parameters, accept suggestions, rate the result, tag it, and add it to albums without losing context.

The inspector must be collapsible for smaller windows. The version lineage can start as a compact list with parent links rather than a graph.

## Resource Library Layout

Each resource library is an independent directory:

```text
MyImageLab.library/
  manifest.json
  library.sqlite
  originals/
    2026/05/<asset_id>.<ext>
  derivatives/
    thumbnails/<asset_id>.jpg
    previews/<asset_id>.jpg
  sidecars/
    <asset_id>.json
  exports/
  trash/
```

`manifest.json` stores resource-library-level metadata: library id, display name, schema version, created timestamp, app compatibility, and optional flags. `library.sqlite` is the authoritative index and transaction boundary. Original image files live under `originals/`. Thumbnails and previews live under `derivatives/`. Sidecars are readable snapshots for export, debugging, and future interoperability, but they are not the authoritative write path in phase one.

The app-level registry records recently opened libraries, hidden status, display names, and root paths. Hidden means hidden from default lists, not encrypted or protected.

## Data Model

Core tables:

```text
libraries
  id, name, root_path, hidden, created_at, last_opened_at

assets
  id, library_id, media_type, original_path, sha256, width, height,
  title, description, rating, status, created_at, updated_at, captured_at

asset_versions
  id, asset_id, parent_version_id, generation_event_id,
  file_path, sha256, width, height, version_label, created_at

generation_events
  id, provider, provider_model, operation_type,
  prompt, negative_prompt, input_asset_version_id,
  parameters_json, raw_request_json, raw_response_json,
  status, started_at, completed_at, error_message

metadata_suggestions
  id, asset_id, source, suggested_title, suggested_description,
  suggested_tags_json, suggested_category, confidence_json,
  status, created_at, reviewed_at

tags
  id, name, color, created_at

asset_tags
  asset_id, tag_id, source, confirmed_at

albums
  id, name, description, smart_query_json, created_at, updated_at

album_items
  album_id, asset_id, sort_order, added_at
```

`assets` represents a logical image work. `asset_versions` represents concrete files under that asset. Prompt edits, image-to-image generations, variations, and future upscale operations can create new versions.

`generation_events` records why a version exists: provider, model, operation type, prompt, parameters, source image version, raw request, raw response, status, and errors.

`metadata_suggestions` stores AI-generated title, tag, category, and description suggestions. Suggestions do not modify canonical fields until the user accepts or edits them. Accepted suggestions write into `assets.title`, `assets.description`, and `asset_tags`.

Albums support two forms:

- Manual albums use `album_items`.
- Smart albums use `smart_query_json` for saved filters such as tag, rating, provider, date, and status.

## CLI Design

Phase-one CLI command surface:

```text
imglab init <path> --name <name>
imglab library list --json
imglab library open <path>
imglab generate --library <path> --provider codex --model gpt-image-2 --prompt <text> --json
imglab generate --library <path> --input <asset-version-id> --prompt <text> --json
imglab import --library <path> <files...> --json
imglab export --library <path> --album <id> --out <path>
imglab tag add --library <path> <asset-id> <tag>
imglab rate --library <path> <asset-id> <1-5>
imglab album add --library <path> <album-id> <asset-id>
imglab search --library <path> --query <query> --json
```

CLI rules:

- All write operations support `--dry-run`.
- All queries and writes support `--json`.
- Errors use stable fields: `code`, `message`, `details`, and `recoverable`.
- The CLI does not maintain an independent state machine.
- GUI and CLI may open the same library concurrently, but phase one relies on SQLite locking and short transactions rather than real-time multi-process collaboration.

## Provider Abstraction

Image providers implement a common interface:

```text
ImageProvider
  - validate_parameters()
  - generate_from_text()
  - generate_from_image()
  - normalize_response()
```

Credentials are separated behind:

```text
ProviderCredentialStore
  - resolve_credentials()
  - validate_credentials()
```

Phase-one providers:

- `CodexGptImageProvider` for Codex `gpt-image-2`.
- `GrokImageProvider` for Grok image generation.

Provider responses are normalized into `GenerationResult`, including output files, normalized parameters, provider metadata, and raw response payloads.

## AI Metadata Suggestions

AI-assisted organization is independent from image generation providers:

```text
MetadataAssistant
  - suggest_title()
  - suggest_tags()
  - suggest_category()
  - suggest_description()
```

Phase-one flow:

1. A generated or imported asset enters the review workflow.
2. A metadata suggestion job creates a `metadata_suggestions` row with `pending_review` status.
3. The user reviews suggestions in the GUI Review Inbox.
4. The user accepts, edits, or rejects each suggestion.
5. Accepted or edited suggestions update canonical asset fields and confirmed tags.
6. Re-running suggestions creates new suggestion records rather than overwriting review history.

## Error Model

Rust core returns domain errors:

```text
LibraryNotFound
SchemaMismatch
ProviderUnavailable
CredentialMissing
GenerationFailed
InvalidAssetReference
FileIntegrityMismatch
ConcurrentWriteConflict
```

The CLI maps domain errors to stable exit codes and JSON error payloads. The GUI maps domain errors to recoverable actions such as retry, choose another library, repair index, or open settings. Provider raw errors are persisted in `generation_events.error_message` or `raw_response_json`, while the UI shows normalized messages by default.

## Testing Strategy

Rust core unit tests:

- Library create, open, and schema migration.
- Asset import, hashing, and thumbnail task enqueue.
- Asset lineage for text-to-image, image-to-image, and child versions.
- Metadata suggestion pending, accepted, edited, and rejected states.
- Album add, remove, and smart query behavior.

Rust integration tests:

- Create a full library in a temporary directory.
- Simulate provider success, failure, timeout, and invalid parameters.
- Verify SQLite and filesystem consistency.
- Verify CLI JSON output and error codes.

SwiftUI and ViewModel tests:

- ViewModel state transitions around core service calls.
- Review Inbox accept, edit, and reject flows.
- Library switcher open, hide, and recent-library behavior.
- Inspector rendering of prompt, parameters, tags, rating, and lineage.

Manual acceptance:

- Create a library.
- Generate an image from GUI.
- Generate images from CLI.
- Import local images.
- Review AI title and tag suggestions.
- Rate and tag assets.
- Create an album and add assets.
- Generate from an existing asset version.
- Inspect the asset version lineage.

## Risks and Mitigations

- Swift/Rust boundary complexity: keep FFI or bridge payloads coarse-grained and service-oriented. Avoid leaking SQLite rows directly into SwiftUI.
- Repository corruption risk: all writes go through Rust core transactions, and file writes should use temp files plus atomic rename where possible.
- Provider API drift: keep raw request and response payloads for debugging, and isolate provider-specific normalization.
- Metadata pollution: store AI suggestions separately and require human review before canonical writes.
- Phase-one scope creep: defer encryption, daemon mode, graph lineage UI, cloud sync, and advanced migration tooling.

## Phase-One Acceptance Criteria

- A user can create and open multiple local resource libraries.
- A user can generate images through Codex `gpt-image-2` and Grok from the GUI.
- A user can generate images through the CLI with JSON output.
- A user can import local images into a library.
- Generated and imported images appear in the GUI Gallery.
- New assets appear in Review Inbox with AI metadata suggestions.
- A user can accept, edit, or reject metadata suggestions.
- A user can rate, tag, and add assets to albums.
- A user can inspect prompt, provider parameters, source image, and version lineage for an asset.
- CLI and GUI operations write through the same Rust core.
- Core tests cover library creation, asset import, generation event persistence, suggestion review, albums, and lineage.
