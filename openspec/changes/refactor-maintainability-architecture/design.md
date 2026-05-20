# Design: Maintainability Architecture Refactor

## Context

The current architecture is directionally correct: Rust core owns business behavior, Tauri exposes desktop commands, React renders the GUI, daemon runs long tasks, and providers are isolated behind adapter crates. The problem is not the top-level architecture. The problem is that several implementation files have become integration surfaces for too many concerns.

This refactor should preserve the top-level architecture and rebuild internal boundaries. The guiding rule is: split by responsibility and runtime boundary, not by line count alone.

## Goals

- Make core service boundaries easier to reason about and test.
- Make desktop frontend workflows independently editable.
- Make Tauri command code a transport layer instead of an orchestration sink.
- Make daemon scheduler, HTTP routing, and task execution independently understandable.
- Remove repeated generation/provider planning logic across CLI, desktop, and daemon.
- Keep public behavior stable while improving maintainability.

## Non-Goals

- Do not introduce broad abstractions that are not serving a current boundary.
- Do not rewrite SQL access behind generic repositories.
- Do not redesign UI visuals.
- Do not replace the daemon protocol.
- Do not change persisted library layout or schema unless a specific task requires it.

## Recommended Approach

Use staged boundary refactoring:

1. Core boundaries.
2. Generation/provider planning.
3. Daemon boundaries.
4. Tauri backend boundaries.
5. Desktop frontend boundaries.
6. Type/view mapping cleanup.

This order keeps the business source of truth stable before UI and transport code are moved around. It also reduces duplicate logic before command and daemon modules are split.

## Core Boundary

`crates/imglab-core/src/library/mod.rs` should become a small module hub. The target shape is:

```text
crates/imglab-core/src/library/
  mod.rs
  registry.rs
  service.rs
  generation.rs
  maintenance.rs
  diagnostics.rs
  schema.rs
  storage.rs
  assets.rs
  gallery.rs
  metadata.rs
  albums.rs
  backup.rs
  export.rs
  repair.rs
  tasks.rs
```

Responsibilities:

- `registry.rs`: registry database lifecycle, upsert, list, hide, alias, unregister, registry counts.
- `service.rs`: `LocalLibraryService` construction, library creation/opening, layout validation, manifest helpers.
- `generation.rs`: `LocalGenerationService`, generation metadata suggestion creation, shared generation request planning.
- `maintenance.rs`: repair and integrity orchestration.
- `diagnostics.rs`: studio overview, diagnostics overview, provider health summaries.
- Existing focused modules keep their current domain responsibilities.

Prefer private functions receiving `&Connection` where a narrower helper is enough. Do not add repository traits just to enable splitting.

## Generation Boundary

CLI, Tauri backend, and daemon currently make overlapping decisions about:

- Provider name normalization.
- Default model labels.
- Operation parsing and formatting.
- Text-to-image vs image-to-image inference.
- Input file and input version loading.
- Provider dispatch.

Move planning into core as a small explicit boundary. A good target is a `GenerationPlan` or extended `PreparedGenerationRequest` that contains:

- Normalized provider id.
- Operation.
- Model label.
- Prepared `GenerateImageRequest`.
- Optional execution metadata needed by callers, such as selected provider id.

Provider execution stays outside this planner. Provider crates still own command construction, authentication assumptions, output parsing, and provider-specific validation.

## Daemon Boundary

`crates/imglab-daemon/src/lib.rs` should be split into:

```text
crates/imglab-daemon/src/
  lib.rs
  runtime.rs
  transport.rs
  routes.rs
  scheduler.rs
  executors/
    mod.rs
    image_generation.rs
    metadata.rs
  views.rs
  logs.rs
```

Responsibilities:

- `runtime.rs`: `DaemonState`, runtime file, token file, listener binding, side-effect setup.
- `transport.rs`: raw HTTP parsing, response serialization, auth extraction.
- `routes.rs`: route dispatch and request body parsing.
- `scheduler.rs`: scheduler loop, runnable-work checks, recovery, tick orchestration.
- `executors/*`: task execution bodies and output recording.
- `views.rs`: daemon response DTOs and conversions.
- `logs.rs`: task log writing, cancellation markers, log tail safety checks.

The first pass should keep the current hand-rolled loopback HTTP transport. Splitting must not change API shape.

## Tauri Backend Boundary

`apps/desktop/src-tauri/src/lib.rs` should be reduced to setup and command registration. Target layout:

```text
apps/desktop/src-tauri/src/
  lib.rs
  errors.rs
  paths.rs
  views.rs
  commands/
    mod.rs
    libraries.rs
    gallery.rs
    generation.rs
    daemon.rs
    albums.rs
    metadata.rs
    logs.rs
    updater.rs
  daemon_client.rs
  app_logs.rs
  metadata_generation.rs
```

Command files should do only:

- Deserialize command inputs.
- Call core or daemon client.
- Convert domain output to serializable views.
- Map errors to `CommandError`.

Generation execution should use the shared core planner. View conversion should move to `views.rs`, not stay interleaved with commands.

## Desktop Frontend Boundary

`apps/desktop/src/main.tsx` should become composition and bootstrap. Target layout:

```text
apps/desktop/src/
  main.tsx
  app/
    App.tsx
    types.ts
    tauri.ts
    mock-data.ts
  hooks/
    useLibraries.ts
    useGallery.ts
    useAlbums.ts
    useReview.ts
    useTasks.ts
    useSettings.ts
    useUpdates.ts
  components/
    gallery/
    albums/
    review/
    queue/
    settings/
    inspector/
    shell/
  lib/
    paths.ts
    formatting.ts
    image-display.ts
```

Workflow hooks should own IPC calls and state invalidation for one workflow. Components should receive view state and callbacks, not reach directly into unrelated workflow state. Existing pure helpers in `workbench-state.ts`, `studio-orchestration.ts`, and `studio-data-hooks.ts` can be kept and gradually relocated as boundaries stabilize.

The first frontend pass should not add a global state library. Local React state and focused hooks remain enough if invalidation is explicit.

## DTO And View Mapping

Short term:

- Move Rust view conversion functions into dedicated `views.rs` modules.
- Keep transport-specific view structs where needed.
- Ensure view mapping functions contain no business decisions.

Medium term:

- Evaluate deriving `Serialize` for stable core view DTOs.
- Consider TypeScript type generation only after Rust transport view boundaries are stable.

Do not start by adding type generation, because that would couple the refactor to a tooling migration.

## Testing Strategy

Each wave must preserve behavior with targeted checks:

- Core: `cargo test --offline -p imglab-core`.
- Provider/CLI integration: `cargo test --offline -p imglab-provider-codex -p imglab-cli`.
- Daemon: `cargo test --offline -p imglab-daemon`.
- Tauri backend: `cargo check --offline -p imglab-desktop` when local dependencies are available.
- Frontend: `cd apps/desktop && npm run test && npm run build`.

Tests should move with code when modules split. Do not leave all migrated module tests in legacy entry files.

## Risks

- Large mechanical moves can hide behavior changes. Mitigation: split one boundary at a time and run checks after each wave.
- Reorganizing frontend state can cause subtle stale selection or refresh behavior. Mitigation: keep pure state helpers tested and move IPC hooks incrementally.
- Provider planning could absorb provider-specific behavior. Mitigation: planner only builds normalized requests; provider crates still execute.
- Tauri command registration can drift during command splitting. Mitigation: command groups should expose explicit registration lists and tests or compile checks should cover all commands.

