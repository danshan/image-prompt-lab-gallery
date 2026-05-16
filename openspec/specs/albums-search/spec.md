## ADDED Requirements

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

系统 SHALL 支持按文本, tag, rating, provider, date, status 和 category 搜索 asset.

#### Scenario: 按标签和评分搜索

- **WHEN** 用户搜索指定 tag 且 rating 大于等于 4 的资产
- **THEN** 系统返回同时满足两个条件的 asset 列表

### Requirement: 提供 Gallery Query

系统 SHALL 提供 core 定义的 Gallery query, 支持按 text, providers, min rating, review status, tags, album 和 sort 查询 asset card read model.

#### Scenario: 按文本和标签查询 Gallery

- **WHEN** 用户输入搜索文本并选择一个或多个 tags
- **THEN** 系统返回同时满足文本和 tags 条件的 Gallery asset 列表

#### Scenario: 按评分和 Provider 查询 Gallery

- **WHEN** 用户选择 min rating 和 provider filter
- **THEN** 系统返回同时满足评分下限和 provider 条件的 Gallery asset 列表

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
