# Performance Code Review — image-prompt-lab-gallery

> Review date: 2025-07-08  
> Scope: All Rust crates (imglab-core, imglab-cli, imglab-daemon, imglab-provider-codex, imglab-provider-grok), Tauri desktop app (backend + React frontend)

---

## Summary

| Severity | Count | Primary Domain |
|----------|-------|----------------|
| Critical | 8 | DB N+1 queries, blocking UI thread, monolithic frontend, image loading |
| High | 8 | Missing indexes, full-file buffering, no memoization, API waterfall |
| Medium | 10 | Redundant queries, daemon concurrency, debounce gaps |
| Low | 5 | Micro-optimizations, timeout tuning |

Top 3 systemic risks:

1. **Gallery query engine N+1 pattern** — 1,000 assets = ~7,000 SQL queries per gallery load
2. **Synchronous `generate_image` on Tauri invoke** — blocks UI for minutes during image generation
3. **3,571-line monolithic React component** with 40+ useState — every keystroke re-renders entire app

---

## CRITICAL

### C-01. N+1 Query Pattern in Gallery Load — O(N x 7) queries

**File:** `crates/imglab-core/src/library/gallery.rs:338-408`

`load_gallery_asset_views` loads all assets, then for each asset issues 6-7 additional queries:

1. `load_latest_asset_version` (2 queries)
2. `load_generation_event` (1 query)
3. `count_asset_versions` (1 query)
4. `load_version_label` (1 query)
5. `load_version_dimensions` (1 query)
6. `load_asset_tags` (1 query)
7. `pending_review_count` (1 query)

**Impact:** 1,000 assets = ~7,000+ SQL round-trips in a single gallery load.

**Fix:** Replace with joined SQL — join `assets`, `asset_versions`, `generation_events`, `tags`, `asset_tags`, `metadata_suggestions` with subqueries. Pre-load into `HashMap<AssetId, ...>` and assemble in-memory.

---

### C-02. N+1 Query in `search_assets` — Per-asset tag/event lookups

**File:** `crates/imglab-core/src/library/gallery.rs:252-281`

Filtering by tags calls `asset_has_all_tags` which issues one `COUNT` query **per tag per asset**. Worst case: 500 assets x 3 tags = 4,500 queries.

**Fix:** Pre-load all tags into `HashMap<AssetId, Vec<Tag>>`. Replace `asset_has_all_tags` loop with `GROUP BY + HAVING COUNT` SQL.

---

### C-03. N+1 Query in `AlbumOrder` Sort Comparator

**File:** `crates/imglab-core/src/library/gallery.rs:154-167`

`album_item_sort_order` queries the database inside a sort comparator. Rust's sort is O(N log N), so ~2 x N x log(N) DB queries. 200 items = ~3,000 queries.

**Fix:** Load all sort orders once: `SELECT asset_id, sort_order FROM album_items WHERE album_id = ?` into a `HashMap`.

---

### C-04. N+1 Query in Manual Album Filtering

**File:** `crates/imglab-core/src/library/gallery.rs:117-120`

`items.retain` calls `asset_in_album` per item, each issuing `SELECT COUNT(*)`.

**Fix:** Load all album asset IDs into a `HashSet` once, then O(1) membership check.

---

### C-05. Synchronous `generate_image` Blocks the UI Thread

**File:** `apps/desktop/src-tauri/src/lib.rs:653-656`

`generate_image` is a sync `#[tauri::command]` that calls `execute_generation`, which for the codex-cli provider spawns a subprocess and busy-polls with `thread::sleep(100ms)` for **minutes**. The WebView's `invoke()` callback waits, freezing the UI.

Sibling commands `generate_review_field` and `regenerate_suggestion` already use `spawn_blocking` — `generate_image` should do the same.

**Fix:**

