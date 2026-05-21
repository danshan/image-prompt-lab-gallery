## Context

当前项目已经从早期大文件演进到按 runtime boundary 和局部业务责任拆分的结构, 例如 `imglab-core/src/library/*.rs`, `imglab-daemon/src/*.rs`, `apps/desktop/src-tauri/src/commands/*` 和 desktop controller hooks. 这解决了一部分文件规模问题, 但还没有形成真正的 DDD 边界:

- `imglab-core` 仍同时暴露 identity, command DTO, read model, provider payload, task model 和 concrete local implementation.
- generation flow 直接依赖 `LocalLibraryService`, SQLite database, filesystem temp file 和 metadata suggestion 写入.
- asset/version 写入同时处理 aggregate rule, storage layout, checksum, image header parsing 和 SQL insert.
- daemon/Tauri/CLI 直接依赖 core concrete service, interface DTO 与 domain model 边界不清晰.
- frontend root 仍承担过多 workflow composition 与 orchestration glue.

本 change 要在一个 OpenSpec change 内完成完整重构, 但它仍是 maintainability refactor: 默认不改变用户可见行为, 不改变 daemon API shape, 不改变 CLI JSON contract, 不改变现有 resource library 的持久化语义。

## Goals / Non-Goals

**Goals:**

- 在 `imglab-core` 内建立完整 DDD 分层: domain, application, ports, infrastructure, interface contracts.
- 为 library, asset/version, generation, metadata review, albums/search 和 task manager 明确 bounded context.
- 将 domain rule 从 SQLite/filesystem implementation 中提取出来, 让 repository 只负责持久化.
- 将 application use case 改为依赖 ports, 而不是直接构造 concrete `LocalLibraryService`.
- 将 CLI, daemon, Tauri 和 frontend 依赖的 DTO/view model 与 domain model 分离, 通过 mapper 显式转换.
- 在每个 bounded context 内保持可复用的 domain logic, application helper 和 mapper, 控制重复逻辑与圈复杂度.
- 保留 public behavior, persisted library behavior 和 provider behavior.
- 在同一个 change 内完成重构, 测试迁移, 文档更新和 OpenSpec 验证, 不留下半新半旧的 core 架构。

**Non-Goals:**

- 不新增 native OpenAI 或 Grok provider.
- 不新增 daemon endpoint 或改变 loopback API.
- 不改变 SQLite 作为当前 resource library 主事实存储的选择.
- 不引入 cloud sync, multi-user, encryption 或 daemon HTTP public API.
- 不做用户可见 UI redesign.
- 不因为代码重组而要求用户迁移既有 library.

## Decisions

### Decision 1: 先在 `imglab-core` 内完成 DDD module boundary, 暂不拆多 crate

选择:

```text
crates/imglab-core/src/
  domain/
    shared/
    library/
    asset/
    generation/
    metadata_review/
    album/
    task/
  application/
    ports/
    use_cases/
    read_models/
  infrastructure/
    sqlite/
    filesystem/
    registry/
  interface_contracts/
```

理由:

- 当前 CLI, daemon, Tauri 和 provider crates 已稳定依赖 `imglab-core`; 先保持 crate boundary 可减少 Cargo workspace 和 public dependency churn.
- DDD 的关键收益来自依赖方向和 ownership, 不必先通过 crate 物理拆分实现.
- 后续如果某些 context 成熟, 可以再把 `domain` 或 `infrastructure` 拆为独立 crate.

替代方案:

- 直接拆成 `imglab-domain`, `imglab-application`, `imglab-infrastructure`. 这种方式边界最硬, 但一次性修改 Cargo dependency graph, test target, public exports 和 downstream imports, 对当前 MVP 风险偏高.

### Decision 2: Domain model 不再承载 interface DTO shape

Domain 层只保留:

- identity/value object, 例如 `AssetId`, `AssetVersionId`, `LibraryId`.
- aggregate/entity, 例如 `Asset`, `AssetVersion`, `ResourceLibrary`, `Task`.
- domain enum 和 invariant, 例如 version number, parent chain, reference source rule, task status transition.
- domain service, 仅处理不依赖 infrastructure 的业务判断.

CLI JSON, daemon view, Tauri command view 和 frontend adapter DTO 放入 `interface_contracts` 或各 runtime 自己的 view 模块, 通过 mapper 与 application model 转换。

