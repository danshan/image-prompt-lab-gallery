# performance-code-health Specification

## Purpose
TBD - created by archiving change refactor-performance-code-health. Update Purpose after archive.
## Requirements
### Requirement: 维护性能 Review 清理基线
系统 SHALL 将性能和代码健康 review 发现转化为可追踪的实现任务, 并在完成后记录验证结果和剩余风险.

#### Scenario: Review 发现进入实现任务
- **WHEN** `docs/PERFORMANCE_REVIEW.md` 中的发现被确认仍适用于当前代码
- **THEN** 对应发现必须映射到 OpenSpec tasks 或明确记录为 deferred 且说明原因

#### Scenario: 完成 Cleanup 后记录验证结果
- **WHEN** cleanup 实现完成
- **THEN** 系统文档必须记录已运行的验证命令, 失败原因或剩余风险

### Requirement: 建立 SQLite Sufficiency Checkpoint
系统 SHALL 在修复查询形态和索引后评估 SQLite 是否仍适合作为 resource library 主事实存储.

#### Scenario: SQLite 优化后满足目标
- **WHEN** Gallery/Search 在目标图库规模下满足可接受响应时间且无明显 lock contention
- **THEN** 系统继续使用 SQLite 作为主事实存储, 并记录后续触发 supplemental index 的条件

#### Scenario: SQLite 优化后仍不满足目标
- **WHEN** Gallery/Search 或并发写入在目标图库规模下仍不满足响应性要求
- **THEN** 系统必须提出单独 OpenSpec change, 评估 FTS5, projection table, Tantivy, DuckDB 或 PostgreSQL 等补充/替换方案

### Requirement: Cleanup 保持公共行为稳定
系统 MUST 在性能 cleanup 中保持既有 public service, desktop workflow 和 daemon API 的可见行为稳定, 除非 spec 明确声明行为变化.

#### Scenario: Refactor 不改变 Gallery 语义
- **WHEN** core read path 从 per-asset 查询重构为 batch read model
- **THEN** Gallery filters, sort 语义和返回字段必须与现有 spec 保持一致

#### Scenario: Refactor 不改变 Daemon API Contract
- **WHEN** daemon request handling 和 scheduler 内部实现被优化
- **THEN** loopback endpoint, token authentication 和 task response shape 必须保持兼容

### Requirement: Maintainability refactor SHALL preserve visible behavior
System SHALL allow large architecture refactors only when public CLI output, desktop workflows, daemon API responses, and persisted resource library behavior remain stable unless a spec explicitly defines a behavior change.

#### Scenario: Refactor wave preserves behavior
- **WHEN** a refactor wave moves code between modules
- **THEN** existing behavior tests for the affected crate or app must continue to pass
- **AND** the implementation must not require users to migrate resource libraries solely because files were reorganized

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

### Requirement: Tests SHALL follow refactored ownership
System SHALL keep tests near the modules that own the behavior after a boundary refactor.

#### Scenario: Module tests move with behavior
- **WHEN** behavior is moved from a legacy large file into a focused module
- **THEN** directly related unit tests must move with that module or be replaced by an equivalent integration test
- **AND** legacy entry files must not remain the primary home for unrelated module tests

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

### Requirement: DDD Boundary Refactor SHALL complete in one change

System SHALL complete the DDD boundary refactor in this change, covering core domain/application/infrastructure boundaries, runtime integration updates, frontend workflow ownership cleanup, tests, docs, and validation. The implementation MUST NOT leave the project in a half-migrated state where major write flows still depend on both old concrete local service orchestration and new application use cases as competing primary paths.

#### Scenario: Core write flows use one primary architecture

- **WHEN** import, generation, metadata review, album, gallery, task, CLI, daemon and Tauri write paths are reviewed after implementation
- **THEN** each migrated path uses the new application facade or use case boundary as its primary architecture
- **AND** old concrete service orchestration is not a competing primary path for the same behavior

#### Scenario: Change is not split into future architecture changes

- **WHEN** tasks for this change are completed
- **THEN** remaining work may include future product capabilities
- **AND** remaining work MUST NOT include unfinished core DDD boundary migration required by this proposal

### Requirement: Architecture checks SHALL enforce dependency direction

System SHALL include verification that prevents domain modules from depending on infrastructure or runtime modules, and prevents runtime layers from bypassing application/use case boundaries for migrated write flows.

#### Scenario: Dependency direction check runs

- **WHEN** maintainability verification is executed
- **THEN** the verification reports whether domain modules import SQLite, filesystem IO, daemon, Tauri, CLI parser or desktop view modules
- **AND** the verification reports whether application modules import SQLite, filesystem IO, runtime crates, infrastructure adapters or legacy library implementation modules
- **AND** any violation is fixed or explicitly documented as a blocker

