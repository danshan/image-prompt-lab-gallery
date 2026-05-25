## Purpose

Define review-first metadata suggestion behavior before writing canonical asset metadata.
## Requirements
### Requirement: 创建 AI Metadata Suggestions

系统 SHALL 为导入或生成的 asset 创建 AI metadata suggestions, 包含 title, description, schema prompt, tags, category 和 confidence.

#### Scenario: 新 asset 进入 Review Inbox

- **WHEN** 一个 asset 被导入或生成完成
- **THEN** 系统创建 pending metadata suggestion, 并让该 asset 出现在 Review Inbox

### Requirement: Review-first 写入 Canonical Metadata

系统 MUST NOT 在用户接受或编辑前将 AI suggestion, generated field result 或 local review draft 写入 canonical asset metadata. Review UI 和 read models MUST 显式区分 canonical metadata, pending suggestion, generated result 和 local draft.

#### Scenario: Suggestion 待审核
- **WHEN** AI 返回 tags, title, description, schema prompt 或 category suggestion
- **THEN** 系统只写入 `metadata_suggestions` 或 staged result, 不修改 `assets.title`, confirmed tags, canonical description, category 或 schema prompt

#### Scenario: Review Draft 不写 Canonical Metadata
- **WHEN** 用户编辑 Review draft 但尚未接受 suggestion
- **THEN** 系统不修改 canonical asset metadata, 且 Gallery/Inspector 中 canonical 字段仍保持原值

#### Scenario: 接受后写入 Canonical Metadata
- **WHEN** 用户接受 Review draft
- **THEN** 系统将最终确认值写入 canonical asset metadata, 并将 suggestion 标记为 accepted

### Requirement: Review 表单本地编辑后接受

系统 SHALL 允许用户在 Review UI 中本地编辑 suggestion 字段, 并在接受时将最终确认值写入 canonical asset metadata.

#### Scenario: 用户编辑后接受

- **WHEN** 用户修改 pending suggestion 的 title, description, schema prompt, tags 或 category 后点击接受
- **THEN** 系统将最终确认值写入 canonical asset metadata, 创建 confirmed tags, 保存 schema prompt, 并将该 suggestion 标记为 accepted

#### Scenario: 接受失败保留编辑状态

- **WHEN** 用户接受 suggestion 时写入失败
- **THEN** Review UI 保留用户当前编辑内容, 并展示可恢复错误

### Requirement: Review Inbox 支持选择和查看 Suggestion

系统 SHALL 在 Review Inbox 中展示 pending metadata suggestions, 允许用户选择单条 suggestion 查看可审核字段, 并支持多选 suggestions 以执行 batch actions.

#### Scenario: 查看 Pending Suggestion

- **WHEN** 用户在 Review Inbox 中选择一条 pending suggestion
- **THEN** 系统展示该 suggestion 的 suggested title, description, schema prompt, tags, category, status 和可用上下文

#### Scenario: 无 Pending Suggestions

- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review Inbox 展示明确 empty state, 且不保留旧 suggestion detail

#### Scenario: 多选 Pending Suggestions

- **WHEN** 用户在 Review Inbox 中选择多条 pending suggestions
- **THEN** 系统保持 selected suggestion ids, 并允许用户执行 batch accept, batch reject 或 add selected assets to album

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

系统 SHALL 在重新生成 metadata suggestions 时创建新记录, 不覆盖既有 review history. 系统 SHALL 允许读取同一 asset 的 suggestion history, 并保留 pending, accepted 和 rejected suggestions 的 status 与 reviewed time.

#### Scenario: 重新生成建议

- **WHEN** 用户对同一个 asset 重新请求 AI metadata suggestion
- **THEN** 系统创建新的 suggestion record, 并保留旧 suggestion 的状态和 reviewed time

#### Scenario: 读取 History

- **WHEN** 用户打开某个 asset 的 suggestion history
- **THEN** 系统返回该 asset 的全部 metadata suggestions, 且每条 suggestion 包含 status, confidence 和 reviewed time

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

系统 MUST 在 apply metadata field generation result 前检查 suggestion id, field 和 base revision, 防止异步结果覆盖用户后续编辑. Stale result MUST 作为 generated result available 展示, 并允许用户显式 apply 或 ignore.

#### Scenario: Apply Current Field Result
- **WHEN** field generation task completed 且当前 Review draft 仍匹配 task 的 suggestion id, field 和 base revision
- **THEN** Review UI 可以将生成结果写入本地 draft, 且 suggestion 仍保持 pending review

