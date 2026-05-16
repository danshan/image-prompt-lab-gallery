## ADDED Requirements

### Requirement: 提供 Asset Detail Read Model
系统 SHALL 提供 asset detail read model, 聚合 asset metadata, 当前 version, generation event, tags, albums, lineage 和 file context.

#### Scenario: 查看 Asset Detail
- **WHEN** 用户选择一个 Gallery asset
- **THEN** 系统返回该 asset 的 prompt, provider/model, tags, albums, versions, lineage 和 file metadata

### Requirement: Gallery Card 展示 Version 摘要
系统 SHALL 在 Gallery card read model 中包含当前或首选 version id, version label 和 version count.

#### Scenario: 展示多版本 Asset
- **WHEN** 一个 asset 包含多个 versions
- **THEN** Gallery card read model 返回 version count 和当前展示 version 的 label 或 id

### Requirement: Lineage Detail 包含 Parent Chain 摘要
系统 SHALL 在 asset detail read model 中返回当前 version 的 parent chain 摘要.

#### Scenario: 查看当前 Version 来源
- **WHEN** 用户在 Inspector 查看当前 version lineage
- **THEN** 系统返回当前 version, parent version 和相关 generation event 摘要
