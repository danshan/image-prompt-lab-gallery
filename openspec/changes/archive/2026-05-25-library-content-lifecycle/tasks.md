## 1. Core archive lifecycle

- [x] 1.1 Add additive asset archive schema migration and DTO/view structs for archived content summaries.
- [x] 1.2 Implement core repository operations to archive and restore assets using persistent archive metadata.
- [x] 1.3 Update Gallery, Albums, smart album, manual album, and add-to-album query paths to exclude archived assets by default.
- [x] 1.4 Implement prompt document archive and restore operations through core repository methods, preserving prompt versions.
- [x] 1.5 Add core tests proving asset archive/restore visibility and prompt archive/restore visibility.

## 2. Archived content and permanent delete

- [x] 2.1 Implement archived content read model for archived assets and prompt documents.
- [x] 2.2 Implement permanent delete dry-run dependency planner for archived assets and prompt documents.
- [x] 2.3 Implement permanent delete apply for archived assets, including cascading SQLite facts and managed file deletion issue reporting.
- [x] 2.4 Implement permanent delete apply for archived prompt documents, including prompt versions and related history references.
- [x] 2.5 Add core tests for dry-run immutability, active item rejection, cascade deletion, and managed file deletion failure reporting.

## 3. Library merge import

- [x] 3.1 Implement merge library dry-run validation for source layout, manifest, schema compatibility, counts, file size, and skipped runtime state.
- [x] 3.2 Implement merge planning with source-to-target ID maps for assets, versions, tags, albums, prompts, prompt versions, generation events, metadata suggestions, and lineage references.
- [x] 3.3 Implement merge apply that copies managed files to target standard paths and writes target SQLite facts with rewritten references.
- [x] 3.4 Ensure merge apply leaves source library unchanged and preserves existing Import Zip restore-as-separate-library semantics.
- [x] 3.5 Add core tests for invalid source rejection, unsupported schema rejection, ID remapping, reference rewriting, skipped runtime state, and source unchanged behavior.

## 4. Tauri command surface

- [x] 4.1 Add Tauri inputs, outputs, commands, and view mappers for asset archive/restore and prompt archive/restore.
- [x] 4.2 Add Tauri inputs, outputs, commands, and view mappers for archived content list and permanent delete dry-run/apply.
- [x] 4.3 Add Tauri inputs, outputs, commands, and view mappers for merge library dry-run/apply.
- [x] 4.4 Register all new commands in the desktop Tauri command handler.
- [x] 4.5 Add desktop backend tests for command mapping and production command outputs.

## 5. Desktop frontend

- [x] 5.1 Add Gallery single asset archive and batch archive actions wired to real Tauri commands in production.
- [x] 5.2 Update Gallery masonry CSS and thumbnail rendering so cards keep stable width, images preserve ratio, and long images do not overflow.
- [x] 5.3 Wire Albums detail thumbnails to existing lightbox behavior without changing manual album remove semantics.
- [x] 5.4 Add Prompt Library archive action wired to real Tauri command in production and update selected prompt behavior after archive.
- [x] 5.5 Add Settings Archived section with assets/prompts tabs, restore action, permanent delete dry-run summary, confirmation, and apply action.
- [x] 5.6 Add Settings / Libraries Merge Library action with source folder picker, dry-run preview, confirmation, apply, and workspace refresh.
- [x] 5.7 Add frontend tests for Gallery archive, Albums lightbox, Prompt archive, Settings restore/delete flow, and Merge Library command wiring.

## 6. Verification and OpenSpec closeout readiness

- [x] 6.1 Run `openspec validate library-content-lifecycle --strict`.
- [x] 6.2 Run `openspec validate --specs --strict`.
- [x] 6.3 Run `cargo fmt --all --check`.
- [x] 6.4 Run `cargo test -p imglab-core`.
- [x] 6.5 Run `cargo test -p imglab-desktop --lib`.
- [x] 6.6 Run `npm test --prefix apps/desktop`.
- [x] 6.7 Run `npm run build --prefix apps/desktop`.
- [x] 6.8 Run `scripts/check-architecture.sh`.
- [x] 6.9 Run `git diff --check` and inspect `git status --short`.
