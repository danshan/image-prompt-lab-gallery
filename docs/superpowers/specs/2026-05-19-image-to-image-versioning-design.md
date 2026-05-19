# Image-to-Image Generation and Numeric Versioning Design

## Context

The current project already has the core boundaries needed for image-to-image generation:

- `GenerationOperation::ImageToImage`
- `input_version_id` in generation request DTOs
- `parent_version_id` on `asset_versions`
- `generation_events.input_asset_version_id`
- provider capability checks
- Gallery and Inspector read models with version summaries

The missing product semantics are:

- Generated image variations should become new versions of the same logical asset.
- Uploaded reference images should be managed and traceable, but should not be forced into the output asset lineage.
- User-visible versions should be numeric and increment per asset, instead of deriving display names from UUIDs.

## Goals

- Support image-to-image generation from an existing generated image version.
- Support image-to-image generation from an uploaded reference image.
- Track generation lineage for both workflows.
- Add user-visible numeric version numbers, incrementing within each asset.
- Keep UUID identifiers as internal stable keys for foreign keys, APIs, tasks, and debugging.

## Non-Goals

- Do not replace internal UUID primary keys.
- Do not support multiple reference images in this change.
- Do not introduce a general graph lineage model.
- Do not add cloud sync, collaborative editing, or external asset storage.

## Recommended Approach

Use a minimal schema extension that reuses the existing asset version and generation event model.

- Add `asset_versions.version_number INTEGER NOT NULL`.
- Add a unique constraint or unique index for `(asset_id, version_number)`.
- Keep `asset_versions.id` as the internal UUID primary key.
- Treat `version_label` as a role or kind label, such as `generated`, `reference`, or `variant`.
- Expose `version_number` and a derived `version_name` such as `v1`, `v2`, `v3` in read models.

This preserves current service boundaries while making user-visible versioning deterministic and readable.

## Alternatives Considered

### Explicit generation input table

Add a `generation_event_inputs` table with input role, input order, and input version references.

This would support future multi-reference workflows more naturally, but it is heavier than the current requirement. The current single-reference requirement can be handled by `generation_events.input_asset_version_id`.

### Managed reference files outside assets

Add a separate managed input or reference file table for uploaded references.

This keeps Gallery cleaner, but it creates another file ownership model separate from asset versions. Integrity checks, backups, imports, exports, and lineage would need duplicate concepts.

## Data Model

### Asset versions

`asset_versions` gains:

```text
version_number INTEGER NOT NULL
```

Rules:

- The first version for a new asset is `1`.
- A child version within the same asset uses `MAX(version_number) + 1`.
- Version number allocation must happen in the same transaction as inserting the new version.
- `(asset_id, version_number)` must be unique.
- Migration backfills historical versions by ordering each asset's versions by `created_at ASC, id ASC`.

### Version identifiers

Internal IDs remain UUIDs:

- `asset_versions.id`
- `generation_events.input_asset_version_id`
- `generation_events.output_version_id`
- `asset_versions.parent_version_id`
- task output target IDs

User-facing version display uses:

- `version_number`
- `version_name = "v{version_number}"`

UUIDs may still appear in raw task details, debugging views, and machine-oriented CLI output.

## Generation Workflows

### Text-to-image

Text-to-image creates a new generated asset and its first version.

- output asset: new generated asset
- output version: `version_number = 1`
- `parent_version_id = NULL`
- `generation_events.input_asset_version_id = NULL`
- `generation_events.output_version_id = output version id`

### Image-to-image from an existing version

The user selects an existing managed asset version and requests a variation.

- Request uses `input_version_id`.
- Core loads the input version bytes from the managed library.
- Provider receives an image-to-image request.
- Output is inserted as a new version under the same asset as the input version.
- `version_number = current max + 1` for that asset.
- `parent_version_id = input_version_id`.
- `generation_events.input_asset_version_id = input_version_id`.
- `generation_events.output_version_id = output version id`.

This is a strict parent-child version lineage inside one logical asset.

### Image-to-image from an uploaded reference file

