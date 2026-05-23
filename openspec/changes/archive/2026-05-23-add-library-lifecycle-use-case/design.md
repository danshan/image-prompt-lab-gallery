# Design: Add Library Lifecycle Use Case

## Overview

This change creates an application-layer owner for resource library lifecycle workflows without changing persistence behavior. The implementation wraps the existing SQLite-backed `LocalLibraryService` behind the existing `LibraryRepository` port and routes runtime adapters through the new owner.

## Boundaries

- `LibraryUseCase` owns application entrypoints for create, open, list, alias rename, unregister, export, backup import/export, repair, integrity, status, and overview.
- `LocalLibraryService` remains the SQLite/filesystem/registry compatibility adapter.
- Runtime adapters parse input, call `app.library_lifecycle()`, and map output.
- Runtime adapters may still call `app.library()` only for explicitly documented gaps such as tag mutation or compatibility-only paths not yet covered by focused application owners.

## Compatibility

The change must preserve:

- CLI JSON output and dry-run behavior.
- Daemon loopback open-library and recovery behavior.
- Tauri command payloads.
- `manifest.json`, `library.sqlite`, registry, backup zip, and managed file layout.

## Migration Strategy

1. Add `LibraryUseCase<R>` in `application::use_cases::library`.
2. Extend the application facade with a `library_lifecycle()` owner while keeping `library()` as the legacy compatibility surface.
3. Wire `library_lifecycle` in SQLite composition.
4. Move CLI, daemon, and Tauri lifecycle calls to `library_lifecycle()`.
5. Update architecture docs, tasks, and specs.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `cargo test -p imglab-cli`
- `cargo test -p imglab-daemon`
- `cargo test -p imglab-desktop`
- `scripts/check-architecture.sh`
- `openspec validate add-library-lifecycle-use-case --strict`
