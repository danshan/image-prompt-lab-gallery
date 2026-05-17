# Album and Review Workflow Expansion Design

## Purpose

This change expands Album and Review workflows as one product change with three bounded areas: Album domain behavior, Smart Album builder behavior, and Review workflow behavior.

The goal is to make Albums and Review usable for real curation work without moving business semantics into the desktop UI. Rust core remains the source of truth for album ordering, smart query validation, batch review writes, suggestion history, and confidence score normalization rules.

## Scope

In scope:

- Drag ordering for the album list.
- Drag ordering for assets inside manual albums.
- Remove assets from manual albums.
- Rename and delete albums.
- Batch add multiple assets to a manual album.
- A complete first-version Smart Album builder using typed query fields.
- Batch accept and reject for Review suggestions.
- Suggestion history comparison with field-level value picking into the current draft.
- Field-level Review regeneration and full suggestion regeneration.
- Add selected Review assets to a manual album.
- Confidence score visualization based on a stable score contract.

Out of scope:

- Nested smart query groups or a generic rules engine.
- Album sharing, collaboration, or cloud sync.
- Graph-style lineage visualization.
- Persisted per-user UI layouts.
- Native provider clients beyond the existing Codex CLI metadata generation boundary.

## Key Decisions

This is implemented as one OpenSpec change, but the implementation is kept in separate boundaries:

- Album domain: album CRUD, album ordering, manual album item ordering, remove asset, and batch add assets.
- Smart Album domain: typed builder contract and core validation for query fields.
- Review domain: selection model, batch actions, suggestion history comparison, regeneration, album add actions, and confidence visualization.

This shape allows the cross-workflow action "add selected Review assets to album" without mixing Album and Review persistence rules.

## Album Architecture

The `albums` table gains a persisted `sort_order INTEGER NOT NULL DEFAULT 0`. `list_albums` returns albums ordered by `sort_order, name`, and migration assigns deterministic order to existing albums.

Manual album asset ordering continues to use `album_items.sort_order`. Reorder writes are album-scoped and transactional. The input should identify the album and the ordered asset ids for that album. Core validates that every id belongs to the target manual album before writing any new order.

New Album service operations:

- `rename_album`
- `delete_album`
- `remove_asset`
- `batch_add_assets`
- `reorder_albums`
- `reorder_album_items`

Manual album operations reject smart albums where the operation has no meaning. Duplicate batch add entries are treated as idempotent success. Missing assets, missing albums, or wrong album kind return recoverable domain errors.

## Smart Album Builder

Smart albums move from raw JSON key allowlisting to a typed query contract. The builder supports:

- `text`
- `tags`
- `providers`
- `minRating`
- `reviewStatus`
- `category`
- `status`
- `createdAtFrom`
- `createdAtTo`
- `sort`

The created date range uses `assets.created_at`. Core validates field names, value types, rating range, date format, and sort values. The stored smart query remains JSON for compatibility, but the accepted JSON shape is the typed contract, not arbitrary user-authored JSON.

Smart album results are computed from the same core gallery semantics used by Gallery query. Smart albums do not write `album_items`.

## Review Architecture

Metadata suggestions remain append-only history. Regenerating a full suggestion creates a new pending suggestion record and preserves previous pending, accepted, and rejected records.

New Review read behavior:

- List pending suggestions for the inbox.
- List suggestion history for an asset, ordered by created time.
- Include status, created time, reviewed time, suggested fields, tags, category, and raw confidence JSON.

Batch accept receives per-suggestion final payloads. The currently open Review form may contribute local draft values, while other selected suggestions use their stored suggestion values. Core validates every item first, then applies the batch in one transaction. If any suggestion is missing, not pending, or fails validation, no canonical metadata is written.

Batch reject marks all selected pending suggestions as rejected in one transaction. If any selected suggestion is no longer pending, the whole operation fails and the UI refreshes the inbox.

## Review UI Workflow

