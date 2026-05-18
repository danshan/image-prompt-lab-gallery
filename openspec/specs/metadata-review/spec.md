## Purpose

Define review-first metadata suggestion behavior before writing canonical asset metadata.

## Requirements

### Requirement: 创建 AI Metadata Suggestions

系统 SHALL 为导入或生成的 asset 创建 AI metadata suggestions, 包含 title, description, schema prompt, tags, category 和 confidence.

#### Scenario: 新 asset 进入 Review Inbox

- **WHEN** 一个 asset 被导入或生成完成
- **THEN** 系统创建 pending metadata suggestion, 并让该 asset 出现在 Review Inbox

### Requirement: Review-first 写入 Canonical Metadata

系统 MUST NOT 在用户接受或编辑前将 AI suggestion 写入 canonical asset metadata.

#### Scenario: Suggestion 待审核

- **WHEN** AI 返回 tags 和 title suggestion
- **THEN** 系统只写入 `metadata_suggestions`, 不修改 `assets.title` 或 confirmed tags

### Requirement: Review 表单本地编辑后接受

系统 SHALL 允许用户在 Review UI 中本地编辑 suggestion 字段, 并在接受时将最终确认值写入 canonical asset metadata.

#### Scenario: 用户编辑后接受

- **WHEN** 用户修改 pending suggestion 的 title, description, schema prompt, tags 或 category 后点击接受
- **THEN** 系统将最终确认值写入 canonical asset metadata, 创建 confirmed tags, 保存 schema prompt, 并将该 suggestion 标记为 accepted

#### Scenario: 接受失败保留编辑状态

- **WHEN** 用户接受 suggestion 时写入失败
- **THEN** Review UI 保留用户当前编辑内容, 并展示可恢复错误

### Requirement: Review Inbox 支持选择和查看 Suggestion

系统 SHALL 在 Review Inbox 中展示 pending metadata suggestions, 并允许用户选择单条 suggestion 查看可审核字段.

#### Scenario: 查看 Pending Suggestion

- **WHEN** 用户在 Review Inbox 中选择一条 pending suggestion
- **THEN** 系统展示该 suggestion 的 suggested title, description, schema prompt, tags, category, status 和可用上下文

#### Scenario: 无 Pending Suggestions

- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review Inbox 展示明确 empty state, 且不保留旧 suggestion detail

### Requirement: Review 表单支持恢复初始建议

系统 SHALL 在 Review UI 中提供恢复操作, 将当前本地编辑恢复为选择 suggestion 时的初始建议值, 且不得修改 suggestion 状态或 canonical asset metadata.

#### Scenario: 恢复 Suggestion 表单

- **WHEN** 用户编辑 pending suggestion 后点击恢复
- **THEN** Review UI 将 title, description, schema prompt, tags 和 category 恢复为该 suggestion 的初始值, suggestion 仍保持 pending review

### Requirement: Review 字段支持重新生成

系统 SHALL 允许用户对 title, description 和 schema prompt 分别触发重新生成, 并通过 Codex CLI 生成对应字段结果. 生成结果 SHALL 写入本地 Review 表单草稿, 接受前不得写入 canonical asset metadata. title 和 description 生成结果 SHALL 以简体中文为主. schema prompt 生成结果 MUST 是可解析为 JSON object 的 JSON Schema Prompt.

#### Scenario: 重新生成 Review 字段

- **WHEN** 用户在 Review UI 中点击 title, description 或 schema prompt 的重新生成按钮
- **THEN** 系统通过 Codex CLI 基于当前 suggestion 和可用 asset 上下文生成对应字段, 更新本地表单草稿, 且 suggestion 仍保持 pending review

#### Scenario: 重新生成中文 Title

- **WHEN** 用户点击 title 的重新生成按钮且 Codex CLI 成功返回结果
- **THEN** Review 表单 title 更新为一个简体中文标题, 且系统不修改 canonical asset metadata

