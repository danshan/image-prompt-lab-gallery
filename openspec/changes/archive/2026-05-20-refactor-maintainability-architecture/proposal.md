# Refactor Maintainability Architecture

## Why

The project has reached a point where product iteration is constrained more by code boundaries than by missing features. Several critical files now combine transport, orchestration, persistence, view mapping, UI state, mock preview behavior, and tests:

- `apps/desktop/src/main.tsx`
- `apps/desktop/src-tauri/src/lib.rs`
- `crates/imglab-core/src/library/mod.rs`
- `crates/imglab-daemon/src/lib.rs`

Earlier cleanup work fixed important correctness and performance issues, but subsequent feature growth has re-concentrated responsibilities in large entry files. The next refactor should rebuild stable module boundaries while preserving visible behavior, so future gallery, generation, review, daemon, settings, and release work can evolve without touching unrelated workflows.

## What Changes

- Split desktop React code by workflow boundaries, with focused hooks, components, transport helpers, and pure state utilities.
- Split Tauri backend code into command groups, serializable views, daemon sidecar management, updater, path handling, and shared command error mapping.
- Reduce `imglab-core` `library/mod.rs` to module wiring and public entry points, moving registry, service implementations, generation orchestration, maintenance, diagnostics, and tests into focused modules.
- Split daemon code into transport parsing, routing, scheduler, task executors, runtime state, views, and tests.
- Consolidate generation/provider request planning so CLI, Tauri, and daemon do not duplicate provider normalization, operation inference, default model selection, or input loading.
- Centralize DTO/view mapping and identify a path toward generated TypeScript types without requiring that as the first implementation step.
- Keep behavior stable unless a delta spec explicitly says otherwise.

## Non-Goals

- No visual redesign.
- No new product workflow.
- No database engine replacement.
- No new daemon transport framework unless the current transport becomes a blocker.
- No broad repository trait layer without a concrete test or runtime boundary.
- No native OpenAI or Grok provider implementation.
- No managed library layout change.

## Impact

- Core behavior should remain compatible with existing CLI, desktop, and daemon workflows.
- Refactor diffs will be large, so implementation must be staged and verified after each wave.
- Tests should move with their modules instead of remaining concentrated in legacy large files.
- Future changes should have smaller blast radius because workflow-specific code will have clearer ownership.

