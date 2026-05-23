# Proposal: Split Gallery Read Model Owners

## Problem

The systematic DDD review identified `crates/imglab-core/src/library/gallery.rs` as a large read-model hotspot. It currently owns gallery list queries, search, album filter context, version tree summaries, asset detail, inspector detail, task origin lookups, file context, and several shared predicates.

This makes future performance work risky because search, gallery list, detail, and version tree changes all happen in one module.

## Goals

- Split gallery read-model code by ownership and change reason.
- Preserve current SQLite schema, query results, CLI JSON, Tauri payloads, and desktop behavior.
- Start with low-risk extraction of search read-model behavior.
- Keep compatibility regression tests passing while future waves split version tree and asset detail owners.

## Non-Goals

- Do not change gallery query semantics.
- Do not introduce FTS5, projection tables, Tantivy, DuckDB, or PostgreSQL in this change.
- Do not change resource library schema or migrations.
- Do not redesign desktop Gallery or Albums UI.

## Impact

- Search read-model behavior moves out of `gallery.rs` into a focused module.
- `gallery.rs` keeps shared gallery loading behavior during the first extraction.
- Follow-up tasks remain for version tree, album filter context, and asset detail extraction.