理由:

- 避免为了某个 runtime 的 serialization shape 污染 domain model.
- 允许 domain model 用更强类型表达 invariant, 而 read model 可以针对 UI/API 优化.

替代方案:

- 继续共享 DTO, 只拆文件. 这种方案改动小, 但不能解决 bounded context namespace 混杂和上层绕过 domain rule 的问题.

### Decision 3: Application use case 依赖 ports, infrastructure 实现 ports

Application 层定义 use case 与 ports:

- `ImportAssetUseCase`
- `CreateChildVersionUseCase`
- `GenerateImageUseCase`
- `ReviewMetadataSuggestionUseCase`
- `CreateAlbumUseCase`
- `QueryGalleryUseCase`
- `EnqueueTaskUseCase`
- `RunTaskAttemptUseCase`

核心 ports:

- `LibraryRepository`
- `AssetRepository`
- `GenerationEventRepository`
- `MetadataSuggestionRepository`
- `AlbumRepository`
- `TaskRepository`
- `ManagedFileStore`
- `LibraryRegistry`
- `ImageProvider`
- `Clock`
- `IdGenerator`
- `TransactionManager`

理由:

- generation 不再直接构造 `LocalLibraryService`.
- repository 不再拥有业务决策, 只负责 load/save/query.
- use case 可以用 in-memory/fake ports 做快速测试, 不必每个业务测试都走 SQLite + filesystem.

替代方案:

- 保持 trait service facade, 只移动 implementation. 这能减少改动, 但仍会让 concrete local service 成为上帝对象.

### Decision 4: Asset aggregate 拥有 version lineage 和 reference source 规则

`Asset` aggregate 负责:

- 创建首个 version.
- 创建 child version.
- 分配 asset 内递增 version number.
- 验证 parent version 属于同一 asset.
- 区分 same-asset parent chain 与 cross-asset reference source.

Generation use case 负责 orchestration:

- text-to-image 创建新 asset.
- image-to-image with input version 若同 asset 则创建 child version.
- uploaded reference 先作为独立 reference asset/version 导入, output asset 不并入 reference asset lineage.
- generation event 记录 input/output 关系.

理由:

- version number 和 parent chain 是 asset aggregate invariant, 不应由 SQL helper 或 generation workflow 隐式维护.
- 这与现有 `asset-versioning` spec 中内部 UUID + 用户可见数字版本, parent chain/reference source 分离的 contract 一致.

### Decision 5: SQLite schema 暂不因为 DDD 重构而改变

本 change 默认只重组代码和 ownership, 不改变 schema version. 允许把 migration source file 拆成 context-owned fragments, 但最终数据库 schema, backfill 行为和 `CURRENT_SCHEMA_VERSION` 不因重构本身改变。

理由:

- 持久化格式变化会显著提高风险, 应独立通过 schema-specific OpenSpec change 管理.
- 这次目标是 boundary correctness 和 maintainability, 不是数据模型演进.

替代方案:

- 顺手归一化 schema. 这会把架构重构和数据迁移风险叠加, 不适合作为当前 change 的默认路径.

### Decision 6: Runtime 层依赖 application facade, 不直接依赖 concrete local service

新增或收敛一个 application facade, 例如:

```text
ImgLabApplication
  library
  assets
  generation
  metadata_review
  albums
  gallery
  tasks
```

CLI, daemon 和 Tauri commands 调用 facade 或 use case entrypoint. Infrastructure composition root 负责装配 SQLite repositories, filesystem store, registry, provider adapters, clock/id generator 和 transaction manager。

理由:

- daemon/Tauri 不再知道 `LocalLibraryService` 的内部模块布局.
- provider adapter 与 task executor 可以通过同一 generation use case 复用业务规则.

### Decision 7: Desktop frontend 同步收敛 workflow ownership

Frontend 不需要套用后端 DDD 名词, 但要同步落实 ownership:

- `App.tsx` 只做 shell composition 和 top-level provider wiring.
- Gallery, Albums, Review, Queue, Settings 各自拥有 controller hook, view model mapper 和 screen.
- `workbench-state.ts` 拆成 workflow-owned pure state modules.
- Tauri adapter 保持 transport-only, 不持有 workflow business rule.

