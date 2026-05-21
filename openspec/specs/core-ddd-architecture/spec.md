# core-ddd-architecture Specification

## Purpose
Define the DDD boundary architecture for `imglab-core` and the runtime integration rules that keep domain logic independent from infrastructure, runtime views, and compatibility shims.

## Requirements

### Requirement: Core 使用 DDD 分层

`imglab-core` SHALL 按 DDD 边界组织主要业务代码, 至少包含 domain, application, ports, infrastructure 和 interface contracts 责任区。Domain 层 MUST 不依赖 SQLite, filesystem IO, daemon, Tauri, CLI 或 frontend view 类型。

#### Scenario: Domain 不依赖基础设施

- **WHEN** 代码完成 DDD 重构
- **THEN** domain modules 不导入 `rusqlite`, `std::fs`, Tauri, daemon runtime, CLI command parser 或 desktop view 类型

#### Scenario: Application 不依赖基础设施实现

- **WHEN** 代码完成 DDD 重构
- **THEN** application modules 不导入 `rusqlite`, `std::fs`, Tauri, daemon runtime, CLI command parser, infrastructure adapters 或 legacy library implementation modules
- **AND** application use cases 通过 ports 读取事实和提交 persistence command

#### Scenario: Core 主入口表达边界

- **WHEN** 开发者查看 `imglab-core` module tree
- **THEN** 能从 module names 区分 domain model, application use cases, ports, infrastructure adapters 和 interface contracts

### Requirement: Bounded Context 拥有自己的语言模型

系统 SHALL 为 resource library, asset/version, generation, metadata review, albums/search 和 task manager 建立清晰 bounded context。每个 bounded context MUST 拥有自己的 aggregate, command, query/read model 和 repository port 边界, 不得继续把所有业务结构集中在单个 shared DTO namespace 中作为主要模型。

#### Scenario: Asset Version 语言模型独立

- **WHEN** asset/version 代码被重构
- **THEN** asset aggregate 拥有 version number, version name, parent chain 和 reference source rule 的 domain model

#### Scenario: Task 语言模型独立

- **WHEN** task manager 代码被重构
- **THEN** task status, attempt, event, output link 和 scheduler policy 的模型位于 task bounded context 或 application task modules 中

### Requirement: Domain 内部保持低冗余和低复杂度

每个 bounded context SHALL 在保持 DDD 边界的同时, 将 domain logic, policy, mapper, query 和 application orchestration 拆成可复用的低复杂度单元。实现 MUST 避免把旧的大文件或大函数平移为新的 context-local 大文件, MUST 避免跨 use case 复制同一业务规则。

#### Scenario: Context Logic 可复用

- **WHEN** 多个 use case 需要相同业务规则, 例如 version number, task transition, provider normalization 或 reference source 判断
- **THEN** 该规则由 context-local policy, domain service 或 shared helper 复用
- **AND** 不在多个 use case 中复制实现

#### Scenario: Application Owner 不复制 Focused Use Case 规则

- **WHEN** an application facade owner aggregates focused use cases for a bounded context
- **THEN** the owner delegates to the focused use case or shared helper for existing rules
- **AND** it does not copy validation, version allocation, persistence command construction or file orchestration logic into a second implementation

#### Scenario: 高复杂度函数被拆分

- **WHEN** migrated domain 或 application function 同时承担 validation, decision, persistence orchestration 和 mapping
- **THEN** implementation 将其拆分为命名清晰, 可独立测试或 review 的步骤

#### Scenario: 拆分基于 Ownership 而非形式

- **WHEN** context 内部文件被拆分
- **THEN** 每个文件有明确 ownership 和变化原因
- **AND** 不为了满足目录形式创建没有行为或没有依赖隔离价值的空壳 abstraction

### Requirement: Application Use Case 依赖 Ports

Application use case SHALL 依赖 repository, file store, provider, clock, id generator 和 transaction ports, 不得直接构造 concrete SQLite, filesystem, registry 或 local service implementation。Infrastructure adapters MUST 实现这些 ports 并由 composition root 装配。

#### Scenario: Generation Use Case 不构造 Local Service

