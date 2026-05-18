## Purpose

Define album and search behavior for organizing assets through manual albums, smart albums, tags, ratings, and structured queries.
## Requirements
### Requirement: 管理 Manual Albums

系统 SHALL 支持创建 manual album, 重命名 manual album, 删除 manual album, 添加和移除 asset, 批量添加多个 assets, 并维护 album item sort order. Manual album item reorder MUST 只接受属于目标 album 的 assets, 且 MUST 以事务方式更新 sort order.

#### Scenario: 添加图片到手动相册

- **WHEN** 用户将 asset 添加到 manual album
- **THEN** 系统创建 album item, 记录 added time, 并按 sort order 返回相册内容

#### Scenario: 批量添加图片到手动相册

- **WHEN** 用户将多个 assets 批量添加到同一个 manual album
- **THEN** 系统为不存在的 memberships 创建 album items, 已存在 membership 视为成功或 no-op, 且不创建重复 membership

#### Scenario: 从手动相册移除图片

- **WHEN** 用户从 manual album 中移除一个 asset
- **THEN** 系统删除该 album membership, 不删除 asset, asset versions 或 canonical metadata

#### Scenario: 重命名手动相册

- **WHEN** 用户修改 manual album name
- **THEN** 系统更新 album name 和 updated time, 并在 album list read model 中返回新名称

#### Scenario: 删除手动相册

- **WHEN** 用户删除 manual album
- **THEN** 系统删除该 album 及其 album items, 不删除任何 asset 或 asset version

#### Scenario: 拖拽排序手动相册内容

- **WHEN** 用户提交某个 manual album 内完整的 ordered asset ids
- **THEN** 系统验证这些 assets 均属于该 album, 更新 `album_items.sort_order`, 并在 album-scoped query 中按新顺序返回

#### Scenario: 对 Smart Album 执行手动相册操作

- **WHEN** 用户尝试对 smart album 添加 asset, 移除 asset 或重排 items
- **THEN** 系统返回可恢复错误, 且不写入 `album_items`

### Requirement: 管理 Smart Albums

系统 SHALL 支持 smart album, 使用受限 typed smart query 自动选择 asset. Smart album query SHALL 支持 text, tags, providers, min rating, review status, category, status, created date range 和 sort. Smart album MUST NOT 通过 `album_items` 持久化 membership.

#### Scenario: 评分智能相册

- **WHEN** 用户创建条件为 rating 大于等于 4 的 smart album
- **THEN** 系统返回所有满足条件的 asset, 且无需写入 album item

#### Scenario: Created Date Range 智能相册

- **WHEN** 用户创建条件为 `assets.created_at` 在指定起止日期范围内的 smart album
- **THEN** 系统返回 created time 满足该范围的 assets, 且不写入 album item

#### Scenario: Review Pending 智能相册

- **WHEN** 用户创建条件为 review status pending 的 smart album
- **THEN** 系统返回存在 pending metadata suggestion 的 assets

### Requirement: 限制 Smart Query 字段

系统 MUST 将 smart album query 限制为 typed query contract. 允许字段为 text, tags, providers, minRating, reviewStatus, category, status, createdAtFrom, createdAtTo 和 sort. 系统 MUST 验证字段类型, rating 范围, date 格式, review status 和 sort 值.

#### Scenario: 查询字段不支持

- **WHEN** 用户创建包含未允许字段的 smart album query
- **THEN** 系统返回 `InvalidSmartAlbumQuery` 且不保存该 smart album

#### Scenario: Created Date 字段无效

- **WHEN** 用户创建包含非法 date 格式或 createdAtFrom 晚于 createdAtTo 的 smart album query
- **THEN** 系统返回 `InvalidSmartAlbumQuery` 且不保存该 smart album

#### Scenario: Sort 值无效

- **WHEN** 用户创建包含不支持 sort 值的 smart album query
- **THEN** 系统返回 `InvalidSmartAlbumQuery` 且不保存该 smart album

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