```rust
#[tauri::command]
async fn generate_image(input: GenerateImageInput) -> Result<Vec<VersionView>, CommandError> {
    tauri::async_runtime::spawn_blocking(move || execute_generation(input, None))
        .await
        .map_err(|e| CommandError { code: "GenerationFailed".into(), message: e.to_string(), recoverable: true })?
}
```

---

### C-06. Monolithic 3,571-Line React Component — Cascading Re-renders

**File:** `apps/desktop/src/main.tsx:1-3571`

All 18+ components, 40+ `useState`, 6+ `useEffect`, and every async function live in a single `App()`. Every state change (e.g., search input keystroke) re-evaluates and re-renders the entire component tree.

**Fix:**

1. Extract components to `components/` directory
2. Extract hooks: `useGallery`, `useAlbums`, `useReview`, `useQueue`, `useSettings`
3. Extract utilities to `lib/`
4. Wrap child components with `React.memo()`

---

### C-07. Gallery Image Grid — No Lazy Loading or Thumbnails

**File:** `apps/desktop/src/main.tsx:2153-2200`

Every gallery image rendered as `<img src={fullResolutionPath}>` with no `loading="lazy"`, no `decoding="async"`, no IntersectionObserver. Full-resolution images (5-20 MB decoded each) all loaded simultaneously.

**Fix:**

1. Add `loading="lazy"` and `decoding="async"` immediately
2. Generate server-side thumbnails (256x256) for grid, load full-res only on click/lightbox
3. Use IntersectionObserver for viewport-only loading

---

### C-08. Search Input Triggers Gallery Re-fetch on Every Keystroke

**File:** `apps/desktop/src/main.tsx:2002-2006, 619-623`

`query.text` updates on every `onChange`, triggering `refreshGallery()` (Tauri IPC + DB query) per character.

**Fix:** Debounce with 300-500ms delay:

```tsx
useEffect(() => {
  const timer = setTimeout(() => { void refreshGallery(); }, 300);
  return () => clearTimeout(timer);
}, [runningInTauri, library?.rootPath, query]);
```

---

## HIGH

### H-01. Missing Database Indexes on Heavily Queried Columns

**File:** `crates/imglab-core/src/library/schema.rs:1-193`

No indexes on:

- `assets(library_id)` — used in many queries
- `asset_versions(asset_id)` — used in `load_asset_versions`, `count_asset_versions`
- `generation_events(asset_id)` — used in `load_latest_generation_event`
- `metadata_suggestions(asset_id, status)` — used in `list_pending`, `pending_review_count`
- `album_items(asset_id)` — used in `load_asset_albums`

**Fix:** Add migration:

```sql
CREATE INDEX IF NOT EXISTS idx_assets_library_id ON assets(library_id);
CREATE INDEX IF NOT EXISTS idx_asset_versions_asset_id ON asset_versions(asset_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_generation_events_asset_id ON generation_events(asset_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_metadata_suggestions_asset_status ON metadata_suggestions(asset_id, status);
CREATE INDEX IF NOT EXISTS idx_album_items_asset_id ON album_items(asset_id);
```

---

### H-02. Entire File Buffered Into Memory for Hashing

**File:** `crates/imglab-core/src/library/hash.rs:18-28`

`read_to_end(&mut data)` buffers the entire file, then `sha256_bytes` clones it again via `to_vec()`. 50 MB file = ~100 MB memory.

**Fix:** Use streaming hash computation — read in 64-byte chunks, update hash incrementally. Peak memory = O(chunk_size).

---

### H-03. Entire File Read for `image_dimensions` (Only Needs Header)

**File:** `crates/imglab-core/src/library/storage.rs:57-59`

`fs::read(path)` loads the full file; `parse_image_dimensions` only needs the first ~30 bytes. 20 MB image = 20 MB wasted.

**Fix:** `File::open` + `Read::read_exact` for a 4 KB buffer, parse dimensions from that.

---

### H-04. `check_integrity` / `repair_library` — Sequential Full Re-hash

**File:** `crates/imglab-core/src/library/mod.rs:800-847, 677-798`

