# Design: Split Gallery Read Model Owners

## Overview

This change splits `library/gallery.rs` incrementally. The first implementation wave extracts search-specific filtering and result mapping into a focused search read-model module while reusing the existing gallery card loader.

## Boundaries

- `gallery.rs` remains the compatibility read adapter for `GalleryReadService`.
- `gallery_search.rs` owns `SearchService` query behavior and search-specific text matching.
- Shared gallery card loading remains in `gallery.rs` until a later projection/read-model split.
- Later waves can extract version tree and asset detail without changing search behavior.

## Compatibility

The change must preserve:

- `SearchService::search` output shape.
- CLI `search` JSON behavior.
- Gallery list and detail behavior.
- Existing library schema and file layout.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `cargo test -p imglab-cli`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-read-model-owners --strict`
