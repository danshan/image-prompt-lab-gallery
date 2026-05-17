## ADDED Requirements

### Requirement: Core 模块按职责拆分
系统 SHALL 将 resource library core 实现组织为按职责划分的内部模块, 并保持 `LocalLibraryService` 作为对 CLI 和 desktop 的稳定 public service entry point.

#### Scenario: Core 内部模块边界清晰
- **WHEN** 开发者查看 `imglab-core` resource library 实现
- **THEN** schema, storage, assets, gallery, metadata, albums, repair, export 和 generation helper 逻辑分别位于职责明确的模块中

#### Scenario: Public service entry point 保持稳定
- **WHEN** CLI 或 desktop 调用 `LocalLibraryService`
- **THEN** 调用方不需要了解 core 内部模块拆分细节

### Requirement: Transport 层不得承载业务查询语义
系统 MUST 保证 Tauri command 层只负责 transport mapping, 不得直接打开 SQLite 来实现 Gallery, Inspector 或 resource library 业务 read model.

#### Scenario: Desktop 查询 Gallery
- **WHEN** 桌面端请求 Gallery asset 列表
- **THEN** Tauri command 调用 Rust core read service 并返回 core 定义的 Gallery read model

#### Scenario: Desktop 加载 Inspector
- **WHEN** 桌面端请求 asset detail
- **THEN** Tauri command 调用 Rust core read service 并返回 core 定义的 asset detail read model

### Requirement: Generation Orchestration 复用
系统 SHALL 在 CLI 和 desktop generation flow 之间复用 provider dispatch, operation inference, input loading 和 request construction 边界.

#### Scenario: CLI 和 Desktop 构造同类 Generation Request
- **WHEN** CLI 和 desktop 使用相同 provider, prompt 和 input version 发起 generation
- **THEN** 两者使用一致的 operation inference, provider capability check 和 input image loading 规则

#### Scenario: Provider Execution 仍由 Provider Crate 承担
- **WHEN** generation flow 选择 Codex CLI provider
- **THEN** Codex command construction, log capture 和 output parsing 仍由 Codex provider crate 承担, 不进入 core orchestration helper

### Requirement: 去除重复业务 Helper
系统 SHALL 将 tag attach, version row mapping, latest generation event lookup 和 rating validation 等重复逻辑收敛到单一 helper 或单一模块边界.

#### Scenario: 添加 Tag
- **WHEN** metadata review accept flow 和手动 add tag flow 都需要绑定 tag
- **THEN** 两者使用同一 tag upsert/attach 逻辑

#### Scenario: 映射 Version Row
- **WHEN** core 从 SQLite 读取 asset version
- **THEN** 共享 row mapping 逻辑生成一致的 version read model