The Review Inbox uses a multi-select list. Batch actions operate on selected suggestions, falling back to the current suggestion when no explicit selection exists.

Review detail contains:

- Current editable draft.
- Suggestion history table for the same asset.
- Field-level pick controls so title, description, schema prompt, tags, and category can be copied from any history row into the draft.
- Field-level regeneration for title, description, and schema prompt. This updates only the current draft.
- Full suggestion regeneration. This creates a new suggestion record and refreshes history.
- Add selected Review assets to a manual album.
- Confidence visualization.

The UI must make the distinction clear between local draft changes and persisted suggestion records. Accepting a suggestion is the only action that writes canonical asset metadata.

## Confidence Contract

`confidence_json` is normalized for display using this contract:

```json
{
  "overall": 0.82,
  "fields": {
    "title": 0.9,
    "description": 0.76,
    "schemaPrompt": 0.7,
    "tags": 0.88,
    "category": 0.64
  }
}
```

Scores may be represented as `0..1` or `0..100`. UI displays normalized `0..100` values. Missing or malformed values display as unknown and do not block Review actions. Unknown keys are ignored for display and preserved in the raw stored JSON.

## Desktop Workflow

Albums view:

- Album list supports drag reorder, rename, and delete.
- Manual album detail supports drag reorder, remove asset, and batch add selected Gallery assets.
- Smart album detail shows builder controls and a live gallery preview from the typed query.

Review view:

- Pending suggestions support multi-select.
- Batch accept and reject apply to selected suggestions.
- Add to album applies to selected suggestions' assets, or the current suggestion's asset when nothing is selected.
- History comparison and field picking update only the local draft.

Gallery and Inspector refresh after writes so badges, album membership, canonical metadata, and review counts stay consistent.

## Error Handling

Album operations:

- Deleting a missing album returns a recoverable error.
- Deleting an album removes manual album memberships for that album.
- Smart albums reject manual-only operations: add asset, remove asset, and reorder items.
- Reorder operations validate full scope membership before writing.
- Batch add is idempotent for existing memberships but fails atomically for invalid album kind or missing assets.

Review operations:

- Batch accept and reject are transactional.
- Batch accept validates all pending status and category rules before writing canonical metadata.
- Full suggestion regeneration failure leaves the current draft and existing history unchanged.
- Confidence parse failure only affects visualization.

## Testing Strategy

Core tests:

- Album list reorder persists and survives reopening the library.
- Manual album item reorder affects album-scoped gallery order.
- Rename, delete, remove, and batch add cover happy path and invalid album kind.
- Smart album query validation covers text, tags, providers, min rating, review pending, category, status, and created date range.
- Suggestion history includes pending, accepted, and rejected records in stable order.
- Batch accept rolls back when one selected suggestion is invalid.
- Confidence normalization handles `0..1`, `0..100`, missing fields, and malformed JSON.

Desktop state tests:

- Review multi-select state.
- Batch accept combines the current draft with stored values for other selected suggestions.
- Field picking from history updates the draft only.
- Library switches clear album and review selection state.

Tauri command tests:

- Input and output keep camelCase contracts.
- Batch operation failures use the existing `CommandError` shape.

Manual QA:

- Drag album list, restart the app, verify order persists.
- Drag manual album assets, reopen album, verify order persists.
- Batch add assets to album and verify duplicates are harmless.
- Batch accept and reject suggestions, verify Review count and Gallery badges refresh.
- Regenerate full suggestion and verify history gains a new row without overwriting old records.

## Risks

The largest risk is scope coupling between Album and Review UI. The mitigation is to keep shared behavior limited to explicit commands: batch add assets to album and refresh read models after writes.

The second risk is smart album query drift from Gallery query semantics. The mitigation is to keep typed smart query conversion in core and reuse Gallery query validation where possible.

The third risk is partial batch writes. The mitigation is transactional validation and write paths in core, with no optimistic partial success semantics in this iteration.
