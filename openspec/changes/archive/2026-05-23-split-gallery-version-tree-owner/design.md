# Design: Split Gallery Version Tree Owner

## Overview

This change continues the gallery read-model decomposition by extracting version tree ownership from `library/gallery.rs`.

The new owner is a library-internal read-model module. It owns row loading, tree construction, degraded-tree issue reporting, promoted-source lookup, and asset-scoped lineage traversal. `gallery.rs` remains the `GalleryReadService` adapter and calls the new module for detail and list summaries.

## Boundaries

- `gallery.rs` keeps `GalleryReadService` orchestration, query validation, album filter context, gallery card loading, asset detail composition, inspector detail composition, and file context loading.
- `gallery_version_tree.rs` owns version tree rows, summary construction, root naming, cross-asset parent issue reporting, cycle detection, promoted source lookup, and asset-scoped lineage traversal.
- The module boundary stays inside `imglab-core` infrastructure compatibility code. It does not create a new public API or persistence contract.

## Compatibility

The change must preserve:

- `AssetDetailView.version_tree`, `version_tree_issues`, `focused_version_tree_name`, `lineage`, and `promoted_from`.
- `GalleryAssetView.current_version_tree_name` and `version_tree_branch_count`.
- Existing version tree degradation behavior for missing parents, cross-asset parents, and cycles.
- Existing library schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-version-tree-owner --strict`
