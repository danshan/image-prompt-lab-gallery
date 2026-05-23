# Proposal: Split Gallery Detail Owner

## Problem

`library/gallery.rs` still owns gallery list composition, asset detail assembly, inspector detail assembly, pending suggestion summaries, reference source lookup, and file integrity context. These read models change for different reasons and already have different correctness risks.

Asset detail and inspector detail are user-facing drill-down projections. They combine canonical metadata, version tree naming, generation event data, reference source data, lineage, pending review state, and filesystem integrity. Keeping that projection in the gallery list adapter makes future performance and ownership changes harder to review.

## Goals

- Extract asset detail and inspector detail projection behavior into a focused library-internal read-model owner.
- Keep `GalleryReadService` as the runtime-facing query adapter while delegating detail assembly.
- Preserve SQLite schema, runtime DTO shape, version tree naming, reference source behavior, and file integrity semantics.
- Update architecture inventory and specs so the remaining `gallery.rs` targets are explicit.

## Non-Goals

- Do not change Gallery list filtering, sorting, smart album behavior, or search semantics.
- Do not change asset/version lineage rules or version number allocation.
- Do not change filesystem layout, checksum algorithms, or library schema.
- Do not redesign desktop Gallery or inspector UI.

## Impact

- Detail and inspector read models become independently reviewable.
- `gallery.rs` is reduced toward list composition and runtime orchestration.
- Future waves can focus on task origin lookup and remaining gallery card composition without carrying detail-specific helpers.
