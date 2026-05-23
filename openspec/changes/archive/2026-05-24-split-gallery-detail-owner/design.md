# Design: Split Gallery Detail Owner

## Overview

This change extracts asset detail and inspector detail projection into a dedicated `gallery_detail` module under the library infrastructure adapter.

`GalleryReadService` continues to open the library database and enforce asset existence through the same public query surface. The new module receives the library path and SQLite connection, then composes the detail DTO from version summaries, version tree data, generation events, reference source rows, pending suggestions, and file context.

## Boundaries

- `gallery.rs` owns `GalleryReadService` orchestration, gallery list loading, filtering delegation, and remaining task origin/card helpers.
- `gallery_detail.rs` owns asset detail, inspector detail, generation event detail, reference source lookup, pending suggestion summaries, asset album/tag detail, and file context projection.
- `gallery_version_tree.rs` remains the owner for version tree naming, lineage, promoted-source lookup, and tree issue reporting.
- `storage.rs` remains the owner for file digest calculation.

## Compatibility

The change must preserve:

- `AssetDetailView` and `AssetInspectorDetailView` payload shape.
- Current version selection and fallback to latest generation event.
- Reference source suppression for same-asset input versions.
- Pending suggestion summary parsing and counts.
- File integrity status values: `verified`, `hash_mismatch`, and `missing`.
- Existing SQLite schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-detail-owner --strict`
- `openspec validate --specs --strict`
