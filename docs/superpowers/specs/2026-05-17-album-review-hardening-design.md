# Album And Review Hardening Design

## Summary

This iteration hardens Image Prompt Lab's Albums and Review workflows as two separate product workspaces. Albums handles long-lived user organization. Review handles metadata trust before AI or system suggestions become canonical asset metadata.

The selected direction is "parallel workspaces": make both views real and usable without merging them into a larger curation workstation. Generated or imported assets can appear in Gallery immediately, while pending metadata suggestions enter Review Inbox and are surfaced through badges and asset detail state.

## Goals

- Make Albums a real organization workflow instead of a provider-grouped placeholder view.
- Make Review Inbox support inspecting, editing, accepting, and rejecting pending metadata suggestions.
- Keep Gallery and Inspector consistent after album membership or review state changes.
- Preserve the review-first metadata boundary: pending suggestions must not become canonical metadata until accepted.
- Keep Albums and Review independent enough that each workflow remains testable and understandable.

## Non-Goals

- Do not implement a full smart album builder.
- Do not implement bulk review operations.
- Do not merge Albums and Review into a unified curation inbox.
- Do not implement cross-view drag-and-drop organization.
- Do not implement album delete, rename, reorder, or remove asset from album in this iteration.
- Do not implement native OpenAI or Grok image providers.

## Module Boundaries

Albums owns collection membership and album browsing. It does not decide whether metadata is trustworthy.

Review owns suggestion state and canonical metadata acceptance. It does not organize assets into albums.

Gallery and Inspector show the result of both workflows:

```text
Albums
  - owns collection membership
  - does not decide metadata trust

Review
  - owns suggestion review state
  - does not organize assets into albums

Gallery / Inspector
  - shows album membership
  - shows review pending state
  - provides entry points into both workflows
```

## Generate Image Flow

Generation completion should not force navigation into Review Inbox. Generation and metadata review are related, but they are different workflows.

Expected behavior:

```text
Generate Image
  -> create asset/version
  -> persist generation event
  -> show asset in Gallery
  -> create pending metadata suggestion when available
  -> mark asset as Review pending
  -> user may continue in Gallery or open Review Inbox
```

Provider, model, prompt, parameters, lineage, and file metadata are factual generation records and can be shown immediately. Title, description, tags, and category are organization metadata and must pass through Review before becoming canonical.

## Albums Workflow

Albums should close the manual album loop first. Smart albums remain visible as a future extension point, but the main implemented path is manual album creation, browsing, and adding assets.

Primary flow:

```text
Albums view
  -> list albums
  -> create manual album
  -> open album
  -> gallery shows album contents
  -> select asset
  -> Inspector shows album membership
  -> Add to album
```

Albums view should show real album records with name, kind, and item count. Empty state should explain that no albums exist and provide a create action. Creating a manual album should refresh the album list.

Opening an album should put the workspace into album detail mode and reuse Gallery cards for album contents. The album detail header should show album name, kind, item count, and a clear way back to the album list or all Gallery results.

Inspector should show the selected asset's current album memberships. `Add to album` should let the user choose an existing manual album and call the existing core write path. After success, the UI should refresh album list, current Gallery query, and selected asset detail.

This iteration should add a lightweight album read model if needed:

```text
list_albums(library_path) -> Vec<AlbumListItem>

AlbumListItem:
  id
  name
  kind
  item_count
```

Smart album item count can be nullable or deferred if computing it would expand scope. The UI should not expose a full smart album builder in this iteration.

## Review Workflow

Review should close the pending suggestion loop: inspect, edit locally, accept, or reject.

Primary flow:

```text
Generate / Import
  -> create pending metadata suggestion
  -> Review badge increments
  -> Review Inbox lists suggestion
  -> user inspects suggestion
  -> edit title / description / tags / category if needed
  -> accept or reject
  -> refresh Gallery + Inspector
```

Review Inbox should show real pending suggestions with enough context to decide whether to open them: asset fallback title, suggested title, category, tags, and status/source when available.

