# Tasks: Split Gallery Card List Owner

## Implementation

- [x] Add a focused `gallery_cards` module for base Gallery card projection.
- [x] Delegate `GalleryReadService::query_gallery` card loading to the new owner.
- [x] Move card-only helpers out of `gallery.rs`.
- [x] Update DDD boundary inventory and canonical specs.

## Verification

- [x] Run `cargo fmt --all --check`.
- [x] Run `cargo test -p imglab-core`.
- [x] Run `scripts/check-architecture.sh`.
- [x] Run `openspec validate split-gallery-card-list-owner --strict`.
- [x] Run `openspec validate --specs --strict`.
- [x] Run `git diff --check`.
