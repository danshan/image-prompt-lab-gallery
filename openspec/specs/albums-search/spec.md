## Purpose

Define album and search behavior for organizing assets through manual albums, smart albums, tags, ratings, and structured queries.

## Requirements

### Requirement: 管理 Manual Albums

系统 SHALL 支持创建 manual album, 添加和移除 asset, 并维护 album item sort order.

#### Scenario: 添加图片到手动相册

- **WHEN** 用户将 asset 添加到 manual album
- **THEN** 系统创建 album item, 记录 added time, 并按 sort order 返回相册内容

### Requirement: 管理 Smart Albums

系统 SHALL 支持 smart album, 使用受限结构化 query 自动选择 asset.

#### Scenario: 评分智能相册

- **WHEN** 用户创建条件为 rating 大于等于 4 的 smart album
- **THEN** 系统返回所有满足条件的 asset, 且无需写入 album item

### Requirement: 限制 Smart Query 字段

系统 MUST 将 smart album query 限制在 tags, rating, provider, date, status 和 category 等稳定字段.

#### Scenario: 查询字段不支持

- **WHEN** 用户创建包含未允许字段的 smart album query
- **THEN** 系统返回 `InvalidSmartAlbumQuery` 且不保存该 smart album

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

### Requirement: 提供 Album List Read Model

系统 SHALL 提供 albums 列表 read model, 用于桌面端展示当前 library 的 albums, 至少包含 album id, name, kind 和 item count.

#### Scenario: 列出 Manual Albums

- **WHEN** 桌面端请求当前 library 的 albums 列表
- **THEN** 系统返回该 library 中可见 albums 的 id, name, kind 和 item count

#### Scenario: 空 Album 列表

- **WHEN** 当前 library 尚未创建 album
- **THEN** 系统返回空列表, 且不伪造默认 album

### Requirement: Gallery Query 支持 Album Detail

系统 SHALL 使用 core Gallery query 的 album filter 展示 album 内容. 打开 manual album 后, Gallery query MUST 只返回该 album 中的 assets.

#### Scenario: 打开 Manual Album

- **WHEN** 用户打开一个 manual album
- **THEN** 系统使用该 album id 查询 Gallery, 并返回该 album 中的 asset card read model

#### Scenario: Album 内容为空

- **WHEN** 用户打开一个没有 asset 的 manual album
- **THEN** 系统返回空 Gallery 列表, 且保持 album detail context

### Requirement: 添加 Asset 到 Manual Album 后可查询

系统 SHALL 支持将当前 asset 添加到 manual album, 并在写入后通过 album-scoped Gallery query 查询到该 asset.

#### Scenario: 添加当前 Asset 到 Album

- **WHEN** 用户从 Inspector 将当前 asset 添加到 manual album
- **THEN** 系统写入 album membership, 并在后续以该 album id 查询 Gallery 时返回该 asset

#### Scenario: 重复添加 Asset 到 Album

- **WHEN** 用户将已经属于该 manual album 的 asset 再次添加到同一 album
- **THEN** 系统将该操作视为成功或 no-op, 且不创建重复 membership

### Requirement: Core 定义 Gallery Sort 语义

系统 MUST 由 core 定义 Gallery sort 语义, 并至少支持 newest, oldest, rating_desc, title_asc 和 provider_asc.

#### Scenario: 按最新排序

- **WHEN** 用户选择 newest sort
- **THEN** 系统按 asset 或首选 version 的最新相关时间降序返回 Gallery asset 列表

#### Scenario: Sort 值无效

- **WHEN** 用户提交不支持的 sort 值
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

### Requirement: Gallery Query 支持 Review Status

系统 SHALL 支持按 review status 查询需要 metadata review 的 asset.

#### Scenario: 查询待 Review 资产

- **WHEN** 用户选择 review pending filter
- **THEN** 系统只返回存在 pending metadata suggestion 的 Gallery asset
