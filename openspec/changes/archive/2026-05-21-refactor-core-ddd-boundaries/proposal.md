## Why

当前架构已经能支撑 MVP, 但 `imglab-core` 同时承载 domain model, application orchestration, SQLite repository, filesystem storage, provider boundary, task model 和 read model contract, 导致业务语义和基础设施实现逐步耦合。现在需要在继续扩展 native provider, daemon task, metadata review 和 library schema 前, 以 DDD 为代码设计思想完成一次完整边界重构, 降低后续行为变更和持久化演进的风险。

## What Changes

- 将 core 代码重新规划为 DDD 分层: domain, application, ports, infrastructure 和 interface contracts.
- 为 resource library, asset/version, generation, metadata review, albums/search 和 task manager 明确 bounded context 与 aggregate ownership.
- 将 application use case 从 concrete `LocalLibraryService` 和 SQLite/filesystem 实现中解耦, 改为依赖 ports.
- 将 SQLite repository, filesystem managed storage, manifest/registry 和 provider adapters 收敛到 infrastructure 边界.
- 将 CLI, daemon, Tauri 和 desktop 依赖的 DTO/view model 与 domain model 显式分离, 通过 mapper 连接.
- 在每个 domain/bounded context 内保持逻辑可复用, 低冗余和低圈复杂度, 避免把旧的复杂度平移为新的 context 内大文件或大函数.
- 完整迁移现有实现到新边界, 不把 DDD 迁移拆成多个 OpenSpec change.
- 保持 public behavior 兼容: CLI JSON, daemon API, desktop workflow, provider behavior 和 persisted resource library 行为不得因本次重构改变.
- 不引入新的 user-facing product capability, 不改变 SQLite schema, 除非实现中发现必须补齐的兼容性 backfill, 且需要在 tasks 中单独标记和验证.

## Capabilities

### New Capabilities

- `core-ddd-architecture`: 定义 core DDD 分层, bounded context, aggregate ownership, application ports, infrastructure adapters, interface contracts 和 refactor compatibility contract.

### Modified Capabilities

- `performance-code-health`: 将现有 maintainability refactor 要求升级为一个完整 DDD boundary refactor, 要求本 change 内完成 core, daemon, Tauri backend 和 desktop frontend 的边界收敛, 并通过结构检查防止回退到 mega file 或跨层依赖.

## Impact

- Affected Rust crates:
  - `crates/imglab-core`
  - `crates/imglab-cli`
  - `crates/imglab-daemon`
  - `crates/imglab-provider-codex`
  - `crates/imglab-provider-grok`
  - `apps/desktop/src-tauri`
- Affected frontend modules:
  - `apps/desktop/src/app`
  - `apps/desktop/src/workbench-state.ts`
  - `apps/desktop/src/studio-*.ts`
- Affected specs:
  - New `core-ddd-architecture`
  - Modified `performance-code-health`
- Public compatibility constraints:
  - CLI command behavior and JSON shape remain stable.
  - Daemon loopback API endpoints, authentication and response shape remain stable.
  - Desktop workflows remain behaviorally stable.
  - Existing resource libraries open without user-visible migration solely because of this refactor.
  - Existing OpenSpec capabilities remain semantically unchanged unless this change explicitly updates architecture requirements.
