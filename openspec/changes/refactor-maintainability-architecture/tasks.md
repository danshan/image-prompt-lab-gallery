# Tasks

## 1. Core Boundary Refactor

- [x] 1.1 Move registry lifecycle and registry commands out of `library/mod.rs`.
- [x] 1.2 Move library creation/open/layout/manifest helpers into a focused service module.
- [x] 1.3 Move repair and integrity orchestration into a maintenance module.
- [x] 1.4 Move studio diagnostics and provider health summary construction into a diagnostics module.
- [x] 1.5 Move generation service and metadata suggestion helpers into a generation module.
- [x] 1.6 Move core tests to the modules that own the behavior.
- [x] 1.7 Run `cargo fmt --all --check` and `cargo test --offline -p imglab-core`.

## 2. Generation Planning Refactor

- [x] 2.1 Define a shared generation planning boundary in core.
- [x] 2.2 Update CLI generation to use the shared planner.
- [x] 2.3 Update Tauri generation commands to use the shared planner.
- [x] 2.4 Update daemon image generation task execution to use the shared planner where applicable.
- [x] 2.5 Remove duplicate provider normalization, default model, and operation parsing helpers from transport layers when no longer needed.
- [x] 2.6 Run `cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli -p imglab-daemon`.

## 3. Daemon Boundary Refactor

- [x] 3.1 Move runtime state, runtime file, token file, and listener helpers into `runtime.rs`.
- [x] 3.2 Move raw HTTP parsing, auth, and response serialization into `transport.rs`.
- [x] 3.3 Move route dispatch and request parsing into `routes.rs`.
- [x] 3.4 Move scheduler loop, runnable checks, recovery, and tick orchestration into `scheduler.rs`.
- [x] 3.5 Move image generation and metadata task execution into executor modules.
- [x] 3.6 Move daemon response DTOs and conversions into `views.rs`.
- [x] 3.7 Move daemon tests to the relevant modules.
- [x] 3.8 Run `cargo test --offline -p imglab-daemon`.

## 4. Tauri Backend Boundary Refactor

- [x] 4.1 Extract command error mapping into `errors.rs`.
- [x] 4.2 Extract path normalization, registry path, runtime path, and reveal helpers into `paths.rs`.
- [x] 4.3 Extract serializable views and conversion functions into `views.rs`.
- [x] 4.4 Split commands by workflow under `commands/`.
- [x] 4.5 Keep `lib.rs` responsible for plugin setup, managed state, command registration, and app startup only.
- [x] 4.6 Run `cargo check --offline -p imglab-desktop` when local dependencies are available.

## 5. Desktop Frontend Boundary Refactor

- [x] 5.1 Move application bootstrap/composition from `main.tsx` into `app/App.tsx`.
- [x] 5.2 Move command invocation and runtime detection into a Tauri adapter module.
- [x] 5.3 Move mock preview data into a dedicated module.
- [x] 5.4 Extract workflow hooks for libraries, gallery, albums, review, tasks, settings, and updates.
- [x] 5.5 Extract workflow components into directories by screen.
- [x] 5.6 Move formatting, image display, and path helpers into focused utility modules.
- [x] 5.7 Keep pure state helpers tested and add tests for any newly extracted pure logic.
- [x] 5.8 Run `cd apps/desktop && npm run test && npm run build`.

## 6. Final Verification And Documentation

- [x] 6.1 Run the full available Rust and frontend validation set.
- [x] 6.2 Update development documentation with the new module map.
- [x] 6.3 Record any commands that could not run and why.
- [x] 6.4 Verify OpenSpec deltas and archive only after implementation is complete.

Verification note: all planned Rust, frontend, and OpenSpec validation commands ran successfully. OpenSpec emitted `edge.openspec.dev` PostHog flush network errors after local validation succeeded; no validation command was blocked by this telemetry failure.
