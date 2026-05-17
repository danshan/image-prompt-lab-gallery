## Purpose

Define review-first metadata suggestion behavior before writing canonical asset metadata.

## Requirements

### Requirement: 创建 AI Metadata Suggestions

系统 SHALL 为导入或生成的 asset 创建 AI metadata suggestions, 包含 title, description, tags, category 和 confidence.

#### Scenario: 新 asset 进入 Review Inbox

- **WHEN** 一个 asset 被导入或生成完成
- **THEN** 系统创建 pending metadata suggestion, 并让该 asset 出现在 Review Inbox

### Requirement: Review-first 写入 Canonical Metadata

系统 MUST NOT 在用户接受或编辑前将 AI suggestion 写入 canonical asset metadata.

#### Scenario: Suggestion 待审核

- **WHEN** AI 返回 tags 和 title suggestion
- **THEN** 系统只写入 `metadata_suggestions`, 不修改 `assets.title` 或 confirmed tags

### Requirement: 接受, 编辑和拒绝 Suggestion

系统 SHALL 支持用户接受, 编辑或拒绝每条 metadata suggestion.

#### Scenario: 用户编辑后接受

- **WHEN** 用户修改 suggestion 的 title 和 tags 后接受
- **THEN** 系统将编辑后的值写入 canonical metadata, 创建 confirmed tags, 并把 suggestion 标记为 reviewed

### Requirement: 保留 Suggestion 历史

系统 SHALL 在重新生成 metadata suggestions 时创建新记录, 不覆盖既有 review history.

#### Scenario: 重新生成建议

- **WHEN** 用户对同一个 asset 重新请求 AI metadata suggestion
- **THEN** 系统创建新的 suggestion record, 并保留旧 suggestion 的状态和 reviewed time
