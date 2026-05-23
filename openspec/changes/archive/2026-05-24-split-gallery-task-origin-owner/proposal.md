# Proposal: Split Gallery Task Origin Owner

## Problem

`library/gallery.rs` still owns task origin loading for gallery cards. This projection reads task outputs and task rows, parses task type, status, provider, and operation values, then maps them into `TaskOriginView`.

Task origin data changes for different reasons from gallery card composition. Keeping the task projection inside the gallery list adapter couples task storage semantics to gallery DTO assembly and makes later task bounded-context hardening harder to review.

## Goals

- Extract gallery task origin projection into a focused library-internal read-model owner.
- Keep `GalleryReadService` and gallery card composition behavior unchanged.
- Preserve task origin precedence: asset-version output before asset output, with latest task rows winning per target.
- Preserve SQLite schema, task status parsing, operation parsing, and runtime DTO shape.
- Update architecture inventory and specs so the remaining `gallery.rs` target is limited to gallery card composition.

## Non-Goals

- Do not change task scheduler behavior, retry policy, task status transitions, or task output persistence.
- Do not change Gallery list filtering, sorting, album context, or card DTO shape.
- Do not change SQLite schema, indexes, or migration behavior.
- Do not redesign Queue or Gallery UI.

## Impact

- Task origin projection becomes independently reviewable.
- `gallery.rs` is reduced toward gallery list composition and query orchestration.
- Future work can focus on splitting the remaining gallery card read model without carrying task parsing logic.