The user uploads a local file as a reference image.

- Request uses `input_file`.
- Core first validates provider capability and generation parameters.
- Core imports the uploaded file as a separate reference asset and version.
- Reference asset status should be `reference`.
- Reference version has `version_number = 1` and `version_label = reference`.
- Provider receives image-to-image input bytes from the managed reference version.
- Output creates a new generated asset with output version `version_number = 1`.
- Output version has `parent_version_id = NULL`.
- `generation_events.input_asset_version_id = reference version id`.
- `generation_events.output_version_id = output version id`.

The reference asset is traceable, but it is not part of the output asset parent chain.

### Failure behavior

If provider capability or request validation fails, no reference asset or output asset should be created.

If reference import succeeds and provider execution fails:

- Keep the reference asset.
- Record a failed generation event when enough request context exists.
- Do not create an output version.

If reference import itself fails:

- Do not create a generation event.
- Return the import or IO error.

## Read Models and UI

### Gallery

Gallery cards should display:

- current version name, such as `v1`
- version count
- no UUID-derived version display by default
- generated and imported content assets by default
- reference assets only when explicitly reached through a source link or a reference-inclusive filter

### Inspector

Inspector should display:

- versions ordered by numeric version number
- current version as `vN`
- lineage inside the current asset as parent chain
- reference source separately when the generation input belongs to another asset

For uploaded reference generation, the source section should read as a reference input, not as a parent version.

### Reference asset visibility

Reference assets are first-class managed assets for integrity, backup, and lineage, but they are not primary gallery content by default.

- Reference assets use `status = reference`.
- Default Gallery queries should exclude `status = reference`.
- Inspector source links may open a reference asset detail view.
- Search or filters may later expose references explicitly, but that is not required for this change.

### Lineage

Same-asset version lineage:

```text
v3 <- v2 <- v1
```

Uploaded-reference lineage:

```text
output asset v1
source reference: reference asset v1
```

## CLI Behavior

`--input-version <version-id>`:

- Uses the existing managed version as source.
- Creates the next numeric version in the same asset.
- Prints output asset ID, output version ID, output version number, and version name.

`--input-file <path>`:

- Imports the file as a reference asset/version.
- Creates a new generated output asset/version.
- Prints output asset/version summary and reference asset/version summary.

CLI may continue accepting UUIDs for stable machine input. Human-readable output should include numeric version names.

## Provider and Task Boundaries

- Operation inference remains: any `input_version_id` or `input_file` means image-to-image.
- Provider capability checks must happen before creating output versions.
- For uploaded files, capability checks should happen before importing the reference asset when possible.
- Daemon task outputs should link to:
  - output asset
  - output version
  - generation event
  - review suggestion when present
  - reference asset/version when an uploaded reference was imported

## Migration

Schema migration should:

1. Add nullable `version_number` if needed.
2. Backfill per asset using deterministic order: `created_at ASC, id ASC`.
3. Make the column required for new writes.
4. Add a unique index on `(asset_id, version_number)`.
5. Increment the library schema version.

Historical `version_label` values should be preserved.

## Testing

Core tests:

- Importing an asset creates `version_number = 1`.
- Text-to-image creates output `v1`.
- Image-to-image from an existing version creates same-asset `v2`.
- A second variation creates `v3`.
- Uploaded reference generation creates a reference asset `v1` and an output asset `v1`.
- Uploaded reference generation records `generation_events.input_asset_version_id`.
- Migration backfills deterministic per-asset version numbers.

CLI tests:

- `--input-version` prints next version number and version name.
- `--input-file` prints output and reference summaries.

UI tests:

- Gallery displays `vN` rather than UUID-derived version names.
- Inspector version list displays numeric versions.
- Generate variation uses the selected version.
- Uploaded reference lineage shows a reference source separate from the parent chain.

## Open Questions Resolved

- Uploaded reference images are imported as independent reference assets.
- Output generated from an uploaded reference creates a separate generated asset.
- Internal version IDs remain UUIDs.
- User-visible versions increment numerically per asset.
