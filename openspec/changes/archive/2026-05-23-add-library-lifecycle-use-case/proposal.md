# Proposal: Add Library Lifecycle Use Case

## Problem

The systematic DDD review identified resource library lifecycle as the largest remaining runtime-visible compatibility boundary. CLI, daemon, and Tauri code still call `LocalLibraryService` directly for create, open, list, repair, export, backup, registry alias, unregister, status, integrity, and overview workflows.

This keeps a legacy infrastructure service visible as a primary-looking runtime boundary. It also blocks later work to narrow daemon `service()` access and make runtime adapters consistently call application owners.

## Goals

- Introduce a focused application owner for resource library lifecycle behavior.
- Preserve existing CLI JSON, daemon behavior, Tauri payloads, SQLite schema, manifest format, registry semantics, and managed file layout.
- Move runtime lifecycle calls toward the application facade.
- Keep `LocalLibraryService` as an infrastructure/compatibility adapter during this wave.
- Document the remaining compatibility surface explicitly.

## Non-Goals

- Do not change resource library schema version or manifest format.
- Do not redesign backup/restore, repair, or registry semantics.
- Do not remove `LocalLibraryService` completely.
- Do not migrate tag mutation or gallery read-model internals in this change.
- Do not introduce another database or sidecar index.

## Impact

- `imglab-core` gains a `LibraryUseCase` that depends on a `LibraryRepository` port.
- Composition exposes both the transitional compatibility service and the new lifecycle owner while runtime callers migrate.
- CLI, daemon, and Tauri library lifecycle paths call the application owner where possible.
- Architecture inventory and OpenSpec specs are updated to reflect the new owner and remaining gaps.
