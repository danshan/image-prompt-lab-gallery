# Full Performance And Code Review Cleanup Design

## Context

The project is a local-first desktop application for AI image prompt, generated image, metadata, album, task, and version lineage management. The current architecture uses:

- Rust workspace for core business logic.
- SQLite-backed resource libraries.
- Tauri + React desktop shell.
- A daemon crate for long-running task execution.
- Provider adapters for fake and Codex CLI image generation.

This cleanup uses `docs/PERFORMANCE_REVIEW.md` as the baseline review input. The review identifies several confirmed risks in the current code:

- Gallery and search hot paths issue N+1 SQL queries.
- SQLite indexes are missing for several heavily queried columns.
- File hashing and image dimension detection read full files into memory.
- The legacy Tauri `generate_image` command still runs synchronously.
- The React desktop entry file is too large and mixes state, view, IPC, queue, review, album, and gallery concerns.
- Gallery images load full-resolution files without lazy loading.
- Several desktop actions refresh data through sequential IPC waterfalls.
- Daemon request handling and scheduler polling hold coarse state boundaries.
- App log discovery scans broad temp directories.

The user wants a full cleanup in one implementation effort, not a narrow first pass. The implementation should still be internally staged so each class of change remains reviewable and testable.

## Goals

- Complete a full technical and code review cleanup based on `docs/PERFORMANCE_REVIEW.md`.
- Fix confirmed performance risks instead of only documenting them.
- Keep behavior stable while reducing query count, memory usage, UI blocking, and unnecessary renders.
- Preserve local-first resource library semantics unless evidence shows SQLite cannot support the target workload.
- Add enough tests and validation hooks to prove the refactor is behavior-preserving.
- Leave the codebase in a more maintainable shape, with clearer module and component boundaries.

## Non-Goals

- Do not add new product features unrelated to the review findings.
- Do not redesign the visual language of the desktop UI.
- Do not replace SQLite by default.
- Do not replace the daemon transport contract unless current evidence proves it is a blocker.
- Do not introduce a broad repository trait layer without a concrete testing or runtime boundary.
- Do not change managed resource library directory layout.
- Do not implement native OpenAI or Grok image providers.

## Database Strategy

SQLite remains the starting point, but it is not assumed to be permanent.

The current highest-impact issues are caused by query shape, missing indexes, repeated per-row lookups, and blocking call boundaries. Replacing SQLite would not automatically fix these problems. It could also increase local-first desktop deployment complexity.

The cleanup will use this decision ladder:

1. Fix query shape and indexes while keeping SQLite.
2. Add SQLite-native read-model improvements if needed, such as FTS5 or projection tables.
3. Add a supplemental index if search requirements outgrow SQLite, for example Tantivy for local full-text search.
4. Add DuckDB only for analytical workloads where SQLite is not the right query engine.
5. Consider PostgreSQL only if local-first serverless operation is no longer a hard constraint.

The implementation should include a storage sufficiency checkpoint after the core hot-path work. The checkpoint should evaluate query count, representative latency, lock contention, and search requirements before proposing any replacement.

## Recommended Approach

Implement the cleanup as one full effort with four internal waves:

1. Core hot path.
2. Desktop rendering and state.
3. Daemon and operational cleanup.
4. Storage and search sufficiency checkpoint.

This gives the user the requested full cleanup while keeping each wave small enough to review, test, and roll back.

## Wave 1: Core Hot Path

### Scope

- `crates/imglab-core/src/library/gallery.rs`
- `crates/imglab-core/src/library/schema.rs`
- `crates/imglab-core/src/hash.rs`
- `crates/imglab-core/src/library/storage.rs`
- Legacy desktop `generate_image` command in `apps/desktop/src-tauri/src/lib.rs`

### Design

Gallery and search should stop loading all rows and then issuing per-asset SQL queries. The new read path should load the base asset set and related data in batches:

- Latest current version per asset.
- Latest generation event per asset.
- Version counts.
- Version labels and dimensions.
- Tags per asset.
- Pending review counts.
- Manual album membership and sort order.

Assembly should happen in memory through focused maps keyed by asset id. Album filtering and album order sorting should use one preloaded membership or sort-order map, not SQL inside `retain` or sort comparators.

Search should push tag-provider-status filters into SQL where practical. If a filter is still applied in memory, all required related data must already be batch loaded.

