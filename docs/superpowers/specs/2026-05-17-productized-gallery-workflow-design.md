# Productized Gallery Workflow Design

## Summary

This iteration upgrades Image Prompt Lab's desktop Gallery into a productized, end-to-end workflow based on the provided reference design. The goal is not a visual-only reskin and not a full metadata workstation rewrite. The scope is the primary Gallery path:

```text
Gallery query -> select asset -> Inspector detail -> metadata/review/lineage/file context -> generate variation entry
```

The implementation should closely follow the reference draft's three-column workbench, dense information hierarchy, restrained macOS-like visual style, and right-side Inspector detail model. At the same time, search, filters, sorting, selected asset detail, and provider capability errors should be backed by Rust core semantics rather than long-lived frontend-only mock logic.

## Goals

- Match the reference draft's overall workbench style with pragmatic product refinements.
- Make Gallery search, filtering, and sorting core-defined semantics.
- Provide card-level and Inspector-level data through core read models.
- Keep React responsible for UI and query state, not business query rules.
- Support a visible image-to-image variation entry point without requiring every provider to implement it immediately.
- Improve responsive behavior so Inspector collapses cleanly on narrower windows.
- Keep Albums, Queue, and Settings usable and visually consistent without turning them into full subprojects in this iteration.

## Non-Goals

- Do not implement graph-style lineage visualization.
- Do not implement a complete smart album builder.
- Do not implement full queue history management.
- Do not implement a full settings center.
- Do not add cloud sync, encryption, daemon, IPC, or local HTTP API.
- Do not require stable native image-to-image support from the Codex CLI adapter.

## Selected Approach

Use the productized reference approach:

- Preserve the reference design's three-column structure and information density.
- Add real core query semantics and asset detail aggregation.
- Add empty, loading, error, and responsive states that make the design usable as an application.
- Keep the first implementation focused on the Gallery main workflow.

This approach balances visual fidelity with long-term architecture. A high-fidelity UI-only shell would leave core and UI semantics split. A power-user Inspector with raw payload editing and deep metadata operations would exceed this iteration's scope.

## Architecture Boundaries

Rust core owns Gallery query semantics and asset detail aggregation. Tauri commands should translate DTOs and domain errors, but should not reconstruct business behavior. React should maintain UI state, query form state, selected asset state, loading state, and recoverable error display.

```text
React Workbench
  - query controls
  - selection and detail state
  - workbench layout and responsive behavior
  - section-level loading and error display

Tauri Commands
  - DTO conversion
  - command input validation that belongs at the boundary
  - domain error mapping

Rust Core
  - GalleryQuery semantics
  - GalleryAssetView aggregation
  - AssetDetailView aggregation
  - provider capability checks
  - search, filters, sorting, and stable joins

SQLite Resource Library
  - canonical assets, versions, generation events, tags, albums, suggestions
  - file metadata and integrity status where available
```

All writes continue through core service boundaries. The frontend must not write SQLite or managed library files directly.

## Core Read Models

Add read models that match the desktop workflow without leaking SQLite rows or UI component state.

### GalleryQuery

`GalleryQuery` should cover business-level query fields:

- `text`
- `providers`
- `min_rating`
- `review_status`
- `tags`
- `album_id`
- `sort`

`view_mode` remains frontend-only unless it changes query semantics. Sorting should be core-defined with stable values:

- `newest`
- `oldest`
- `rating_desc`
- `title_asc`
- `provider_asc`

Invalid query combinations should return a typed domain error instead of silently falling back.

### GalleryAssetView

`GalleryAssetView` should include the data needed for Gallery cards:

- asset id
- title
- category
- rating
- status
- provider
- model label
- tags
- review pending count or review status
- current or preferred version id
- thumbnail or original image path
- version label
- version count
- added or generated timestamp

If a field is not yet present in the library, core should return `None` or an empty collection. The UI should show a deliberate empty state instead of fabricating real data.

### AssetDetailView

`AssetDetailView` should include the data needed by Inspector:

- asset identity, title, rating, status, and dates
- prompt and negative prompt where available
- provider and model
- generation parameters such as aspect ratio, resolution, seed, cfg scale, sampler, and duration when available
- tags and album memberships
- pending review state
- versions and lineage entries
- file metadata such as filename, relative location, mime type, size, dimensions, integrity status, and checksum

The first implementation can keep file size, dimensions, and duration nullable if the current schema or import pipeline does not reliably populate them.

## Tauri Commands

Add or consolidate commands around the read workflow:

```text
query_gallery(input) -> Vec<GalleryAssetView>
get_asset_detail(input) -> AssetDetailView
start_generation(input) -> GenerationJobView
```

`start_generation` should accept `input_version_id` and route it through core capability checks. If the selected provider does not support image-to-image, the command should return a recoverable `UnsupportedProviderCapability` error.

## Error Handling

Core should return typed domain errors for expected workflow failures:

- `InvalidGalleryQuery`
- `LibraryNotOpen`
- `AssetNotFound`
- `UnsupportedProviderCapability`
- `IntegrityCheckFailed`

