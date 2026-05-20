## ADDED Requirements

### Requirement: Desktop controller complexity SHALL be separated by workflow ownership
System SHALL keep desktop workflow orchestration in focused controller hooks or equivalent modules, so root composition, workflow UI, transport calls, and local UI-only state remain separately maintainable.

#### Scenario: Controller hooks own async workflow orchestration
- **WHEN** desktop workflow state for libraries, gallery selection, generation composer, tasks, review, settings, logs 或 updates changes
- **THEN** async orchestration, loading state, recoverable errors 和 pending action bookkeeping SHALL live in focused controller modules or equivalent ownership boundaries
- **AND** root composition SHALL not directly contain every workflow's independent state machine

#### Scenario: Screen components avoid transport orchestration
- **WHEN** workflow screen components render Gallery, Albums, Review, Queue, Settings 或 Inspector
- **THEN** they SHALL receive stable props or controller outputs
- **AND** they SHALL avoid directly coordinating unrelated Tauri commands or cross-workflow state transitions

### Requirement: Gallery filtering logic SHALL use a shared predicate pipeline
System SHALL keep gallery query filtering and smart album preview filtering on a shared predicate pipeline for overlapping semantics.

#### Scenario: Gallery query and smart album share common filters
- **WHEN** gallery query and smart album query both filter by text, tags, provider, rating, review pending state, category, created time 或 sort order
- **THEN** implementation SHALL use shared helper logic for those overlapping predicates
- **AND** differences between gallery query and smart album query SHALL be explicit in conversion or adapter code

#### Scenario: Filtering refactor preserves result semantics
- **WHEN** the shared filtering pipeline replaces duplicated query paths
- **THEN** existing gallery search, album-scoped gallery, and smart album preview results SHALL remain behaviorally equivalent

### Requirement: Refactor verification SHALL include complexity-oriented checks
System SHALL verify maintainability refactors with both behavior checks and structure checks for large-file regression.

#### Scenario: Large-file regression is checked
- **WHEN** a refactor wave claims to reduce single-file complexity
- **THEN** verification SHALL include a source file size or ownership scan that identifies remaining mega files and explains deferred cases

#### Scenario: Behavior checks still pass
- **WHEN** code is moved across modules without intended behavior changes
- **THEN** affected TypeScript builds, Rust formatting/checks, and focused tests SHALL pass or any failure SHALL be documented as a blocker

## MODIFIED Requirements

### Requirement: Large files SHALL be split by responsibility boundaries
System SHALL split oversized implementation files by clear runtime or domain responsibilities rather than by arbitrary line count. Physical splitting SHALL create real ownership boundaries; it MUST NOT rely on file names alone when all code is still injected into one module scope.

#### Scenario: Core library service is split
- **WHEN** `crates/imglab-core/src/library/mod.rs` is refactored
- **THEN** registry, service lifecycle, generation, diagnostics, maintenance, and domain modules must have separate ownership
- **AND** `mod.rs` must no longer contain the bulk of implementation and tests

#### Scenario: Desktop frontend entry is split
- **WHEN** `apps/desktop/src/app/App.tsx` is refactored
- **THEN** workflow components, workflow hooks, mock data, Tauri transport helpers, and formatting utilities must be separated
- **AND** application bootstrap must remain small enough to show composition rather than workflow internals

#### Scenario: Tauri backend entry is split
- **WHEN** `apps/desktop/src-tauri/src/lib.rs` is refactored
- **THEN** commands, views, errors, paths, updater, and daemon sidecar behavior must be separated into real Rust modules
- **AND** command handlers must avoid embedding unrelated view mapping or path helper logic
- **AND** the refactor must not keep those implementation files wired primarily through root-level `include!`

#### Scenario: Daemon entry is split
- **WHEN** `crates/imglab-daemon/src/lib.rs` is refactored
- **THEN** runtime, transport, routing, scheduler, executors, logs, and views must be separated into real Rust modules
- **AND** daemon API shape must remain compatible
- **AND** the refactor must not keep those implementation files wired primarily through root-level `include!`