Every `asset_version` re-hashed sequentially (full file read + SHA-256). 10,000 versions x 5 MB avg = 50 GB processed one file at a time.

**Fix:** Use `rayon::par_iter` for parallel file I/O. Cache checksums keyed by file mtime for incremental checks.

---

### H-05. `find_library_containing_album` Opens Every Library DB

**File:** `crates/imglab-core/src/library/albums.rs:217-231`

Iterates ALL libraries, opens each DB (validates layout + migrates), queries `album_exists`. 10 libraries = 10 DB opens + 11 queries.

**Fix:** Add `library_id` column to `albums` table for direct lookup.

---

### H-06. Generation Job Polling — No Cleanup or Backoff

**File:** `apps/desktop/src/main.tsx:979-1011`

Fixed 1200ms `setTimeout` recursion with no abort on unmount, no backoff, no max timeout. Long-running jobs = wasted API calls; component unmount = state updates on unmounted component.

**Fix:** Store timeout in `useRef`, clear on unmount. Add exponential backoff (1200ms -> 2400ms -> 4800ms, cap 30s). Consider Tauri events for push-based updates.

---

### H-07. `acceptSuggestion` Fires 5 Sequential API Calls (Waterfall)

**File:** `apps/desktop/src/main.tsx:1400-1440`

3 sequential API calls (`refreshGallery` + `loadAssetDetail` + `refreshSuggestions`) triggering 5+ full re-renders.

Same pattern in: `addTagToSelectedAsset`, `addSelectedAssetToAlbum`, `deleteAlbumById`, `removeAssetFromSelectedAlbum`, `requestAssetReview`.

**Fix:** Batch independent calls with `Promise.all()`:

```tsx
await Promise.all([refreshGallery(), refreshSuggestions()]);
if (detailState.detail?.id === asset.id) {
  await loadAssetDetail(asset.id, selectedAsset?.currentVersionId ?? null);
}
```

---

### H-08. No `React.memo()` on Any Child Component

**File:** `apps/desktop/src/main.tsx:1877, 1980, 2116, 2231, 2573, 2934, 2952, 3066`

All 8 major child components (`Sidebar`, `WorkspaceToolbar`, `GalleryView`, `AlbumsView`, `ReviewInbox`, `GenerationQueue`, `SettingsView`, `Inspector`) lack memoization.

**Fix:** Wrap each with `React.memo()`. Apply to leaf components (`Thumbnail`, `StarRatingDisplay`, `MetaRow`) as well.

---

## MEDIUM

### M-01. `get_lineage` — Sequential DB Lookups Per Chain Link

**File:** `crates/imglab-core/src/library/assets.rs:245-268`

Follows `parent_version_id` one link at a time. 20-deep chain = 40 queries.

**Fix:** Use SQLite recursive CTE: `WITH RECURSIVE lineage AS (...) SELECT * FROM lineage`.

---

### M-02. `list_libraries(true)` Called Repeatedly to Resolve Library by ID

**File:** `crates/imglab-core/src/library/albums.rs:21-27, 37-43` / `library/gallery.rs:23-29`

`list_libraries(true)?.into_iter().find(...)` loads ALL libraries every time.

**Fix:** Add `get_library(library_id)` with `SELECT ... WHERE id = ?1`.

---

### M-03. `attach_tag` Issues 3 Queries Per Tag (Should Be 2)

**File:** `crates/imglab-core/src/library/metadata.rs:283-317`

INSERT + SELECT + INSERT sequence. SELECT always runs even after successful INSERT.

**Fix:** Use `INSERT ... ON CONFLICT(name) DO NOTHING RETURNING id`, fall back to SELECT only if no row returned.

---

### M-04. `add_assets_to_manual_album` — MAX Query Per Asset

**File:** `crates/imglab-core/src/library/albums.rs:505-512`

`SELECT COALESCE(MAX(sort_order), 0) + 1` per asset in a loop. 100 assets = 100 queries.

**Fix:** Query MAX once before the loop, increment counter in Rust.