Tauri should map domain errors into the existing command error shape:

```text
{ code, message, recoverable }
```

The UI should display recoverable errors inline. Unsupported image-to-image should appear near the variation action or generation status area without breaking the whole workbench.

## UI Information Architecture

The workbench remains:

```text
Library Sidebar | Workspace | Inspector
```

### Library Sidebar

The sidebar should include:

- macOS-style app chrome treatment where appropriate
- library switcher card
- Gallery, Albums, Review Inbox, Generation Queue, and Settings navigation
- badge counts for review and queue
- bottom Library Status panel

Library Status should show storage usage, integrity status, last checked time, and a `Run Integrity Check` action. If real storage usage is not available yet, the section should present an explicit unavailable state.

### Workspace

The Gallery workspace should include:

- command/search bar
- Generate split button
- grid/list view toggle
- provider, rating, review status, and tag filters
- clear filters action
- sort control
- item count

Gallery cards should show:

- thumbnail
- title
- provider/model chip
- rating
- review pending badge
- tags
- version label
- version count

The selected card should use a teal outline and must not change its dimensions when selected.

### Inspector

Inspector should use stable sections:

- header preview
- prompt
- provider and model
- tags
- albums
- versions and lineage
- file

The header shows title, rating, review status, and dates. Prompt supports full-text expansion and copy. Provider and model shows normalized parameters where available. Tags and Albums provide add actions with minimal inline input or popover. Versions and Lineage uses a list model and includes `Generate variation`. File shows filename, relative location, mime type, size, dimensions, integrity, checksum, and re-verify/open actions when supported.

## Responsive Behavior

Wide desktop uses fixed three columns. Medium width collapses Inspector into a right drawer or overlay. Narrow width shifts sidebar into a compact top or drawer navigation, keeps the Gallery query controls usable, and opens selected asset detail as a panel or page.

Text must not overlap or overflow controls. Toolbar controls, cards, icon buttons, and section headers should have stable dimensions so loading, hover, badges, and dynamic content do not shift the layout unexpectedly.

## Image-to-Image Variation Entry

Inspector should expose a variation workflow from the selected version. The first iteration should wire the UI and command boundary:

```text
selected version -> Generate variation -> start_generation(input_version_id)
```

Provider support is capability-based. If the provider cannot perform image-to-image, core returns a recoverable capability error and the UI explains that the selected provider only supports text-to-image. This preserves the lineage workflow without forcing unstable provider behavior.

## Testing Strategy

Core tests should cover:

- Gallery query text filtering.
- Tag, rating, provider, review status, and album filters.
- Sort order semantics.
- Asset detail aggregation for prompt, tags, albums, versions, lineage, and file metadata.
- Unsupported provider capability for image-to-image.

Tauri tests should cover:

- DTO mapping for `query_gallery`.
- DTO mapping for `get_asset_detail`.
- Error mapping for recoverable provider capability failures.

Frontend tests should cover:

- query state updates.
- gallery selection behavior.
- rating and review action state updates.
- capability error display.
- empty, loading, and error states.

Visual verification should cover:

- wide desktop three-column layout.
- medium width Inspector collapse.
- narrow width navigation and detail panel behavior.
- selected card, filter toolbar, and Inspector section alignment.

## Implementation Order

1. Create OpenSpec deltas for `desktop-workbench`, `albums-search`, `image-generation`, `asset-versioning`, and `resource-library`.
2. Add core read models and query/detail service tests.
3. Add Tauri commands and error mapping.
4. Refactor frontend state around query, selection, detail, loading, and errors.
5. Rebuild the Gallery workbench UI to match the productized reference style.
6. Verify with Rust tests, desktop tests, frontend tests, and browser visual inspection.
7. Commit after each stable boundary: spec, core query, Tauri commands, frontend UI, and verification fixes.

## Risks and Mitigations

- Scope creep into a full metadata workstation. Mitigation: keep this iteration centered on Gallery and Inspector.
- UI depending on fabricated data. Mitigation: core returns nullable fields and UI shows explicit unavailable states.
- Provider-specific behavior leaking into UI. Mitigation: expose provider capability through domain errors and normalized metadata.
- Query performance degrading with larger libraries. Mitigation: define core query semantics now and add indexes where tests or query plans show need.
- `main.tsx` growing too large. Mitigation: split focused components and state helpers during the frontend implementation.

## Acceptance Criteria

- Gallery search, filtering, and sorting are handled by core-defined query semantics.
- Selecting an asset loads an Inspector detail view from core aggregation.
- The desktop UI visually follows the provided reference draft with productized states.
- The Inspector includes prompt, provider/model, tags, albums, lineage, and file sections.
- The variation entry point accepts a selected version and shows a recoverable error for unsupported providers.
- Responsive layouts avoid overlapping text and preserve the primary Gallery workflow.
- Tests cover core query semantics, detail aggregation, command mapping, and key frontend state behavior.
