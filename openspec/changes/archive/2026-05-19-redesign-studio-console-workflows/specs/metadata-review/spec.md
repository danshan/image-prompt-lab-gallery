## MODIFIED Requirements

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

## ADDED Requirements

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

Review workspace SHALL 覆盖 pending list, selected detail, generated results, batch actions 和 task mirror 的 loading, empty, error 和 recovery states. Accept failure MUST 保留当前 local draft.

#### Scenario: Accept Failure Preserves Draft
- **WHEN** 用户接受 Review draft 时写入失败
- **THEN** Review UI 保留用户当前 draft, 展示可恢复错误, 并允许用户重试或继续编辑

#### Scenario: No Pending Suggestions
- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review workspace 展示 empty state, 清理旧 selected suggestion detail, 并保留其他 workflow context

#### Scenario: Related Task Failed
- **WHEN** Review field generation 相关 task failed
- **THEN** Review workspace 展示 task failure summary 和 Open task detail 入口, 不覆盖当前 draft
