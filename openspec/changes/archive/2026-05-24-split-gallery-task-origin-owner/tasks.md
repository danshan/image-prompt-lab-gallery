# Tasks: Split Gallery Task Origin Owner

## Implementation

- [x] Add a focused `gallery_task_origin` module for task origin projection.
- [x] Delegate gallery card task origin loading to the new owner.
- [x] Remove task-origin SQL and parsing helpers from `gallery.rs`.
- [x] Update DDD boundary inventory and canonical specs.

## Verification

- [x] Run `cargo fmt --all --check`.
- [x] Run `cargo test -p imglab-core`.
- [x] Run `scripts/check-architecture.sh`.
- [x] Run `openspec validate split-gallery-task-origin-owner --strict`.
- [x] Run `openspec validate --specs --strict`.
- [x] Run `git diff --check`.
