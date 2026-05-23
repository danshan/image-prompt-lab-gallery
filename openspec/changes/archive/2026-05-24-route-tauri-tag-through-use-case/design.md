# Design: Route Tauri Tag Through Use Case

## Overview

This change updates the Tauri metadata command for tag addition to use `desktop_app().assets().add_tag(...)`. The use case and repository port already exist from the asset tag boundary change, so this is a runtime adapter migration.

## Boundaries

- Tauri command owns input mapping and error mapping.
- `AssetUseCase` owns the asset tag mutation application entrypoint.
- `AssetRepository` owns the persistence port.
- `LocalLibraryService` remains the SQLite adapter and compatibility surface.

## Compatibility

The change must preserve:

- Tauri `add_tag_to_asset` command input and return shape.
- Existing tag deduplication and asset-tag upsert behavior.
- Existing SQLite schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-desktop`
- `scripts/check-architecture.sh`
- `openspec validate route-tauri-tag-through-use-case --strict`
