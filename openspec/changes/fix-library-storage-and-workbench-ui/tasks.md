## 1. Core Storage And Metadata

- [x] 1.1 Add structured checksum fields to the core DTO and persistence path, including checksum algorithm and digest for asset versions.
- [x] 1.2 Implement MD5 digest calculation and keep legacy SHA-256 data readable during migration or compatibility reads.
- [x] 1.3 Add or update SQLite migration logic so new asset versions record `MD5` plus the digest without breaking existing libraries.
- [x] 1.4 Replace new managed image file paths with `originals/$year/$month/$uuid.$extension` for import and generation flows.
- [x] 1.5 Update integrity check to calculate digest using each version's recorded checksum algorithm and report algorithm-aware mismatch messages.
- [x] 1.6 Add library storage size calculation for managed directories without capacity limit semantics.
- [x] 1.7 Parse image dimensions for new imported and generated asset versions and persist width and height when available.
- [x] 1.8 Add explicit library repair service for historical path, checksum, and dimension normalization with dry-run support.

## 2. Core Read Models And Commands

- [x] 2.1 Extend `FileContextView` and Tauri file DTOs to expose checksum algorithm, checksum digest, size, width and height consistently.
- [x] 2.2 Expose Library Status data needed by the desktop UI, including actual storage size and integrity status without a storage upper bound.
- [x] 2.3 Ensure Gallery or selected asset read models provide real resolution data when available and nullable values when unavailable.
- [x] 2.4 Update CLI and Tauri command mappings affected by changed checksum and file context DTOs.
- [x] 2.5 Expose repair summary through CLI and Tauri command mappings.

## 3. Desktop Shell And Library Switching

- [x] 3.1 Set the Tauri main window to use app-provided chrome instead of the system default titlebar.
- [x] 3.2 Add an app top bar with drag region, app title and window controls.
- [x] 3.3 Load visible registered libraries on startup and allow switching current Library from the Sidebar selector.
- [x] 3.4 On Library switch, reload Gallery for the selected Library, clear stale Inspector selection and keep recoverable errors scoped to the active Library.

## 4. Workbench UI Fixes

- [x] 4.1 Refactor Sidebar layout so it does not create an internal scrollbar at supported window sizes.
- [x] 4.2 Change Sidebar Library Status storage display to show only actual Library size and remove fixed capacity meter semantics.
- [x] 4.3 Introduce a shared selector style or selector component and apply it to filter, sort, provider and Library selector controls.
- [x] 4.4 Fix resolution rendering to show `$width x $height` only when both dimensions are available, otherwise show an unavailable state.
- [x] 4.5 Render checksum in the Inspector File section as `MD5: $hash` when the file context algorithm is `MD5`.
- [x] 4.6 Update mock data and demo-mode state so it matches the new Library switcher, storage, resolution and checksum contracts.

## 5. Verification

- [x] 5.1 Add core tests for date-bucketed UUID file paths, duplicate source filenames, MD5 digest recording and integrity mismatch detection.
- [x] 5.2 Add migration or compatibility tests covering old asset version checksum data and new MD5 asset version data.
- [x] 5.3 Add frontend state tests for Library switching, stale selection clearing and selector-driven query preservation.
- [x] 5.4 Add core tests covering PNG and generated-image dimension persistence, plus unavailable dimensions for unknown binary files.
- [x] 5.5 Add repair tests covering dry run, legacy path moves, checksum repair, dimension backfill, missing file issues, and CLI output.
- [x] 5.6 Run Rust unit tests for `imglab-core` and CLI tests affected by DTO changes.
- [x] 5.7 Run desktop build or typecheck and verify no TypeScript regressions.
- [ ] 5.8 Manually verify the Tauri window shows only one set of window controls and supports drag, minimize, maximize or restore, and close.