理由:

- 现有 spec 已要求 root component 保持组合职责.
- 后端 use case 边界清晰后, frontend 也需要避免把 workflow orchestration 回流到 root.

### Decision 8: 每个 bounded context 内部也必须做低复杂度设计

DDD 重构不能只完成目录分层。每个 context 内部需要按真实变化原因继续拆分:

- `model`: entity, value object 和 invariant.
- `commands`: use case command/input normalization, 不包含 persistence.
- `policies`: 可复用业务策略, 例如 version numbering, task transition, retry classification.
- `repositories`: port trait 或 infrastructure adapter.
- `mappers`: domain/application/interface contract 转换.
- `queries`: read model query 组装和 projection.
- `tests`: 靠近 owner 的 focused tests.

约束:

- 不用 arbitrary line count 驱动拆分, 但对明显承担多个变化原因的文件必须拆分.
- 重复逻辑优先抽成 context-local helper 或 shared domain service, 不复制到多个 use case.
- 大函数需要按 decision, validation, persistence orchestration 和 mapping 拆成可测试步骤.
- 新增 abstraction 必须消除真实重复或隔离明确依赖方向, 不为形式化 DDD 增加空壳层.

理由:

- DDD 的目标是让业务语言和 invariant 可维护, 不是简单制造目录层级.
- 当前项目已有复杂 read model, generation workflow 和 task scheduler, 如果只移动文件不降低圈复杂度, 后续维护风险不会真正下降.

## Risks / Trade-offs

- 大规模移动文件导致行为回归 -> 以 behavior-preserving refactor 为硬约束, 每个阶段运行 Rust/TypeScript tests, 并补充 import/architecture checks.
- DDD 抽象过度, 增加样板代码 -> 只为已有 bounded context 和真实 dependency inversion 建 ports, 不为简单 helper 创建抽象.
- Context 内部继续出现大函数和重复 mapper -> 为每个 migrated context 增加 ownership review, 检查重复逻辑, helper 复用和高复杂度函数.
- 一次 change 完成全部重构导致 PR 过大 -> tasks 按可验证阶段拆分, 但都留在同一个 OpenSpec change 内完成.
- Re-export 兼容层拖延清理 -> 允许短期 compatibility exports, 但 tasks 必须包含最终收敛, 不让旧 namespace 成为长期主入口.
- SQLite transaction boundary 与 use case boundary 不匹配 -> 先定义 `TransactionManager` port, 让 application 表达原子性, SQLite infrastructure 实现事务.
- Test rewrite 成本较高 -> 将现有 integration tests 作为回归网, 对 domain/application 新增 focused unit tests, 不要求每个 repository helper 都复制旧测试形态.

## Migration Plan

1. 建立新 module skeleton 和 compatibility exports, 确保项目仍可编译.
2. 迁移 domain identity/value objects, aggregate 和 domain rules.
3. 迁移 application ports 和 use cases, 先保留旧 service facade 委托到新 use cases.
4. 迁移 SQLite/filesystem/registry implementation 到 infrastructure.
5. 更新 CLI, daemon, Tauri backend 使用 application facade.
6. 收敛 frontend workflow ownership.
7. 清理旧 namespace, 删除不再需要的 compatibility paths.
8. 更新 docs 和 OpenSpec task 状态.
9. 跑完整验证: formatting, Rust tests/checks, desktop tests/build, OpenSpec validation, source ownership scan.

Rollback strategy:

- 因为本 change 默认不改 persisted schema, rollback 可以通过 Git revert 回退代码.
- 若实现中发现必须触碰 schema version, 必须先更新 design/spec/tasks, 并增加旧 library upgrade/rollback 说明后再继续.

## Open Questions

- `interface_contracts` 是否保留在 `imglab-core` 内, 还是让 CLI/daemon/Tauri 各自拥有 runtime DTO? 默认先保留 core shared contracts, 对 runtime-specific view 仍放在对应 runtime.
- `TransactionManager` port 是否需要同步支持跨多个 repository 的 unit of work? 默认需要, 因为 asset version + generation event + task output 属于同一个提交语义.
- 是否要为 architecture checks 引入新工具? 默认先使用仓库脚本或轻量 shell/Node scan, 不新增重依赖.