Selecting a suggestion should show an editable review form. The form state is local UI state, not a mutation of the suggestion record. Accept submits the final confirmed fields to the existing accept command. Reject updates only suggestion status and must not write canonical asset metadata.

Accepted fields should update canonical metadata:

- title
- description
- category
- confirmed tags

After accept or reject, the UI should refresh pending suggestions, the Review badge, current Gallery query, and selected asset detail if the selected asset is affected.

## Data Flow

Albums data flow:

```text
list_albums(library_path)
  -> Albums view

create_manual_album(library_path, name)
  -> refresh list_albums

open album
  -> set GalleryQuery.album_id
  -> query_gallery

add_asset_to_album(album_id, asset_id)
  -> refresh list_albums
  -> refresh query_gallery
  -> refresh get_asset_detail when selected asset matches
```

Review data flow:

```text
list_pending_suggestions(library_path)
  -> Review Inbox

select suggestion
  -> local editable form state

accept_suggestion(final fields)
  -> core writes canonical metadata
  -> refresh pending suggestions
  -> refresh query_gallery
  -> refresh get_asset_detail when selected asset matches

reject_suggestion
  -> core updates suggestion status
  -> refresh pending suggestions
```

Rust core owns query and write semantics. Tauri commands map DTOs and domain errors. React owns interaction state, selected album state, selected suggestion state, form state, loading state, and recoverable errors.

## Error Handling

Expected recoverable cases:

- If no library is open, Albums and Review show explicit empty states and do not retain stale data.
- If the selected album cannot be found, the UI clears selected album state and refreshes album list.
- Adding an asset that is already in an album should behave as success or no-op.
- If accepting a suggestion fails, the review form keeps the user's edits and shows an inline recoverable error.
- If rejecting a suggestion fails, the suggestion remains visible and the UI shows an inline recoverable error.
- Switching libraries clears selected album, selected suggestion, editable review form, and stale Inspector selection.

## Testing Strategy

Core tests:

- Listing albums returns manual albums with item counts.
- Adding an asset to a manual album makes `GalleryQuery.album_id` return that asset.
- Accepting a suggestion writes canonical title, category, and confirmed tags, then removes it from pending results.
- Rejecting a suggestion removes it from pending results without writing canonical metadata.

Tauri tests or mapping coverage:

- `list_albums`.
- `create_manual_album`.
- `add_asset_to_album`.
- `list_pending_suggestions`.
- `accept_suggestion`.
- `reject_suggestion`.

Frontend state tests:

- Creating an album refreshes album list.
- Opening an album updates Gallery query with album id.
- Adding the selected asset to an album refreshes Inspector membership.
- Accepting or rejecting a suggestion updates Review badge and pending list.
- Library switching clears album and review stale state.

Manual acceptance:

```text
create library
import or generate image
create pending suggestion
accept edited suggestion
reject another suggestion
create manual album
add current asset to album
open album
confirm Gallery and Inspector stay consistent
```

## Implementation Sequence

1. Add album list read model and command if missing.
2. Replace placeholder Albums view with real album list, create action, and album detail mode.
3. Wire Inspector `Add to album` to existing album write command and refresh affected state.
4. Upgrade Review Inbox from simple accept/reject rows to selectable suggestion detail with local editable form.
5. Ensure accept/reject refresh pending suggestions, Gallery, and selected Inspector detail.
6. Add focused core, Tauri mapping, and frontend state tests.
7. Run Rust, desktop, and OpenSpec validation commands relevant to the changed surface.

## Deferred Decisions

- Smart album item count is not required for this iteration. If count computation is already cheap through existing query helpers, it may be included; otherwise the UI should show an unavailable state for smart album counts.
- Album removal, rename, deletion, and drag ordering are follow-up work after the manual album create/open/add loop is stable.
- Review remains a metadata trust gate in this iteration. A broader curation inbox that also includes album assignment, rating, and tagging can be designed later after Albums and Review are independently usable.
