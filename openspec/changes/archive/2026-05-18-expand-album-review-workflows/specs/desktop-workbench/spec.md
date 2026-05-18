## MODIFIED Requirements

### Requirement: 提供 Gallery 和 Albums 视图

桌面应用 SHALL 展示导入和生成的图片, 并支持进入 manual album 和 smart album 视图. Albums 视图 SHALL 支持 album list 排序, album rename/delete, manual album item 排序, 从 manual album 移除 asset, 批量添加 selected assets, 以及 Smart Album builder.

#### Scenario: 打开智能相册

- **WHEN** 用户打开一个 smart album
- **THEN** Workspace 展示当前满足 smart query 的 asset 列表

#### Scenario: 打开 Albums 管理视图

- **WHEN** 用户打开 Albums 视图
- **THEN** Workspace 展示 album list, album detail 区域, 并提供 manual album 和 smart album 管理入口

### Requirement: Albums Workspace 使用真实 Album 数据

桌面应用 SHALL 提供 Albums workspace, 展示真实 albums 列表, 支持创建 manual album, 打开 album detail, 并复用 Gallery card 展示 album 内容. Albums workspace SHALL 支持 album list drag reorder, rename, delete, manual album item drag reorder, remove asset, batch add selected assets, and Smart Album builder.

#### Scenario: 查看 Albums Workspace

- **WHEN** 用户打开 Albums workspace
- **THEN** 桌面应用展示当前 library 的 album list, 每项包含 name, kind 和 item count

#### Scenario: 创建 Manual Album

- **WHEN** 用户在 Albums workspace 输入 name 并创建 manual album
- **THEN** 桌面应用通过 Rust core 创建 album, 刷新 album list, 并展示新 album

#### Scenario: 打开 Album Detail

- **WHEN** 用户点击一个 manual album
- **THEN** Workspace 展示该 album 的 header 和 album-scoped Gallery cards

#### Scenario: 拖拽排序 Album List

- **WHEN** 用户在 Albums workspace 拖拽调整 album list 顺序
- **THEN** 桌面应用调用 Rust core 保存 album sort order, 并按保存后的顺序渲染 album list

#### Scenario: 重命名 Album

- **WHEN** 用户在 Albums workspace 重命名 album
- **THEN** 桌面应用调用 Rust core 更新 album name, 刷新 album list 和当前 album detail header

#### Scenario: 删除 Album

- **WHEN** 用户在 Albums workspace 删除 album
- **THEN** 桌面应用调用 Rust core 删除 album, 清理当前 selected album, 并刷新 album list 和 Gallery query

#### Scenario: 拖拽排序 Manual Album Assets

- **WHEN** 用户在 manual album detail 中拖拽 asset cards
- **THEN** 桌面应用调用 Rust core 保存 album item sort order, 并按 album order 重新渲染该 album

#### Scenario: 从 Manual Album 移除 Asset

- **WHEN** 用户在 manual album detail 中移除某个 asset
- **THEN** 桌面应用调用 Rust core 删除 membership, 刷新 album detail 和 album list item count

#### Scenario: 批量添加 Gallery Assets 到 Manual Album

- **WHEN** 用户选择多个 Gallery assets 并添加到 manual album
- **THEN** 桌面应用调用 Rust core 批量写入 memberships, 刷新 album list, Gallery 和受影响 asset detail

#### Scenario: 构建 Smart Album

- **WHEN** 用户在 Smart Album builder 中设置 text, tags, providers, min rating, review status, category, status, created date range 或 sort
- **THEN** 桌面应用将 typed query 提交给 Rust core validation, 并展示满足 query 的 live preview

### Requirement: 提供 Review Inbox

桌面应用 SHALL 提供 Review Inbox, 用于处理 pending metadata suggestions. Review Inbox SHALL 支持单选查看, 多选 batch actions, batch accept/reject, suggestion history compare, full suggestion regeneration, confidence visualization, 以及将当前或选中 suggestions 对应 assets 加入 manual album.

#### Scenario: 接受 Suggestion

- **WHEN** 用户在 Review Inbox 接受某条 suggestion
- **THEN** 应用调用 Rust core 写入 canonical metadata, 并从 pending 列表中移除该 suggestion

#### Scenario: 批量接受 Suggestions

- **WHEN** 用户在 Review Inbox 选择多条 pending suggestions 并点击批量接受
- **THEN** 桌面应用将当前打开 suggestion 的 draft 和其他选中 suggestions 的 persisted values 作为 final payloads 提交给 Rust core

#### Scenario: 批量拒绝 Suggestions

- **WHEN** 用户在 Review Inbox 选择多条 pending suggestions 并点击批量拒绝
- **THEN** 桌面应用调用 Rust core 批量标记 rejected, 并刷新 pending list, Review badge 和 Gallery

### Requirement: Review Inbox Workspace 支持 Editable Detail

桌面应用 SHALL 提供 Review Inbox workspace, 包含 pending suggestion list 和 selected suggestion editable detail form. Review 表单 SHALL 支持 title, description 和 JSON schema prompt 的字段级 Codex CLI 重新生成, 并在字段生成期间展示 loading 状态. Review detail SHALL 展示 suggestion history, 支持从 history pick 字段值到本地 draft, 支持 full suggestion regeneration, 并展示 confidence score.

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

#### Scenario: 展示 Suggestion History

- **WHEN** 用户打开 selected suggestion detail
- **THEN** 桌面应用展示同一 asset 的 suggestion history, 并标识每条 suggestion 的 status, created time 和 reviewed time

#### Scenario: 从 History Pick 字段

- **WHEN** 用户从 suggestion history 中选择某个字段值
- **THEN** 桌面应用只更新当前本地 Review draft, 不调用 Rust core 写入 canonical metadata

#### Scenario: 重新生成完整 Suggestion

- **WHEN** 用户点击 full suggestion regeneration
- **THEN** 桌面应用调用后端生成新的 pending suggestion record, 刷新 history 和 pending list, 并保留当前 draft 的可恢复状态

#### Scenario: 展示 Confidence Score

- **WHEN** selected suggestion 包含可解析 confidence_json
- **THEN** 桌面应用展示 normalized overall score 和字段级 score chips

#### Scenario: Review 中加入 Album

- **WHEN** 用户在 Review 中选择一个 manual album 并执行 Add to Album
- **THEN** 桌面应用将 selected suggestions 对应 assets 或当前 suggestion asset 批量添加到该 album, 并保持 suggestion status 不变
