## ADDED Requirements

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

## MODIFIED Requirements

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

### Requirement: 保留 Suggestion 历史

系统 SHALL 在重新生成 metadata suggestions 时创建新记录, 不覆盖既有 review history. 系统 SHALL 允许读取同一 asset 的 suggestion history, 并保留 pending, accepted 和 rejected suggestions 的 status 与 reviewed time.

#### Scenario: 重新生成建议

- **WHEN** 用户对同一个 asset 重新请求 AI metadata suggestion
- **THEN** 系统创建新的 suggestion record, 并保留旧 suggestion 的状态和 reviewed time

#### Scenario: 读取 History

- **WHEN** 用户打开某个 asset 的 suggestion history
- **THEN** 系统返回该 asset 的全部 metadata suggestions, 且每条 suggestion 包含 status, confidence 和 reviewed time
