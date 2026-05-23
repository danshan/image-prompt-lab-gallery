# Proposal: Split Gallery Filter Owner

## Problem

`library/gallery.rs` still owns gallery query orchestration, album filter context loading, smart album preview filtering, shared predicate application, and album-order sorting. These concerns change for different reasons from asset detail, inspector detail, file context, and task origin lookups.

The shared filter pipeline is already important for correctness because Gallery and smart albums must preserve overlapping semantics. Keeping it inside the gallery adapter makes future query-shape optimization and smart album behavior changes harder to review.

## Goals

- Extract album filter context, shared gallery predicate application, smart album preview filtering, and gallery sort behavior into a focused owner.
- Preserve Gallery query results, smart album semantics, album-order validation, SQLite schema, and runtime DTO payloads.
- Keep `GalleryReadService` as the runtime-facing orchestration adapter.
- Document the reduced remaining split targets for `gallery.rs`.

## Non-Goals

- Do not change Gallery filter semantics.
- Do not change smart album query syntax or allowed fields.
- Do not change SQLite schema, projection strategy, or indexing strategy.
- Do not redesign Gallery or Albums UI.

## Impact

- Gallery filtering and album-scope behavior become a focused read-model owner.
- Search can reuse the same text predicate without depending on the monolithic gallery adapter.
- Future waves can focus on asset detail, inspector detail, task origin, and file context.