#### Scenario: Preserve User Edits For Stale Result
- **WHEN** field generation task completed 但当前 Review draft 不匹配 task 的 base revision
- **THEN** Review UI 保留用户当前编辑, 显示 generated result available, 并允许用户显式查看, apply 或 ignore 该结果

#### Scenario: Ignore Stale Result
- **WHEN** 用户忽略 stale generated result
- **THEN** Review UI 移除或收起该 generated result, 不修改当前 draft, suggestion record 或 canonical metadata

### Requirement: Review Task State 可追溯

系统 SHALL 允许 Review Inbox 从 metadata generation 状态跳转到对应 task detail, 并从 task output 回到对应 suggestion.

#### Scenario: Open Task From Review Field

- **WHEN** Review field generation task running, retry waiting 或 failed
- **THEN** Review Inbox 显示 Open task detail 入口, 用户可查看 attempts, timeline 和 logs

#### Scenario: Open Suggestion From Task

- **WHEN** metadata suggestion generation task completed
- **THEN** Task Detail 显示 Open review suggestion link, 用户可跳转到新 pending suggestion

### Requirement: 批量接受和拒绝 Suggestions

系统 SHALL 支持批量接受和批量拒绝 pending metadata suggestions. 批量接受 MUST 接收每条 suggestion 的最终审核 payload, 并在写入前验证所有 suggestions 仍为 pending. 批量操作 MUST 以事务方式执行, 不得产生部分成功.

#### Scenario: 批量接受 Suggestions

- **WHEN** 用户选择多条 pending suggestions 并提交每条 suggestion 的最终 title, description, schema prompt, tags 和 category
- **THEN** 系统在单个事务中写入 canonical metadata, 创建 confirmed tags, 并将这些 suggestions 标记为 accepted

#### Scenario: 批量接受包含非 Pending Suggestion

- **WHEN** 用户批量接受的任意 suggestion 已不处于 pending review 状态
- **THEN** 系统返回可恢复错误, 且不写入任何 canonical metadata 或 suggestion status

#### Scenario: 批量拒绝 Suggestions

- **WHEN** 用户选择多条 pending suggestions 并执行批量拒绝
- **THEN** 系统在单个事务中将这些 suggestions 标记为 rejected, 且不修改 canonical asset metadata

### Requirement: Suggestion History 对比和字段合并

系统 SHALL 支持按 asset 查询 metadata suggestion history, 包含 pending, accepted 和 rejected suggestions. Review UI SHALL 允许用户比较同一 asset 的多个 suggestions, 并将任意 history row 的字段值复制到当前本地 Review draft. 字段合并 MUST NOT 修改 suggestion record 或 canonical asset metadata.

#### Scenario: 查询 Asset Suggestion History

- **WHEN** 用户在 Review 中打开某个 asset 的 suggestion detail
- **THEN** 系统返回该 asset 的 suggestion history, 包含 status, created time, reviewed time, suggested fields, tags, category 和 confidence

#### Scenario: 从 History 选择字段值

- **WHEN** 用户从 suggestion history 中选择 title, description, schema prompt, tags 或 category 字段值
- **THEN** Review UI 将该字段值写入当前本地 draft, 且不调用 core 写入 suggestion 或 canonical metadata

### Requirement: Full Suggestion Regeneration

系统 SHALL 支持为已有 asset 重新生成完整 metadata suggestion record. Full suggestion regeneration MUST 创建新的 pending suggestion, 并保留既有 suggestion history. 字段级 regeneration 仍只更新本地 draft, 不创建 suggestion record.

#### Scenario: 重新生成完整 Suggestion

- **WHEN** 用户对某个 asset 触发 full suggestion regeneration 且 AI metadata generation 成功
- **THEN** 系统创建新的 pending metadata suggestion, 刷新 suggestion history, 且不覆盖旧 suggestions

#### Scenario: 完整 Suggestion 重新生成失败

- **WHEN** AI metadata generation 失败或返回无法保存的 suggestion payload
- **THEN** 系统保留当前 draft 和既有 suggestion history, 展示可恢复错误

### Requirement: Confidence 可视化评分模型

系统 SHALL 支持将 suggestion 的 `confidence_json` 规范化为可展示评分模型. 评分模型 SHALL 支持 overall 分数和字段级 title, description, schemaPrompt, tags, category 分数. 分数可使用 `0..1` 或 `0..100`, UI MUST normalize 为 `0..100` 展示. 缺失或非法 confidence 值 MUST 显示为 unknown, 且不得阻塞 review 操作.

#### Scenario: 展示 Overall 和字段级 Confidence

