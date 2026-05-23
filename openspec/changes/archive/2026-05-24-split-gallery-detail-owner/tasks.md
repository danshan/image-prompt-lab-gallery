# Tasks: Split Gallery Detail Owner

## Implementation

- [x] Add a focused `gallery_detail` module for asset detail and inspector detail projections.
- [x] Delegate `GalleryReadService::get_asset_detail` and `get_asset_inspector_detail` to the new owner.
- [x] Move detail-only helpers out of `gallery.rs`.
- [x] Update DDD boundary inventory and canonical specs.

## Verification

- [x] Run `cargo fmt --all --check`.
- [x] Run `cargo test -p imglab-core`.
- [x] Run `scripts/check-architecture.sh`.
- [x] Run `openspec validate split-gallery-detail-owner --strict`.
- [x] Run `openspec validate --specs --strict`.
- [x] Run `git diff --check`.
