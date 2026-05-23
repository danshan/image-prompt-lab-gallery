# Design: Route Tauri Albums Through Use Case

## Overview

This change extends the album application boundary with path-scoped list and manual-create methods. Desktop commands already hold a selected resource library path, so the use case forwards that shape to the repository port while preserving existing behavior.

## Boundaries

- Tauri commands own input mapping, application invocation, view mapping, and error mapping.
- `AlbumUseCase` owns album workflow entrypoints for runtime adapters.
- `AlbumRepository` owns persistence-facing album operations.
- `LocalLibraryService` remains the SQLite adapter behind the repository port.

## Compatibility

The change must preserve:

- `list_albums` Tauri command output.
- `create_manual_album` Tauri command output.
- Existing album storage, ordering, and smart/manual album semantics.
- Existing SQLite schema and migrations.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-desktop`
- `scripts/check-architecture.sh`
- `openspec validate route-tauri-albums-through-use-case --strict`
