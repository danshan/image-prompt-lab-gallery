## ADDED Requirements

### Requirement: Generation 完成后进入 Review 队列但不强制导航
系统 SHALL 在生成完成后将生成结果展示在 Gallery, 并在存在 metadata suggestion 时将 suggestion 加入 Review Inbox. 桌面端 MUST NOT 因生成完成而强制从当前 workflow 跳转到 Review Inbox.

#### Scenario: 生成完成后保持当前 Workflow
- **WHEN** 用户发起 Generate Image 且生成成功
- **THEN** 系统创建 asset/version 和 generation event, Gallery 可展示生成结果, 桌面端保持用户当前 workflow

#### Scenario: 生成产生 Pending Suggestion
- **WHEN** 生成成功并创建 metadata suggestion
- **THEN** 该 suggestion 出现在 Review Inbox, Review badge 更新, 可包含 title 和 JSON schema prompt draft, 且 asset detail 可展示 review pending state

#### Scenario: Factual Generation Metadata 立即可见
- **WHEN** 用户选择新生成的 asset
- **THEN** Gallery 或 Inspector 可展示 provider, model, prompt, parameters, lineage 和 file metadata, 但 pending suggestion 的 title, schema prompt, tags, description 和 category 不得在接受前作为 canonical metadata 展示
