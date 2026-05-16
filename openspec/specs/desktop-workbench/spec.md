## ADDED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供 `Library Sidebar | Workspace | Inspector` 三栏工作台, 并允许 Inspector 在窄窗口中折叠.

#### Scenario: 选择 Gallery 图片

- **WHEN** 用户在 Gallery 中选择一个 asset
- **THEN** Workspace 保持图片网格上下文, Inspector 展示该 asset 的 metadata, prompt, tags, albums 和 versions

### Requirement: 提供 Gallery 和 Albums 视图

桌面应用 SHALL 展示导入和生成的图片, 并支持进入 manual album 和 smart album 视图.

#### Scenario: 打开智能相册

- **WHEN** 用户打开一个 smart album
- **THEN** Workspace 展示当前满足 smart query 的 asset 列表

### Requirement: 提供 Generation Composer 和 Queue

桌面应用 SHALL 支持用户发起文生图和图生图, 并展示 generation queue 状态.

#### Scenario: 发起图生图

- **WHEN** 用户在 Inspector 中选择一个 asset version 并输入新 prompt
- **THEN** 应用创建 generation job, 在 queue 中展示状态, 完成后刷新 Gallery 和 Inspector lineage

### Requirement: 提供 Review Inbox

桌面应用 SHALL 提供 Review Inbox, 用于处理 pending metadata suggestions.

#### Scenario: 接受 Suggestion

- **WHEN** 用户在 Review Inbox 接受某条 suggestion
- **THEN** 应用调用 Rust core 写入 canonical metadata, 并从 pending 列表中移除该 suggestion

### Requirement: 所有写操作通过 Rust Core

桌面应用 MUST 通过 Tauri command 调用 Rust core 完成资源库写操作, 不得在前端直接写 SQLite 或 managed files.

#### Scenario: 更新评分

- **WHEN** 用户在 Inspector 中修改 asset rating
- **THEN** 桌面应用调用 core service 更新 rating, 并根据返回结果刷新 UI state

### Requirement: 提供产品化 Gallery 工作台

桌面应用 SHALL 提供接近设计稿风格的 Gallery 工作台, 包含 library sidebar, workspace toolbar, gallery card grid 和 asset Inspector, 并保持高信息密度与稳定布局.

#### Scenario: 打开 Gallery 主工作流

- **WHEN** 用户打开已注册资源库的 Gallery
- **THEN** 桌面应用展示 library sidebar, Gallery 查询工具栏, asset card grid 和当前选中 asset 的 Inspector

#### Scenario: 选择 Gallery 图片

- **WHEN** 用户选择 Gallery 中的某个 asset card
- **THEN** card 显示选中状态且尺寸不发生变化, Inspector 加载该 asset 的详情

### Requirement: 通过 Core Read Model 加载 Gallery 和 Inspector

桌面应用 MUST 通过 Tauri command 调用 Rust core 获取 Gallery card 列表和 Inspector asset detail, 不得在前端用多个低级结果重建业务查询语义.

#### Scenario: 查询 Gallery 列表

- **WHEN** 用户修改搜索, filter 或 sort 条件
- **THEN** 桌面应用调用 core 定义的 Gallery query command 并使用返回的 card read model 渲染列表

#### Scenario: 加载 Inspector 详情

- **WHEN** 用户选择某个 asset
- **THEN** 桌面应用调用 asset detail command 并展示 prompt, provider/model, tags, albums, lineage 和 file sections

### Requirement: 提供可恢复错误和空状态

桌面应用 SHALL 对 Gallery 和 Inspector 的 loading, empty 和 recoverable error 状态提供明确 UI 反馈.

#### Scenario: Provider 不支持 Variation

- **WHEN** 用户从 Inspector 发起 variation 且当前 provider 不支持图生图
- **THEN** 桌面应用在 generation 或 variation 区域展示可恢复错误, 且 workbench 仍保持可操作

### Requirement: 支持 Inspector 响应式折叠

桌面应用 SHALL 在窄窗口中折叠 Inspector, 并保持 Gallery 查询和选择流程可用.

#### Scenario: 窄窗口查看 Asset 详情

- **WHEN** 窗口宽度不足以展示三栏布局且用户选择 asset
- **THEN** 桌面应用以 drawer, overlay 或详情面板展示 Inspector 内容, 且不遮挡主要导航和查询入口
