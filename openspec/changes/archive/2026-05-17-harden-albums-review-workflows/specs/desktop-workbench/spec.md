## ADDED Requirements

### Requirement: Albums Workspace 使用真实 Album 数据
桌面应用 SHALL 提供 Albums workspace, 展示真实 albums 列表, 支持创建 manual album, 打开 album detail, 并复用 Gallery card 展示 album 内容.

#### Scenario: 查看 Albums Workspace
- **WHEN** 用户打开 Albums workspace
- **THEN** 桌面应用展示当前 library 的 album list, 每项包含 name, kind 和 item count

#### Scenario: 创建 Manual Album
- **WHEN** 用户在 Albums workspace 输入 name 并创建 manual album
- **THEN** 桌面应用通过 Rust core 创建 album, 刷新 album list, 并展示新 album

#### Scenario: 打开 Album Detail
- **WHEN** 用户点击一个 manual album
- **THEN** Workspace 展示该 album 的 header 和 album-scoped Gallery cards

### Requirement: Inspector 支持 Album Membership 和 Add To Album
桌面应用 SHALL 在 Inspector 中展示当前 asset 的 album memberships, 并允许用户将当前 asset 添加到已有 manual album.

#### Scenario: 查看 Asset Album Membership
- **WHEN** 用户选择一个 asset
- **THEN** Inspector 展示该 asset 当前所属 albums

#### Scenario: 添加 Asset 到 Album
- **WHEN** 用户在 Inspector 中选择一个 manual album 并确认添加
- **THEN** 桌面应用调用 Rust core 写入 membership, 并刷新 album list, 当前 Gallery query 和当前 asset detail

### Requirement: Review Inbox Workspace 支持 Editable Detail
桌面应用 SHALL 提供 Review Inbox workspace, 包含 pending suggestion list 和 selected suggestion editable detail form.

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
- **THEN** 桌面应用更新对应本地 form 字段, 不修改 pending suggestion 状态

#### Scenario: Gallery 发起 Re-review
- **WHEN** 用户在 Gallery asset card 中点击重新 review
- **THEN** 桌面应用调用 Rust core 创建 pending suggestion, 刷新 Review badge, Gallery 和受影响的 Inspector detail

### Requirement: Library Switch 清理 Albums 和 Review State
桌面应用 MUST 在切换 library 时清理 selected album, selected suggestion, editable review form 和 stale Inspector selection.

#### Scenario: 切换 Library
- **WHEN** 用户从 Sidebar 切换到另一个 library
- **THEN** 桌面应用清空旧 library 的 selected album, selected suggestion, review form 和 Inspector selection, 并使用新 library 重新加载数据