Schema migration should add indexes for hot-path columns:

- `assets(library_id)`
- `asset_versions(asset_id, created_at DESC)`
- `generation_events(asset_id, started_at DESC)`
- `metadata_suggestions(asset_id, status)`
- `album_items(asset_id)`
- `album_items(album_id, sort_order)`
- `asset_tags(asset_id)`
- `asset_tags(tag_id)`
- `tags(name)`

File hashing should compute digest incrementally from a bounded buffer. Image dimension detection should read only the header range needed for PNG, JPEG, and WebP instead of loading full image files.

The legacy Tauri `generate_image` command should not execute long-running provider work synchronously. It should either use `spawn_blocking` with correct error mapping or be explicitly routed through the daemon path when parity is complete. For this cleanup, use the smallest behavior-preserving fix first: `spawn_blocking`.

### Validation

- Add or update Rust tests for gallery query with multiple assets, tags, pending suggestions, versions, and manual album ordering.
- Add a regression test that album order sorting does not depend on per-comparator database calls by validating equivalent output after batch preload.
- Add tests for streaming SHA-256 and MD5 digest results against known inputs.
- Add tests for image dimension parsing from bounded header reads.
- Run:

```bash
cargo fmt --all --check
cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli
cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok
```

## Wave 2: Desktop Rendering And State

### Scope

- `apps/desktop/src/main.tsx`
- `apps/desktop/src/workbench-state.ts`
- Potential new files under `apps/desktop/src/components/`, `apps/desktop/src/hooks/`, and `apps/desktop/src/lib/`

### Design

The desktop frontend should be split by workflow boundaries instead of mechanically splitting by file size. The initial component boundaries should be:

- Shell and view routing.
- Library sidebar.
- Workspace toolbar.
- Gallery workspace.
- Albums workspace.
- Review inbox.
- Task workspace.
- Settings workspace.
- Inspector.
- Lightbox.

Stateful data loading should move behind focused hooks where it reduces coupling:

- `useGallery`
- `useAlbums`
- `useReview`
- `useTasks`
- `useLibrarySettings`

The first pass should avoid adding a global state library. React state remains sufficient if invalidation is explicit and derived data is memoized.

Gallery images should add:

- `loading="lazy"`
- `decoding="async"`
- Stable dimensions or aspect-ratio constraints.

Search and smart album preview should avoid recomputing or re-fetching on every keystroke. Use debounced query inputs for IPC-backed gallery refresh and `useMemo` for local derived previews.

Derived values such as available providers, queue count, pending count, and filtered gallery collections should be memoized when passed into large child components.

Actions that currently run independent refreshes sequentially should use `Promise.all` where ordering is not semantically required. Detail refresh remains conditional and can run after the gallery/suggestion refresh if it depends on selected state.

Polling and delayed task waits should keep timeout handles in refs and clear them on unmount. Any recursive polling should have a stop condition, backoff, or cancellation path.

### Validation

- Run TypeScript build and existing frontend tests.
- Add or update tests in `apps/desktop/tests` for debounced state helpers if logic is extracted to pure modules.
- Manually smoke test gallery search, album operations, review accept/reject, task queue, settings logs, and lightbox.
- Run:

```bash
cd apps/desktop
npm run test
npm run build
```

## Wave 3: Daemon And Operational Cleanup

### Scope

- `crates/imglab-daemon/src/lib.rs`
- `crates/imglab-daemon/src/main.rs`
- `apps/desktop/src-tauri/src/daemon_client.rs`
- `apps/desktop/src-tauri/src/app_logs.rs`
- Related tests in daemon and desktop Tauri crates

### Design

Daemon HTTP handling should reduce the duration of global state locking. The first implementation should keep the current transport and avoid introducing a new web framework. Prefer these steps:

- Parse and authenticate request without holding the state lock.
- Acquire mutable state only for route execution that needs it.
- Keep health and capabilities responses lock-free when possible.
- Avoid holding state lock across long-running task execution.

Scheduler should avoid cloning full daemon state every interval when there is no eligible work. Add a cheap check for opened libraries and potentially runnable tasks before deep clone or tick execution. If the current design needs a snapshot for safe execution, make snapshot creation conditional.

App log listing should only scan app-owned directories:

- Desktop daemon runtime/log root.
- Daemon log root.
- Known provider log subdirectories created by this application.