#### Scenario: 重新生成中文 Description

- **WHEN** 用户点击 description 的重新生成按钮且 Codex CLI 成功返回结果
- **THEN** Review 表单 description 更新为简体中文描述, 且系统不修改 canonical asset metadata

#### Scenario: 重新生成 JSON Schema Prompt

- **WHEN** 用户点击 JSON Schema Prompt 的重新生成按钮且 Codex CLI 返回合法 JSON object
- **THEN** Review 表单 schema prompt 更新为格式化后的 JSON 文本, 且系统不修改 canonical asset metadata

#### Scenario: Codex 重新生成失败

- **WHEN** Codex CLI 不可用, 返回非零状态, 输出为空, 或 JSON Schema Prompt 输出无法解析
- **THEN** 系统保留用户当前 Review 表单草稿, 展示可恢复错误, 并保留 pending suggestion 状态

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

### Requirement: 保留 Suggestion 历史

系统 SHALL 在重新生成 metadata suggestions 时创建新记录, 不覆盖既有 review history.

#### Scenario: 重新生成建议

- **WHEN** 用户对同一个 asset 重新请求 AI metadata suggestion
- **THEN** 系统创建新的 suggestion record, 并保留旧 suggestion 的状态和 reviewed time

### Requirement: Review Metadata Generation 通过 Task Manager 执行

系统 SHALL 将 Review Inbox 的 field-level metadata generation 和 full suggestion regeneration 作为 daemon tasks 执行, 并保留 review-first 写入语义.

#### Scenario: Enqueue Field Generation Task

- **WHEN** 用户点击 title, description 或 JSON schema prompt 的 regenerate 按钮
- **THEN** 系统创建 `metadata_field_generation` task, input snapshot 包含 suggestion id, field, asset context 和 base revision

#### Scenario: Enqueue Full Suggestion Generation Task

- **WHEN** 用户点击 regenerate suggestion
- **THEN** 系统创建 `metadata_suggestion_generation` task, 并在完成后创建新的 pending suggestion record, 保留旧 suggestion history

#### Scenario: Field Generation Does Not Write Canonical Metadata

- **WHEN** `metadata_field_generation` task completed
- **THEN** 系统只产生 field result 或 review draft patch, 不修改 canonical asset metadata, confirmed tags 或 accepted suggestion state

### Requirement: Field Generation Result 安全 Apply

系统 MUST 在 apply metadata field generation result 前检查 suggestion id, field 和 base revision, 防止异步结果覆盖用户后续编辑.

#### Scenario: Apply Current Field Result

- **WHEN** field generation task completed 且当前 Review form 仍匹配 task 的 suggestion id, field 和 base revision
- **THEN** Review UI 可以将生成结果写入本地 form draft, 且 suggestion 仍保持 pending review

#### Scenario: Preserve User Edits For Stale Result

- **WHEN** field generation task completed 但当前 Review form 不匹配 task 的 base revision
- **THEN** Review UI 保留用户当前编辑, 显示 generated result available, 并允许用户显式查看或应用该结果

#### Scenario: Schema Prompt Result Validation

- **WHEN** metadata field generation task 的目标字段是 JSON schema prompt
- **THEN** daemon 或 core MUST 校验生成结果可解析为 JSON object, 否则将 attempt 标记为 schema validation failure

### Requirement: Review Task State 可追溯

系统 SHALL 允许 Review Inbox 从 metadata generation 状态跳转到对应 task detail, 并从 task output 回到对应 suggestion.

#### Scenario: Open Task From Review Field

- **WHEN** Review field generation task running, retry waiting 或 failed
- **THEN** Review Inbox 显示 Open task detail 入口, 用户可查看 attempts, timeline 和 logs

#### Scenario: Open Suggestion From Task

- **WHEN** metadata suggestion generation task completed
- **THEN** Task Detail 显示 Open review suggestion link, 用户可跳转到新 pending suggestion
