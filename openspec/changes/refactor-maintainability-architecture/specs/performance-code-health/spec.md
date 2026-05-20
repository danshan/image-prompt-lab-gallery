## ADDED Requirements

### Requirement: Maintainability refactor SHALL preserve visible behavior
System SHALL allow large architecture refactors only when public CLI output, desktop workflows, daemon API responses, and persisted resource library behavior remain stable unless a spec explicitly defines a behavior change.

#### Scenario: Refactor wave preserves behavior
- **WHEN** a refactor wave moves code between modules
- **THEN** existing behavior tests for the affected crate or app must continue to pass
- **AND** the implementation must not require users to migrate resource libraries solely because files were reorganized

### Requirement: Large files SHALL be split by responsibility boundaries
System SHALL split oversized implementation files by clear runtime or domain responsibilities rather than by arbitrary line count.

#### Scenario: Core library service is split
- **WHEN** `crates/imglab-core/src/library/mod.rs` is refactored
- **THEN** registry, service lifecycle, generation, diagnostics, maintenance, and domain modules must have separate ownership
- **AND** `mod.rs` must no longer contain the bulk of implementation and tests

#### Scenario: Desktop frontend entry is split
- **WHEN** `apps/desktop/src/main.tsx` is refactored
- **THEN** workflow components, workflow hooks, mock data, Tauri transport helpers, and formatting utilities must be separated
- **AND** application bootstrap must remain small enough to show composition rather than workflow internals

#### Scenario: Tauri backend entry is split
- **WHEN** `apps/desktop/src-tauri/src/lib.rs` is refactored
- **THEN** commands, views, errors, paths, updater, and daemon sidecar behavior must be separated
- **AND** command handlers must avoid embedding unrelated view mapping or path helper logic

#### Scenario: Daemon entry is split
- **WHEN** `crates/imglab-daemon/src/lib.rs` is refactored
- **THEN** runtime, transport, routing, scheduler, executors, logs, and views must be separated
- **AND** daemon API shape must remain compatible

### Requirement: Tests SHALL follow refactored ownership
System SHALL keep tests near the modules that own the behavior after a boundary refactor.

#### Scenario: Module tests move with behavior
- **WHEN** behavior is moved from a legacy large file into a focused module
- **THEN** directly related unit tests must move with that module or be replaced by an equivalent integration test
- **AND** legacy entry files must not remain the primary home for unrelated module tests

