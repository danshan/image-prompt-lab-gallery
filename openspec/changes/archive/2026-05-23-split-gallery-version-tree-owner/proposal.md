# Proposal: Split Gallery Version Tree Owner

## Problem

The systematic DDD review identified `crates/imglab-core/src/library/gallery.rs` as a read-model hotspot. The previous search extraction reduced one change reason, but the module still owns version tree construction, branch summaries, promoted-source lookup, and asset-scoped lineage assembly alongside gallery list and detail orchestration.

Version tree behavior is business-facing read logic with its own invariants: root naming, cross-asset parent degradation, cycle detection, branch counts, promoted source labels, and asset-scoped lineage traversal. Keeping this logic embedded in the gallery adapter makes future lineage and performance work harder to review.

## Goals

- Extract version tree, promoted source, and lineage read-model behavior into a focused owner.
- Preserve current SQLite schema, query semantics, DTO payloads, CLI output, daemon behavior, and desktop behavior.
- Keep `GalleryReadService` as the runtime-facing compatibility adapter while moving version tree implementation details behind a module boundary.
- Document the new owner and the remaining gallery read-model split targets.

## Non-Goals

- Do not change asset version numbering, tree naming semantics, or lineage rules.
- Do not introduce projection tables, FTS5, Tantivy, DuckDB, PostgreSQL, or schema migrations in this change.
- Do not redesign Gallery, Albums, or asset detail UI.
- Do not change provider output persistence or generation event persistence.

## Impact

- `gallery.rs` delegates version tree, promoted source, and asset-scoped lineage logic to a dedicated read-model module.
- Version tree behavior becomes easier to test and evolve independently from gallery list filtering and album context logic.
- Future changes can split asset detail and gallery list projections without moving lineage algorithms again.
