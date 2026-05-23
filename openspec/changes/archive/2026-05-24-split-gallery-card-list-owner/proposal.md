# Proposal: Split Gallery Card List Owner

## Problem

`library/gallery.rs` still owns gallery card list projection. It loads latest versions, generation event summaries, version counts, tags, pending review counts, album memberships, task origins, and version tree summaries, then assembles `GalleryAssetView`.

Those card projection details change for different reasons from `GalleryReadService` orchestration, album filtering, sorting, search, detail, and task origin parsing. Keeping the remaining projection in `gallery.rs` leaves the adapter as a read-model hotspot and weakens the ownership model established by the previous gallery split waves.

## Goals

- Extract gallery card list projection into a focused library-internal owner.
- Keep `GalleryReadService` as the runtime-facing query orchestration boundary.
- Preserve Gallery card DTO shape, latest-version selection, event fallback, tag/review/album aggregates, and version tree labels.
- Update architecture inventory and specs so `gallery.rs` is no longer listed as a remaining read-model split target.

## Non-Goals

- Do not change Gallery filters, sort behavior, album context, search semantics, detail views, or task origin parsing.
- Do not change SQLite schema, indexes, migration behavior, or storage format.
- Do not redesign Gallery UI or card layout.

## Impact

- Gallery card projection becomes independently reviewable.
- `gallery.rs` is reduced to query orchestration and service trait implementation.
- The gallery read-model split sequence has no remaining explicit owner-split target in the architecture inventory.