---

### M-05. Daemon HTTP Server — Single-Threaded, Mutex on Entire Request

**File:** `crates/imglab-daemon/src/lib.rs:398-416`

`serve_forever_shared` handles one request at a time. `Mutex<DaemonState>` acquired for entire request handling. Long tasks block all HTTP queries (including health checks).

**Fix:** Use `RwLock` for read-write separation. Consider threaded or async HTTP server.

---

### M-06. Scheduler Clones Entire `DaemonState` Every 2 Seconds

**File:** `crates/imglab-daemon/src/lib.rs:462-474`

Deep clone of full state every 2s, even when no tasks are queued.

**Fix:** Add "has work" check before cloning. Only clone when non-terminal tasks exist.

---

### M-07. Generation Jobs HashMap Grows Without Bound

**File:** `apps/desktop/src-tauri/src/lib.rs:28-31, 676-682`

`DesktopState::generation_jobs` never removes completed/failed jobs. Long sessions = memory leak.

**Fix:** Add `expire_generation_jobs` that removes jobs older than N minutes.

---

### M-08. `list_app_logs` Scans System Temp Directory

**File:** `apps/desktop/src-tauri/src/app_logs.rs:35-37, 47-89`

Iterates every entry in `/tmp`, performs `symlink_metadata()` + `canonicalize()` on each.

**Fix:** Scope to app-specific subdirectory. Add `spawn_blocking`.

---

### M-09. No Debounce on Smart Album Preview Count

**File:** `apps/desktop/src/main.tsx:2301-2311, 2851-2896`

`assets.filter(...)` with 10+ conditions runs on every keystroke in smart builder inputs.

**Fix:** Debounce or wrap in `useMemo`.

---

### M-10. `availableProviders` Computed Inline Without `useMemo`

**File:** `apps/desktop/src/main.tsx:1781`

New array reference per render forces child re-renders.

**Fix:** Wrap in `useMemo(() => ..., [gallery])`.

---

## LOW

### L-01. `Hex` Digest Uses `format!` Per Byte

**File:** `crates/imglab-core/src/library/hash.rs:102-108`

32 allocations for SHA-256. Use hex lookup table instead.

---

### L-02. `has_task_output` Uses `COUNT(*)` Instead of `EXISTS`

**File:** `crates/imglab-core/src/library/tasks.rs:257-277`

`EXISTS` is semantically correct and can short-circuit.

---

### L-03. Cancel Mechanism Uses Filesystem Polling

**File:** `crates/imglab-provider-codex/src/lib.rs:251-258`

`cancel_path.exists()` every 100ms. Replace with `AtomicBool` or channel.

---

### L-04. 2-Second Daemon Client Timeout Too Short

**File:** `apps/desktop/src-tauri/src/daemon_client.rs:14`

May cause spurious timeouts during busy periods. Increase to 10-30s or add exponential backoff.

---

### L-05. Vite Build Config Has No Code Splitting

**File:** `apps/desktop/vite.config.ts`

Single chunk for entire app. Add `manualChunks` for vendor splitting as codebase grows.

---

## Recommended Priority

| Priority | Action | Impact |
|----------|--------|--------|
| P0 | Fix C-01 ~ C-04 (N+1 queries) + H-01 (indexes) | Gallery load: seconds -> milliseconds |
| P0 | Fix C-05 (async generate_image) | Eliminates UI freeze during generation |
| P1 | Fix C-06 + C-07 + C-08 (frontend architecture) | Eliminates cascading re-renders, reduces memory |
| P1 | Fix H-02 + H-03 (streaming hash/dimensions) | Memory: O(file_size) -> O(chunk_size) |
| P2 | Fix H-06 + H-07 (API waterfall, polling) | Reduces redundant API calls by 60%+ |
| P2 | Fix M-05 + M-06 (daemon concurrency) | Enables concurrent HTTP queries |
| P3 | Remaining Medium/Low items | Incremental improvements |