### Requirement: 提供 Album List Read Model

系统 SHALL 提供 albums 列表 read model, 用于桌面端展示当前 library 的 albums, 至少包含 album id, name, kind, item count 和 sort order. Album list MUST 按 persisted sort order 返回, 并在 sort order 相同时按 name 稳定排序.

#### Scenario: 列出 Manual Albums

- **WHEN** 桌面端请求当前 library 的 albums 列表
- **THEN** 系统返回该 library 中可见 albums 的 id, name, kind, item count 和 sort order

#### Scenario: 空 Album 列表

- **WHEN** 当前 library 尚未创建 album
- **THEN** 系统返回空列表, 且不伪造默认 album

#### Scenario: 拖拽排序 Album List

- **WHEN** 用户提交当前 library 中完整的 ordered album ids
- **THEN** 系统验证所有 ids 属于当前 library, 更新 albums sort order, 并在后续 album list read model 中按该顺序返回

### Requirement: Gallery Query 支持 Album Detail

系统 SHALL 使用 core Gallery query 的 album filter 展示 album 内容. 打开 manual album 后, Gallery query MUST 只返回该 album 中的 assets. 当 sort 为 `album_order` 时, manual album query MUST 按 `album_items.sort_order` 返回 assets.

#### Scenario: 打开 Manual Album

- **WHEN** 用户打开一个 manual album
- **THEN** 系统使用该 album id 查询 Gallery, 并返回该 album 中的 asset card read model

#### Scenario: Album 内容为空

- **WHEN** 用户打开一个没有 asset 的 manual album
- **THEN** 系统返回空 Gallery 列表, 且保持 album detail context

#### Scenario: 按 Album Order 查询 Manual Album

- **WHEN** 用户打开 manual album 且 Gallery query sort 为 `album_order`
- **THEN** 系统按 `album_items.sort_order` 升序返回该 album 的 asset card read model

### Requirement: 添加 Asset 到 Manual Album 后可查询

系统 SHALL 支持将当前 asset 或多个 selected assets 添加到 manual album, 并在写入后通过 album-scoped Gallery query 查询到这些 assets.

#### Scenario: 添加当前 Asset 到 Album

- **WHEN** 用户从 Inspector 将当前 asset 添加到 manual album
- **THEN** 系统写入 album membership, 并在后续以该 album id 查询 Gallery 时返回该 asset

#### Scenario: 重复添加 Asset 到 Album

- **WHEN** 用户将已经属于该 manual album 的 asset 再次添加到同一 album
- **THEN** 系统将该操作视为成功或 no-op, 且不创建重复 membership

#### Scenario: 批量添加 Selected Assets 到 Album

- **WHEN** 用户将多个 selected assets 添加到 manual album
- **THEN** 系统写入缺失 memberships, 跳过已存在 memberships, 并允许后续以该 album id 查询到这些 assets

### Requirement: Core 定义 Gallery Sort 语义

系统 MUST 由 core 定义 Gallery sort 语义, 并至少支持 newest, oldest, rating_desc, title_asc, provider_asc 和 album_order. `album_order` MUST 仅在 album-scoped query 中有效.

#### Scenario: 按最新排序

- **WHEN** 用户选择 newest sort
- **THEN** 系统按 asset 或首选 version 的最新相关时间降序返回 Gallery asset 列表

#### Scenario: Sort 值无效

- **WHEN** 用户提交不支持的 sort 值
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

#### Scenario: Album Order 用于非 Album 查询

- **WHEN** 用户提交 `album_order` sort 但没有 album filter
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

### Requirement: Gallery Query 支持 Review Status

系统 SHALL 支持按 review status 查询需要 metadata review 的 asset.

#### Scenario: 查询待 Review 资产

- **WHEN** 用户选择 review pending filter
- **THEN** 系统只返回存在 pending metadata suggestion 的 Gallery asset

