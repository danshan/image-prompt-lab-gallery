## MODIFIED Requirements

### Requirement: 搜索资产

系统 SHALL 支持按文本, tag, rating, provider, date, status 和 category 搜索 asset. 文本搜索 SHALL 至少匹配 canonical title, generation prompt 和 tags.

#### Scenario: 按标签和评分搜索

- **WHEN** 用户搜索指定 tag 且 rating 大于等于 4 的资产
- **THEN** 系统返回同时满足两个条件的 asset 列表

#### Scenario: 按 Prompt 文本搜索

- **WHEN** 用户搜索某个已记录 generation prompt 中的文本片段
- **THEN** 系统返回包含该 prompt 片段的生成 asset

### Requirement: 提供 Gallery Query

系统 SHALL 提供 core 定义的 Gallery query, 支持按 text, providers, min rating, review status, tags, album 和 sort 查询 asset card read model. Gallery text query SHALL 匹配 asset title, generation prompt, provider, model label 和 tags. Gallery asset card read model SHALL 返回当前版本可追溯到的 provider, model label, tags 和 image path.

#### Scenario: 按文本和标签查询 Gallery

- **WHEN** 用户输入搜索文本并选择一个或多个 tags
- **THEN** 系统返回同时满足文本和 tags 条件的 Gallery asset 列表

#### Scenario: 按评分和 Provider 查询 Gallery

- **WHEN** 用户选择 min rating 和 provider filter
- **THEN** 系统返回同时满足评分下限和 provider 条件的 Gallery asset 列表

#### Scenario: 按 Prompt 查询 Gallery

- **WHEN** 用户输入某个 generation prompt 中的文本片段
- **THEN** 系统返回 prompt 包含该片段的生成 asset, 且 card read model 包含 provider 和 model label

#### Scenario: 查询带 Tag 的 Gallery

- **WHEN** 用户选择一个 tag filter
- **THEN** 系统只返回包含该 tag 的 asset, 且每个返回项包含其 tags 列表
