# Design: Split Gallery Task Origin Owner

## Overview

This change extracts task origin projection into a dedicated `gallery_task_origin` module under the library infrastructure adapter.

The new owner loads task origins from `task_outputs` joined to `tasks`, maps storage values to task domain/interface view values, and exposes a small lookup structure for gallery card composition.

## Boundaries

- `gallery_task_origin.rs` owns task origin SQL, task type/status parsing, operation parsing, and target lookup maps for gallery read models.
- `gallery.rs` owns `GalleryReadService` orchestration and gallery card DTO composition.
- `tasks.rs` remains the repository owner for task write/list/detail behavior.
- `domain::task::policies` remains the owner for task transition and retry policy.

## Compatibility

The change must preserve:

- `GalleryAssetView.task_origin` payload shape.
- Existing task origin precedence for card display.
- Existing parsing of `task_type`, `status`, provider, and operation values.
- Existing SQLite schema and task output storage semantics.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `scripts/check-architecture.sh`
- `openspec validate split-gallery-task-origin-owner --strict`
- `openspec validate --specs --strict`
