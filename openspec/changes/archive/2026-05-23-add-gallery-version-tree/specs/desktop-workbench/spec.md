## ADDED Requirements

### Requirement: Inspector 支持 Version Tree Navigation

桌面应用 SHALL 在 Inspector 中展示 selected asset 的 version tree. Version tree MUST 支持鼠标和键盘选择. 选择 tree node 后, Inspector preview, file context, generation event, lineage 和 action target MUST 聚焦该 version.

#### Scenario: 点击 Version Tree Node

- **WHEN** 用户点击 Inspector version tree 中的某个 child version
- **THEN** 桌面应用将 `focusedVersionId` 更新为该 version id
- **AND** Inspector preview 和 metadata 切换到该 version
- **AND** Gallery selected asset id 不改变

#### Scenario: Tree Keyboard Navigation

- **WHEN** keyboard focus 位于 version tree
- **THEN** Arrow up/down 在 visible nodes 间移动
- **AND** Enter 选择当前 focused tree node
- **AND** Left/Right 支持 collapse / expand 或 parent navigation

### Requirement: Focused Version Actions

桌面应用 SHALL 基于 Inspector focused version 提供 actions. `Generate variation` MUST 使用 focused version id. `Promote as new asset` MUST 创建新的 Gallery asset, 成功后刷新 Gallery 并选中新 asset.

#### Scenario: Generate Variation 使用 Focused Version

- **WHEN** 用户在 Inspector 聚焦 `v1.1`
- **AND** 点击 `Generate variation`
- **THEN** Composer 使用 `v1.1` 对应的 asset version UUID 作为 `input_version_id`

#### Scenario: Promote 成功后选中新 Asset

- **WHEN** 用户点击 `Promote as new asset`
- **AND** core 成功创建新 asset
- **THEN** Gallery 刷新并展示新 asset card
- **AND** 桌面端选中新 asset
- **AND** Inspector 显示新 asset root `v1` 和 `Promoted from` summary

### Requirement: Gallery Card 展示 Tree Summary

桌面应用 SHALL 在 Gallery card 中展示当前或首选 version 的 tree label 和 tree summary, 但 MUST NOT 将同一 asset 的所有 child versions 展开成独立 card.

#### Scenario: 多分支 Asset Card

- **WHEN** asset 有多个 child versions
- **THEN** Gallery 仍展示一个 asset card
- **AND** card 展示当前 tree label 和 version / branch summary
