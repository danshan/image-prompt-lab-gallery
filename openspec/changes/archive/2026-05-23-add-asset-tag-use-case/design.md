# Design: Add Asset Tag Use Case

## Overview

This change adds a focused asset tag mutation path to the application layer. The use case delegates persistence to `AssetRepository`, preserving the current SQLite implementation and compatibility behavior.

## Boundaries

- CLI owns argument parsing, dry-run output, and JSON formatting.
- `AssetUseCase` owns the application entrypoint for tag mutation.
- `AssetRepository` owns the persistence port method.
- `LocalLibraryService` remains the SQLite adapter and compatibility surface.

## Compatibility

The change must preserve:

- CLI `tag add --library <path> <asset-id> <tag>` JSON shape.
- CLI dry-run output.
- Existing tag deduplication and `asset_tags` upsert behavior.
- Existing library schema and migration behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `cargo test -p imglab-cli`
- `scripts/check-architecture.sh`
- `openspec validate add-asset-tag-use-case --strict`
