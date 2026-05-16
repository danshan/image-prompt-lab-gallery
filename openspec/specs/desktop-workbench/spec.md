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