It should not scan the entire system temp directory. Blocking filesystem scanning should run off the Tauri command thread.

Daemon client timeout and retry behavior should be context-aware:

- Short timeout for health checks.
- Longer timeout for task detail and log reads.
- Backoff for repeated transient connection failures.

Task/job retention should be explicit. Completed and failed task history is persisted in the library. In-memory caches should have retention limits or cleanup by age/status.

### Validation

- Add daemon tests for lock-free health/capabilities behavior where practical.
- Add scheduler tests for no-work iterations.
- Add app log tests proving broad temp files are ignored.
- Run:

```bash
cargo test --offline -p imglab-daemon
cargo check --offline -p imglab-desktop
```

## Wave 4: Storage And Search Sufficiency Checkpoint

### Scope

- Bench or test harnesses under existing Rust test structure.
- Documentation updates in `docs/` and current OpenSpec specs if storage strategy changes.

### Design

After Waves 1-3, add a checkpoint that records whether SQLite remains sufficient.

The checkpoint should use representative local datasets, or synthetic datasets if no real library is available:

- 10k assets.
- 50k assets if runtime is reasonable.
- Multiple versions per asset.
- Multiple tags per asset.
- Pending review suggestions.
- Manual albums and smart album queries.

Evaluate:

- Gallery query P50/P95.
- Search query P50/P95.
- Import/generation write latency.
- Concurrent desktop short write plus daemon task write behavior.
- Query complexity and maintainability.

Decision outcomes:

- Keep SQLite with indexes.
- Add SQLite FTS5 for text search.
- Add projection tables for gallery read models.
- Add Tantivy as a supplemental local search index.
- Add DuckDB for analytical queries.
- Revisit PostgreSQL only if local-first constraints are relaxed.

### Validation

- Document the checkpoint outcome in `docs/PERFORMANCE_REVIEW.md` or a follow-up performance notes file.
- Do not migrate storage backend without a dedicated design update and explicit approval.

## Risks

- Rewriting gallery SQL can introduce subtle differences in filtering, album ordering, or fallback event selection.
- Adding indexes changes database files and migration behavior. Tests must cover existing-library migrations.
- Frontend extraction can create stale closures or broken refresh ordering.
- Parallel or batched refreshes can expose assumptions that were accidentally serialized before.
- Daemon lock changes can break recovery or task state transitions if route boundaries are not clear.
- Replacing SQLite prematurely could increase operational complexity without fixing the actual hot paths.

## Rollback Strategy

- Keep each wave in separate commits if possible.
- For Wave 1, retain public service method signatures unless a change is required by correctness.
- For Wave 2, extract components without changing user-visible workflows first, then add debounce and memoization.
- For Wave 3, keep transport endpoints and response shapes stable.
- For Wave 4, document evidence before proposing storage replacement.

## Implementation Order

1. Add or update review-focused tests around current behavior.
2. Implement schema indexes and core batch gallery/search read path.
3. Implement streaming hash and bounded dimension reads.
4. Move legacy `generate_image` onto blocking execution.
5. Extract desktop workflow components and hooks.
6. Add gallery lazy image loading, debounced query refresh, memoized derived values, and batched refresh calls.
7. Reduce daemon lock scope and scheduler no-work clone cost.
8. Restrict app log scanning to app-owned roots.
9. Add storage/search sufficiency benchmark or synthetic test evidence.
10. Update docs and current specs with final findings.

## Acceptance Criteria

- Gallery and search no longer perform per-asset SQL lookups for tags, latest event, review count, or album membership.
- Manual album order sorting does not call SQL from the comparator.
- Hot-path SQLite indexes exist and migrate safely.
- Hashing and dimension detection use bounded memory.
- Legacy `generate_image` no longer blocks synchronously on the Tauri command thread.
- Gallery images lazy-load and decode asynchronously.
- Search refresh is debounced and derived frontend data is memoized where it affects large children.
- Independent refresh actions avoid unnecessary IPC waterfalls.
- Polling paths clean up timeout handles.
- Daemon health/capabilities remain responsive during scheduler activity.
- App log listing does not scan the full system temp directory.
- Validation commands pass or any failures are documented with concrete cause.
- A storage/search checkpoint records whether SQLite is still sufficient and what would trigger a future replacement.

