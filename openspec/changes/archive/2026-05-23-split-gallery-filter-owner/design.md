# Design: Split Gallery Filter Owner

## Overview

This change extracts gallery filtering and album filter behavior into a dedicated library-internal read-model module.

The new owner handles normalized album filters, album context validation, manual and smart album membership filtering, shared predicate application, and sorting. `gallery.rs` continues to load cards and compose runtime DTOs, then delegates filter and sort behavior.

## Boundaries

- `gallery_filtering.rs` owns album filter context, smart album preview filtering, shared predicate conversion, text matching, and gallery sort behavior.
- `gallery.rs` owns `GalleryReadService` orchestration, gallery card loading, asset detail, inspector detail, file context, and task origin lookup.
- `gallery_search.rs` consumes the shared text predicate from the filtering owner instead of depending on `gallery.rs`.

## Compatibility

The change must preserve:

- Gallery filter and sort result semantics.
- Smart album preview and unassigned album behavior.
- `album_order` validation and sorting behavior.
- CLI, daemon, and Tauri runtime payload shape.
- Existing library schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-filter-owner --strict`
