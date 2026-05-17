## MODIFIED Requirements

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
