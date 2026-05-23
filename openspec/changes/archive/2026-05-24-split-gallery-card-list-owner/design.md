# Design: Split Gallery Card List Owner

## Overview

This change extracts gallery card list projection into a dedicated `gallery_cards` module under the library infrastructure adapter.

`GalleryReadService::query_gallery` continues to validate the query, load album contexts, delegate card projection, apply shared filters, apply album filters, sort the final list, and attach album context. The new owner only assembles the base `GalleryAssetView` rows before query-specific filtering and sorting.

## Boundaries

- `gallery_cards.rs` owns gallery card SQL, latest version lookup, generation event summaries, version counts, tag aggregates, pending review counts, album membership aggregates, and card DTO assembly.
- `gallery.rs` owns runtime-facing `GalleryReadService` orchestration and service trait wiring.
- `gallery_filtering.rs` owns filters, album context, smart album preview filtering, and sorting.
- `gallery_version_tree.rs` owns tree naming and branch summaries.
- `gallery_task_origin.rs` owns task origin projection.

## Compatibility

The change must preserve:

- `GalleryAssetView` payload shape.
- Latest version selection and generation event fallback behavior.
- Current version tree label behavior.
- Tags, pending review counts, album memberships, and task origin values.
- Existing SQLite schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-card-list-owner --strict`
- `openspec validate --specs --strict`