#### Scenario: Runtime bypass check runs

- **WHEN** CLI, daemon or Tauri code is migrated
- **THEN** verification or review confirms runtime code calls the application facade or use case entrypoints instead of recreating domain decisions locally

#### Scenario: Desktop compatibility barrel check runs

- **WHEN** desktop workflow ownership is migrated
- **THEN** verification reports any desktop source module that imports the legacy `workbench-state` barrel as a primary state owner
- **AND** migrated workflow code imports workflow-owned modules directly

### Requirement: Complexity review SHALL cover each migrated domain

System SHALL review each migrated bounded context for reusable logic, duplication, ownership clarity, and cyclomatic complexity risk. The refactor MUST reduce or preserve complexity in critical paths; it MUST NOT hide complexity by moving large functions into newly named DDD modules.

#### Scenario: Domain complexity scan is documented

- **WHEN** a bounded context migration is completed
- **THEN** tasks or verification notes identify remaining large files, high-complexity functions, duplicated rules, and deferred cleanup if any

#### Scenario: Shared rules are not duplicated

- **WHEN** two migrated use cases need the same domain rule
- **THEN** code review confirms the rule is reused through a policy, domain service, helper, or shared application component

#### Scenario: Facade owners do not duplicate focused use cases

- **WHEN** an application facade owner is introduced to group focused use cases
- **THEN** code review confirms the owner delegates to the focused use cases or shared components
- **AND** the owner does not create a second copy of already-migrated validation, allocation or persistence command construction logic

### Requirement: Behavior-preserving refactor SHALL include public contract tests

System SHALL verify that the DDD refactor preserves public behavior for CLI JSON output, daemon loopback API, desktop workflows, provider generation behavior, and persisted library access.

#### Scenario: Public behavior checks pass

- **WHEN** implementation claims the DDD refactor is complete
- **THEN** Rust formatting, Rust core tests, daemon tests, CLI tests, desktop frontend tests, desktop build and OpenSpec validation pass
- **AND** any unavailable or failing check is documented with cause and residual risk

#### Scenario: Existing library compatibility is checked

- **WHEN** implementation completes
- **THEN** verification includes opening or exercising a pre-refactor test library fixture or equivalent compatibility flow
- **AND** the result confirms no user-visible migration is required solely because of code reorganization

### Requirement: Tests SHALL prefer owning boundaries while preserving regression coverage

System SHALL add or relocate tests so new or migrated domain rules, application use cases, infrastructure repositories and runtime adapters are tested near their owning modules. Legacy large integration test files MAY remain as compatibility regression suites when they exercise public behavior, persisted library compatibility, or cross-context flows. They MUST NOT be the primary home for newly migrated domain/application rules, and their remaining role MUST be documented in verification notes.

#### Scenario: Domain rule tests are focused

- **WHEN** asset version, reference source, task state transition or generation planning rules are migrated
- **THEN** tests for those rules live near the owning domain or application modules

#### Scenario: Infrastructure tests cover adapter behavior

- **WHEN** SQLite or filesystem adapters are migrated
- **THEN** tests verify persistence, migration/backfill and managed file behavior without duplicating unrelated domain rule tests

#### Scenario: Legacy regression tests remain documented

- **WHEN** a legacy large test file remains after the refactor
- **THEN** verification notes identify it as compatibility or cross-context regression coverage
- **AND** newly migrated reusable rules are still covered by owner-local focused tests

### Requirement: Systematic review findings are tracked to implementation tasks

Architecture review findings SHALL be recorded with severity, area, evidence, impact, recommendation, and validation. High and critical findings MUST map to OpenSpec tasks or be explicitly deferred with rationale.

#### Scenario: Finding maps to task

- **WHEN** a review finding is marked High or Critical
- **THEN** the finding has a corresponding task, validation item, or explicit deferred decision

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Large owner is refactored

- **WHEN** a hotspot file such as a large controller, read model, repository, or regression suite is refactored
- **THEN** the resulting files have clear owners
- **AND** tests move or remain according to those owners

#### Scenario: Gallery read model is split incrementally

- **WHEN** gallery read-model code is split
- **THEN** search, gallery list, asset detail, version tree, album filter context, and file context concerns are separated in focused waves
- **AND** each wave preserves public behavior and records remaining split targets

### Requirement: Persistence performance decisions use workload evidence

Performance refactors for gallery, search, smart albums, version tree, and task queue SHALL use workload evidence before choosing a storage or indexing architecture.

#### Scenario: Query engine decision is evidence-based

- **WHEN** implementation proposes SQLite tuning, FTS5/projection tables, Tantivy, DuckDB, or PostgreSQL
- **THEN** the proposal includes workload evidence, migration impact, and backup/restore implications
