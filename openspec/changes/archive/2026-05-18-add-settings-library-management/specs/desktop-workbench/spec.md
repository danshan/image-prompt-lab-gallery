## ADDED Requirements

### Requirement: Settings 提供 Libraries 和 Logs 子页

桌面应用 SHALL 在 Settings workspace 中提供 `Libraries` 和 `Logs` 两个子页. Settings 默认打开 `Libraries` 子页. 切换 Settings 子页 MUST NOT 改变当前 resource library.

#### Scenario: 打开 Settings 默认进入 Libraries

- **WHEN** 用户打开 Settings
- **THEN** 桌面应用展示 `Libraries` 子页, 并提供切换到 `Logs` 子页的控件

#### Scenario: 切换 Settings 子页不改变当前 Library

- **WHEN** 用户在 `Settings / Libraries` 和 `Settings / Logs` 之间切换
- **THEN** 当前 resource library, Gallery query 和 Inspector selection 不因子页切换而改变

### Requirement: Settings Libraries 维护多个资源库

桌面应用 SHALL 在 `Settings / Libraries` 中展示 registered libraries 管理表, 并提供创建资源库, 打开已有资源库文件夹, 导入备份 zip, 切换当前资源库, 重命名本机 alias, 取消注册, 导出备份 zip 和 Reveal in Finder 操作.

#### Scenario: 查看 Registered Libraries 管理表

- **WHEN** 用户打开 `Settings / Libraries`
- **THEN** 桌面应用展示 registered libraries 列表, 每行包含 name, path 和 actions, 并将 missing on disk 作为行状态标记

#### Scenario: 创建资源库后设为当前资源库

- **WHEN** 用户在 `Settings / Libraries` 中输入 library name 和 folder name, 选择父目录并创建新资源库成功
- **THEN** 桌面应用刷新 registered libraries, 将新资源库设为当前资源库, 并清空旧资源库相关的 Workspace 和 Inspector 上下文

#### Scenario: 打开已有资源库文件夹

- **WHEN** 用户选择 `Open Existing Library` 并选择有效资源库目录
- **THEN** 桌面应用调用 Rust core 校验并注册该资源库, 刷新 registered libraries, 并将其设为当前资源库

#### Scenario: 重命名当前资源库 Alias

- **WHEN** 用户在 `Settings / Libraries` 中重命名当前资源库 alias
- **THEN** 桌面应用刷新 registered libraries, 并在 Sidebar Library selector 和 Settings row 中展示新 alias

#### Scenario: 取消注册当前资源库

- **WHEN** 用户 Close 当前资源库并确认
- **THEN** 桌面应用调用 Rust core 取消注册该资源库, 清空当前 library, Gallery, Inspector detail, Albums, Review, Queue 和 selected ids, 并进入 no-library state

#### Scenario: 取消注册非当前资源库

- **WHEN** 用户 Close 一个非当前资源库并确认
- **THEN** 桌面应用刷新 registered libraries, 且当前 library 和 Workspace 上下文保持不变

#### Scenario: 导出资源库备份 Zip

- **WHEN** 用户对一个有效资源库执行 Export Zip 并选择保存路径
- **THEN** 桌面应用调用 Rust core 导出完整资源库备份 zip, 并在操作期间阻止重复提交

#### Scenario: 导入资源库备份 Zip

- **WHEN** 用户选择有效备份 zip 和目标目录
- **THEN** 桌面应用调用 Rust core 导入并注册该资源库, 刷新 registered libraries, 并默认切换到导入后的资源库

#### Scenario: Reveal in Finder

- **WHEN** 用户对一个路径存在的资源库执行 Reveal in Finder
- **THEN** 桌面应用通过操作系统文件管理器打开该资源库 root folder, 且不改变当前资源库状态

#### Scenario: Missing Path 行为

- **WHEN** registered library 的 root path 不存在
- **THEN** `Settings / Libraries` 将该行标记为 missing on disk, 保持 Close 可用, 并禁用或拒绝 Export Zip 和 Reveal in Finder

## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供 `Library Sidebar | Workspace | Inspector` 三栏工作台, 并允许 Inspector 在窄窗口中折叠. Workbench implementation MUST 按 workflow 组件和 data hooks 划分职责, 避免单个入口组件长期承载 Gallery, Albums, Review, Queue, Settings, Inspector 和 IPC orchestration 的全部逻辑. Settings workspace SHALL 按 `Libraries` 和 `Logs` 子页拆分 Library lifecycle management 与 diagnostics.

#### Scenario: 选择 Gallery 图片

- **WHEN** 用户在 Gallery 中选择一个 asset
- **THEN** Workspace 保持图片网格上下文, Inspector 展示该 asset 的 metadata, prompt, tags, albums 和 versions

#### Scenario: Workflow 边界清晰

- **WHEN** 开发者维护 desktop workbench
- **THEN** Gallery, Albums, Review, Task, Settings 和 Inspector 的主要 rendering 与 data orchestration 位于职责明确的组件或 hooks 中

#### Scenario: Settings 边界清晰

- **WHEN** 开发者维护 Settings workspace
- **THEN** Library lifecycle management 和 Logs diagnostics 位于职责明确的 Settings 子组件或 hooks 中
