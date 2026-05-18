## MODIFIED Requirements

### Requirement: 搜索资产
系统 SHALL 支持按文本, tag, rating, provider, date, status 和 category 搜索 asset. 文本搜索 SHALL 至少匹配 canonical title, generation prompt 和 tags. Search read path MUST 避免对每个 asset 重复查询 tags 或 latest generation event.

#### Scenario: 按标签和评分搜索
- **WHEN** 用户搜索指定 tag 且 rating 大于等于 4 的资产
- **THEN** 系统返回同时满足两个条件的 asset 列表

#### Scenario: 按 Prompt 文本搜索
- **WHEN** 用户搜索某个已记录 generation prompt 中的文本片段
- **THEN** 系统返回包含该 prompt 片段的生成 asset

#### Scenario: 搜索避免 Per-Asset 查询
- **WHEN** 系统执行包含 text, tag 或 provider filter 的 search
- **THEN** tags 和 latest generation event 数据通过 SQL filter, batch preload 或 read model 一次性取得, 不得按 asset 循环查询

### Requirement: 提供 Gallery Query
系统 SHALL 提供 core 定义的 Gallery query, 支持按 text, providers, min rating, review status, tags, album 和 sort 查询 asset card read model. Gallery text query SHALL 匹配 asset title, generation prompt, provider, model label 和 tags. Gallery asset card read model SHALL 返回当前版本可追溯到的 provider, model label, tags 和 image path. Gallery read path MUST 避免按 asset 重复查询 current version, generation event, tags, pending review count 或 album membership.

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

#### Scenario: Gallery 查询避免 N+1
- **WHEN** 系统执行 Gallery query 并返回多个 assets
- **THEN** current version, generation event, tags, version count 和 pending review count 必须通过 batch read 或 projection read model 取得, 不得按返回 asset 数量线性增加 SQL round-trip

### Requirement: Gallery Query 支持 Album Detail
系统 SHALL 使用 core Gallery query 的 album filter 展示 album 内容. 打开 manual album 后, Gallery query MUST 只返回该 album 中的 assets. Manual album membership MUST 使用 batch membership set 判断, 不得对每个 asset 执行 membership 查询.

#### Scenario: 打开 Manual Album
- **WHEN** 用户打开一个 manual album
- **THEN** 系统使用该 album id 查询 Gallery, 并返回该 album 中的 asset card read model

#### Scenario: Album 内容为空
- **WHEN** 用户打开一个没有 asset 的 manual album
- **THEN** 系统返回空 Gallery 列表, 且保持 album detail context

#### Scenario: Manual Album Membership 批量加载
- **WHEN** 系统执行带 manual album filter 的 Gallery query
- **THEN** 系统一次性加载该 album 的 asset ids, 并用内存 membership 判断过滤结果

### Requirement: Core 定义 Gallery Sort 语义
系统 MUST 由 core 定义 Gallery sort 语义, 并至少支持 newest, oldest, rating_desc, title_asc, provider_asc 和 album_order. `album_order` MUST 只在 manual album context 中可用, 且排序不得在 comparator 内执行数据库查询.

#### Scenario: 按最新排序
- **WHEN** 用户选择 newest sort
- **THEN** 系统按 asset 或首选 version 的最新相关时间降序返回 Gallery asset 列表

#### Scenario: Sort 值无效
- **WHEN** 用户提交不支持的 sort 值
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

#### Scenario: Manual Album Order 排序
- **WHEN** 用户在 manual album context 中选择 album_order sort
- **THEN** 系统按 album item sort_order 返回 assets, 且 sort_order 数据必须在排序前批量加载
