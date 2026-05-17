## MODIFIED Requirements

### Requirement: Review Inbox Workspace 支持 Editable Detail

桌面应用 SHALL 提供 Review Inbox workspace, 包含 pending suggestion list 和 selected suggestion editable detail form. Review 表单 SHALL 支持 title, description 和 JSON schema prompt 的字段级 Codex CLI 重新生成, 并在字段生成期间展示 loading 状态.

#### Scenario: 选择 Pending Suggestion

- **WHEN** 用户在 Review Inbox 选择一条 pending suggestion
- **THEN** Workspace 展示可编辑的 title, description, JSON schema prompt, tag chips 和单选 category 表单

#### Scenario: 接受 Edited Suggestion

- **WHEN** 用户编辑 suggestion 表单后点击接受
- **THEN** 桌面应用调用 Rust core 写入 canonical metadata, 并刷新 pending list, Review badge, Gallery 和受影响的 Inspector detail

#### Scenario: 快速编辑 Tags

- **WHEN** 用户在 Review tag input 中输入 tag
- **THEN** 桌面应用展示已有 tag 自动补全, Enter 添加 chip, 新 tag 也可作为 chip 添加, 且重复 tag 不会重复显示

#### Scenario: 选择 Category

- **WHEN** 用户在 Review category 控件中选择 category
- **THEN** 桌面应用只允许选择当前 library 已存在 category 或空值, 不提供自动新建 category 行为

#### Scenario: 恢复 Edited Suggestion

- **WHEN** 用户点击恢复 selected suggestion
- **THEN** 桌面应用不调用 Rust core, 仅将当前本地 form state 恢复为 selected suggestion 初始值

#### Scenario: 重新生成 Review 字段

- **WHEN** 用户点击 title, description 或 JSON schema prompt 的重新生成按钮
- **THEN** 桌面应用调用后端 Codex CLI metadata generation command, 展示对应字段 loading 状态, 成功后只更新对应本地 form 字段, 不修改 pending suggestion 状态

#### Scenario: 重新生成期间保留其他字段可编辑

- **WHEN** 用户正在重新生成某一个 Review 字段
- **THEN** 桌面应用只禁用该字段和对应按钮, 其他 Review 字段仍可编辑

#### Scenario: 切换 Suggestion 后忽略旧响应

- **WHEN** 用户触发字段重新生成后切换到另一条 suggestion
- **THEN** 原请求完成时不得覆盖当前 selected suggestion 的 Review 表单内容

#### Scenario: Gallery 发起 Re-review

- **WHEN** 用户在 Gallery asset card 中点击重新 review
- **THEN** 桌面应用调用 Rust core 创建 pending suggestion, 刷新 Review badge, Gallery 和受影响的 Inspector detail

## ADDED Requirements

### Requirement: Settings 展示 App Logs

桌面应用 SHALL 在 Settings 中提供 Logs 模块, 展示最近 app 生成日志, 支持刷新日志列表, 并允许用户查看选中日志内容.

#### Scenario: 查看最近日志

- **WHEN** 用户打开 Settings Logs 模块
- **THEN** 桌面应用展示最近 app-owned Codex image generation 和 metadata generation 日志列表, 包含 kind, path, modified time, size 和 preview

#### Scenario: 刷新日志列表

- **WHEN** 用户点击 Logs 模块的刷新按钮
- **THEN** 桌面应用重新加载最近日志列表, 并在刷新期间展示 loading 状态

#### Scenario: 查看日志内容

- **WHEN** 用户选择某条日志
- **THEN** 桌面应用读取并展示该日志内容预览

#### Scenario: 无日志 Empty State

- **WHEN** 当前没有 app-owned 生成日志
- **THEN** Settings Logs 模块展示明确 empty state
