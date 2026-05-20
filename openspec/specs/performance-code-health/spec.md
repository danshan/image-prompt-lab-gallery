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