- **WHEN** pending suggestion 的 confidence_json 包含 overall 和字段级分数
- **THEN** Review UI 展示 normalized overall score 和字段级 confidence chips

#### Scenario: Confidence JSON 不合法

- **WHEN** suggestion 的 confidence_json 无法解析或包含非法分数
- **THEN** Review UI 将对应分数显示为 unknown, 且用户仍可接受或拒绝 suggestion

### Requirement: Review 中加入 Album

系统 SHALL 允许用户从 Review workflow 将当前或选中的 suggestions 对应 assets 加入 manual album. 当存在 selected suggestions 时, 操作作用于这些 suggestions 的 asset ids. 当没有 selected suggestions 时, 操作作用于当前 suggestion 的 asset id.

#### Scenario: 将选中 Review Assets 加入 Album

- **WHEN** 用户在 Review 中选择多条 suggestions 并选择一个 manual album 执行 Add to Album
- **THEN** 系统将这些 suggestions 对应 assets 批量添加到该 manual album, 并保持 suggestions 的 review status 不变

#### Scenario: 没有批量选择时加入 Album

- **WHEN** 用户在 Review detail 中未选择批量 items 但执行 Add to Album
- **THEN** 系统将当前 suggestion 对应 asset 添加到目标 manual album, 并保持 suggestion 的 review status 不变

### Requirement: ReviewDraftDetail Read Model

系统 SHALL 提供 Review draft detail read model, 用于 Studio Console Review workspace. Read model MUST 包含 suggestion, asset context, local draft seed, confidence, suggestion history, generated field results, related tasks 和 canonical metadata summary.

#### Scenario: 加载 Review Draft Detail
- **WHEN** 用户选择 pending suggestion
- **THEN** 系统返回该 suggestion 的 staged fields, canonical metadata summary, draft seed, confidence, history, related tasks 和 asset context

#### Scenario: Canonical 和 Staged 字段并存
- **WHEN** pending suggestion 字段与 canonical metadata 字段不同
- **THEN** read model 同时返回 canonical value 和 staged value, UI 可以明确展示差异

### Requirement: Suggestion History Compare

系统 SHALL 支持 Review UI 比较同一 asset 的 suggestion history, 并允许用户将任意 history row 的字段值复制到当前 local draft. 复制字段值 MUST NOT 修改 suggestion record 或 canonical asset metadata.

#### Scenario: 查看 History Diff
- **WHEN** 用户打开 suggestion history
- **THEN** Review UI 展示 pending, accepted 和 rejected suggestions 的字段值, status, confidence, created time 和 reviewed time

#### Scenario: 从 History 应用字段到 Draft
- **WHEN** 用户从 history row 选择 title, description, schema prompt, tags 或 category
- **THEN** Review UI 将该字段值写入当前 local draft, 不调用 core 写入 canonical metadata

### Requirement: Review Workspace 状态覆盖

Review workspace SHALL 覆盖 pending list, selected detail, generated results, batch actions 和 task mirror 的 loading, empty, error 和 recovery states. Accept failure MUST 保留当前 local draft. Pending suggestions list SHALL remain vertically scrollable inside the Review Inbox panel when its content exceeds the available desktop workflow height, while header and batch controls remain reachable above the list. Review metadata selected detail SHALL remain vertically scrollable inside the desktop workflow surface when its content exceeds the available viewport height, so fields, actions 和 suggestion history remain reachable.

#### Scenario: Accept Failure Preserves Draft
- **WHEN** 用户接受 Review draft 时写入失败
- **THEN** Review UI 保留用户当前 draft, 展示可恢复错误, 并允许用户重试或继续编辑

#### Scenario: No Pending Suggestions
- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review workspace 展示 empty state, 清理旧 selected suggestion detail, 并保留其他 workflow context

#### Scenario: Related Task Failed
- **WHEN** Review field generation 相关 task failed
- **THEN** Review workspace 展示 task failure summary 和 Open task detail 入口, 不覆盖当前 draft

#### Scenario: Pending Review List Scrolls
- **WHEN** Review Inbox pending suggestions exceed the available height of the left panel
- **THEN** Review UI allows vertical scrolling within the pending suggestions list
- **AND** batch actions and add-to-album controls remain reachable above the list

#### Scenario: Review Metadata Detail Scrolls
- **WHEN** Review metadata detail content exceeds the available desktop workflow height
- **THEN** Review UI allows vertical scrolling within the selected detail area
- **AND** title, category, description, schema prompt, tags, review actions 和 suggestion history remain reachable without horizontal scrolling
