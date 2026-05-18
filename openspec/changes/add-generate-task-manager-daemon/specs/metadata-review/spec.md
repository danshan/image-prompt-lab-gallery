## ADDED Requirements

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