- **WHEN** generation flow 执行 text-to-image 或 image-to-image
- **THEN** use case 通过 ports 读取和写入 library, asset version, generation event, metadata suggestion 和 managed files
- **AND** use case 不直接构造 `LocalLibraryService`

#### Scenario: Use Case 可用 Fake Ports 测试

- **WHEN** application use case 需要验证 domain rule
- **THEN** 测试可以使用 fake 或 in-memory ports, 不必须启动 SQLite database 和真实 filesystem

#### Scenario: Facade Owner 使用 Application Use Case

- **WHEN** the SQLite-backed composition root exposes asset import or child-version behavior
- **THEN** the facade owner for assets is an application use-case owner
- **AND** runtime callers do not receive the legacy local service as the primary asset owner for migrated asset write behavior

### Requirement: Infrastructure 负责持久化和外部系统

Infrastructure 层 SHALL 拥有 SQLite repositories, schema migration source, managed filesystem storage, manifest/registry implementation 和 provider adapters。Infrastructure MUST NOT 成为业务决策入口, 业务 invariant MUST 位于 domain 或 application 层。

#### Scenario: Repository 不分配业务版本语义

- **WHEN** repository 保存新的 asset version
- **THEN** version number, parent chain 和 reference source 语义已经由 domain/application 决定
- **AND** repository 只负责持久化这些已验证结果

#### Scenario: Schema 分片仍统一迁移

- **WHEN** SQLite schema source 被按 bounded context 拆分
- **THEN** library database 仍通过统一 migration entrypoint 执行迁移和 backfill

### Requirement: Interface Contracts 与 Domain Model 分离

CLI JSON, daemon HTTP view, Tauri command view 和 desktop frontend adapter payload SHALL 不作为 domain model 的主要形态。Runtime-facing DTO MUST 通过 mapper 从 application/domain model 转换而来。

#### Scenario: CLI 输出保持兼容

- **WHEN** CLI command 返回 JSON output
- **THEN** 输出 shape 与重构前兼容
- **AND** domain model 不需要为了 CLI JSON 字段命名暴露 runtime-specific structure

#### Scenario: Daemon API 保持兼容

- **WHEN** desktop client 调用 daemon loopback API
- **THEN** endpoint, authentication 和 response shape 与重构前兼容
- **AND** daemon view 通过 mapper 从 application result 构造

### Requirement: DDD 重构保持持久化兼容

本次 DDD 重构 SHALL 默认不改变 resource library SQLite schema version, manifest identity, managed file layout 或 registry 语义。若实现中发现必须改变持久化格式, MUST 先更新本 change 的 design, specs 和 tasks, 并加入旧 library upgrade 和 rollback 验证。

#### Scenario: 既有 Library 可打开

- **WHEN** 用户使用重构后的应用打开重构前创建的 resource library
- **THEN** library 可按原有 schema 和 manifest 语义打开
- **AND** 不因为代码 module 重组要求用户执行手工迁移

#### Scenario: Schema Change 需要显式更新

- **WHEN** 实现需要修改 SQLite schema version 或 manifest format
- **THEN** 本 change 的 artifacts 必须先记录 schema 变化, upgrade 语义和验证步骤

### Requirement: Compatibility Facade 逐步收敛

系统 SHALL 允许短期 compatibility exports 帮助分阶段迁移 CLI, daemon 和 Tauri imports, 但本 change 完成时 primary imports MUST 指向新的 DDD boundary。旧的 all-in-one DTO 或 local service namespace MUST 不再作为新增代码主入口。

#### Scenario: 旧 Export 不再是主入口

- **WHEN** DDD 重构完成
- **THEN** 新增或迁移后的 runtime 代码通过 application facade, use case 或 interface contract imports 访问 core
- **AND** 旧 shared DTO namespace 仅作为兼容层或被删除

#### Scenario: Compatibility State Barrel 不再是前端主入口

- **WHEN** desktop workflow ownership cleanup is complete
- **THEN** desktop source modules import workflow-owned state, query, derived-state and controller modules directly
- **AND** the legacy `workbench-state` barrel remains only for compatibility tests or explicitly documented transitional use
