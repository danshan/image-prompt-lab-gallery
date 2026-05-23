# Proposal: Add Asset Tag Use Case

## Problem

The DDD boundary inventory still documents CLI tag mutation as a compatibility path that calls `LocalLibraryService` directly. This leaves a migrated runtime command using the legacy concrete service as its primary business entrypoint.

Tag mutation belongs with asset metadata behavior. Runtime adapters should parse CLI input and print JSON, while the application owner handles the mutation through a repository port.

## Goals

- Add an application use-case entrypoint for adding a tag to an asset.
- Route CLI `tag add` through the application asset owner instead of direct `LocalLibraryService`.
- Preserve existing CLI JSON output, dry-run behavior, SQLite schema, and tag persistence semantics.
- Update the architecture inventory so CLI tag is no longer a direct legacy-service exception.

## Non-Goals

- Do not redesign tag taxonomy, colors, or metadata review behavior.
- Do not change SQLite schema or migration behavior.
- Do not remove existing `LocalLibraryService::add_tag_to_asset` compatibility method used by legacy tests.

## Impact

- `AssetUseCase` gains a tag mutation method backed by `AssetRepository`.
- The SQLite adapter implements the repository method using existing tag persistence behavior.
- CLI no longer needs the generic legacy service for `tag add`.
