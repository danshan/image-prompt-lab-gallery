# Proposal: Route Tauri Tag Through Use Case

## Problem

The Tauri metadata command module still calls `desktop_app().library().add_tag_to_asset(...)` directly. CLI tag mutation already uses `AssetUseCase`, so Tauri now has a remaining compatibility-service entrypoint for the same behavior.

This keeps runtime ownership inconsistent and leaves a direct legacy service call in a user-facing metadata mutation path.

## Goals

- Route Tauri `add_tag_to_asset` through `AssetUseCase`.
- Preserve Tauri command signature, error mapping, and existing tag persistence semantics.
- Keep SQLite schema and library compatibility unchanged.
- Update the architecture inventory and specs to remove the remaining Tauri tag compatibility note.

## Non-Goals

- Do not change tag taxonomy, colors, or metadata review workflow.
- Do not change SQLite schema or migration behavior.
- Do not remove the existing compatibility method used by legacy tests.

## Impact

- Tauri tag mutation uses the same asset application owner as CLI tag mutation.
- Runtime adapters no longer call the concrete library service for tag mutation.
- Remaining runtime cleanup can focus on type import narrowing and gallery read-model decomposition.
