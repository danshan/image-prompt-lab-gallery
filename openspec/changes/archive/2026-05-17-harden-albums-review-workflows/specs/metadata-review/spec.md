## ADDED Requirements

### Requirement: Review Inbox 支持选择和查看 Suggestion
系统 SHALL 在 Review Inbox 中展示 pending metadata suggestions, 并允许用户选择单条 suggestion 查看可审核字段.

#### Scenario: 查看 Pending Suggestion
- **WHEN** 用户在 Review Inbox 中选择一条 pending suggestion
- **THEN** 系统展示该 suggestion 的 suggested title, description, schema prompt, tags, category, status 和可用上下文

#### Scenario: 无 Pending Suggestions
- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review Inbox 展示明确 empty state, 且不保留旧 suggestion detail

### Requirement: Review 表单本地编辑后接受
系统 SHALL 允许用户在 Review UI 中本地编辑 suggestion 字段, 并在接受时将最终确认值写入 canonical asset metadata.

#### Scenario: 编辑后接受 Suggestion
- **WHEN** 用户修改 pending suggestion 的 title, description, schema prompt, tags 或 category 后点击接受
- **THEN** 系统将最终确认值写入 canonical asset metadata, 创建 confirmed tags, 保存 schema prompt, 并将该 suggestion 标记为 accepted

#### Scenario: 接受失败保留编辑状态
- **WHEN** 用户接受 suggestion 时写入失败
- **THEN** Review UI 保留用户当前编辑内容, 并展示可恢复错误

### Requirement: Review 表单支持恢复初始建议
系统 SHALL 在 Review UI 中提供恢复操作, 将当前本地编辑恢复为选择 suggestion 时的初始建议值, 且不得修改 suggestion 状态或 canonical asset metadata.

#### Scenario: 恢复 Suggestion 表单
- **WHEN** 用户编辑 pending suggestion 后点击恢复
- **THEN** Review UI 将 title, description, schema prompt, tags 和 category 恢复为该 suggestion 的初始值, suggestion 仍保持 pending review

### Requirement: Review 字段支持重新生成
系统 SHALL 允许用户对 title, description 和 schema prompt 分别触发重新生成, 并将生成结果写入本地 Review 表单草稿, 接受前不得写入 canonical asset metadata.

#### Scenario: 重新生成 Review 字段
- **WHEN** 用户在 Review UI 中点击 title, description 或 schema prompt 的重新生成按钮
- **THEN** 系统基于当前 suggestion 和可用 asset 上下文更新对应表单字段, 且 suggestion 仍保持 pending review

### Requirement: Gallery Asset 支持重新进入 Review
系统 SHALL 允许用户从 Gallery 对已有 asset 重新创建 pending metadata suggestion, 使该 asset 回到 Review Inbox 并显示 review pending state.

#### Scenario: Gallery 重新 Review Asset
- **WHEN** 用户在 Gallery 中对一个没有 pending suggestion 的 asset 触发重新 review
- **THEN** 系统为该 asset 创建 pending metadata suggestion, Review Inbox 显示该 suggestion, 且该 asset 显示 Review pending

### Requirement: Category 只能从已有值选择
系统 MUST 将 category 作为单选归类字段处理. Review 接受 suggestion 时, category 只能使用当前 library 已存在的 category 值或为空, 不得因为 AI suggestion 或用户输入而自动创建新 category.

#### Scenario: 接受已有 Category
- **WHEN** 用户在 Review UI 中从已有 category 下拉列表选择一个 category 后接受
- **THEN** 系统将该 category 写入 canonical asset metadata

#### Scenario: 没有合适 Category
- **WHEN** AI suggestion 没有匹配到已有 category 或用户不选择 category
- **THEN** 系统保持 category 为空或保留原值, 不自动创建新 category

### Requirement: Tags 支持快速添加和自动补全
系统 SHALL 允许 Review UI 通过 chip input 管理 tags. 用户输入已有 tag 时应自动补全并添加, 输入不存在的 tag 时可以创建新的 tag chip, 接受 suggestion 时再写入 confirmed tags.

#### Scenario: 添加已有 Tag
- **WHEN** 用户在 Review tag input 中输入已有 tag 并确认
- **THEN** Review UI 将该 tag 作为 chip 添加, 且不重复添加同名 tag

#### Scenario: 添加新 Tag
- **WHEN** 用户在 Review tag input 中输入不存在的 tag 并确认
- **THEN** Review UI 将其作为新 tag chip 添加, 接受 suggestion 时由 core 创建 confirmed tag
