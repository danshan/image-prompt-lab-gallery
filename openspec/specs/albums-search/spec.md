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
系统 SHALL 提供 core 定义的 Gallery query, 支持按 text, providers, min rating, review status, tags, album filter 和 sort 查询 asset card read model. Gallery album filter SHALL 支持 `Any`, `InAny(albumIds)` 和 `Unassigned`. `InAny(albumIds)` MUST 使用 union 语义并返回属于任意指定 album 的 assets, 且不得返回重复 asset. `Unassigned` MUST 返回不属于任何 album 的 assets. Gallery text query SHALL 匹配 asset title, generation prompt, provider, model label 和 tags. Gallery asset card read model SHALL 返回当前版本可追溯到的 provider, model label, tags 和 image path. Gallery read path MUST 避免按 asset 重复查询 current version, generation event, tags, pending review count 或 album membership.

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

#### Scenario: 多 Album Union 查询 Gallery
- **WHEN** 用户提交 `InAny` album filter 且包含多个 album ids
- **THEN** 系统返回属于任意指定 album 的 Gallery assets
- **AND** 同一个 asset 即使属于多个 selected albums 也只返回一次

#### Scenario: 查询未分配 Album 的 Gallery Assets
- **WHEN** 用户提交 `Unassigned` album filter
- **THEN** 系统只返回不属于任何 manual 或 smart album membership 的 assets

#### Scenario: 空 Album Filter 回到 Any
- **WHEN** 用户提交 `InAny` album filter 且 album ids 为空
- **THEN** 系统按 `Any` album filter 执行 Gallery query

#### Scenario: Gallery 查询避免 N+1
- **WHEN** 系统执行 Gallery query 并返回多个 assets
- **THEN** current version, generation event, tags, version count, pending review count 和 album membership 必须通过 batch read 或 projection read model 取得, 不得按返回 asset 数量线性增加 SQL round-trip

### Requirement: 提供 Album List Read Model

系统 SHALL 提供 albums 列表 read model, 用于 Studio Console Albums workspace 和 Library Context collections. Album list item SHALL 包含 album id, name, kind, item count, sort order, smart/manual status 和可选 preview context. Album list MUST 按 persisted sort order 返回, 并在 sort order 相同时按 name 稳定排序.

#### Scenario: 列出 Albums For Studio Console
- **WHEN** 桌面端请求当前 library 的 albums 列表
- **THEN** 系统返回该 library 中可见 albums 的 id, name, kind, item count, sort order 和 display context

### Requirement: Gallery Query 支持 Album Detail

系统 SHALL 使用 core Gallery query 的 album filter 展示 album 内容. 打开 manual album 或 smart album 后, Studio Console SHALL 展示 album-scoped asset board. Manual album detail MUST 使用单个 manual album context 查询 Gallery. Manual album 在 sort 为 `album_order` 时 MUST 按 `album_items.sort_order` 返回 assets. Smart album MUST 使用 typed smart query 计算内容, 不通过 `album_items` 持久化 membership.

#### Scenario: 打开 Manual Album Asset Board
- **WHEN** 用户打开一个 manual album
- **THEN** 系统使用该 album id 查询 Gallery, 并返回 album-scoped asset board items

#### Scenario: 打开 Smart Album Asset Board
- **WHEN** 用户打开一个 smart album
- **THEN** 系统验证并执行该 smart album query, 返回满足条件的 asset board items

#### Scenario: Manual Album 使用 Album Order
- **WHEN** 用户打开 manual album 并选择 `album_order` sort
- **THEN** 系统按该 manual album 的 `album_items.sort_order` 返回 Gallery asset board items

### Requirement: 添加 Asset 到 Manual Album 后可查询

系统 SHALL 支持将当前 asset 或多个 selected assets 添加到 manual album, 并在写入后通过 album-scoped Gallery query 和 Gallery album filter 查询到这些 assets. 重复添加已存在 membership MUST 视为成功或 no-op, 且不得创建重复 membership. 对 smart album 执行 add, remove 或 reorder MUST 返回可恢复 domain error, 且不得写入 `album_items`.

#### Scenario: 添加当前 Asset 到 Album

- **WHEN** 用户从 Inspector 或 Albums add drawer 将当前 asset 添加到 manual album
- **THEN** 系统写入 album membership, 并在后续以该 album id 查询 Gallery 时返回该 asset

#### Scenario: 重复添加 Asset 到 Album

- **WHEN** 用户将已经属于该 manual album 的 asset 再次添加到同一 album
- **THEN** 系统将该操作视为成功或 no-op, 且不创建重复 membership

#### Scenario: 批量添加 Selected Assets 到 Album

- **WHEN** 用户将多个 selected assets 添加到 manual album
- **THEN** 系统写入缺失 memberships, 跳过已存在 memberships, 并允许后续以该 album id 查询到这些 assets

#### Scenario: Smart Album 拒绝 Manual Membership Mutation

- **WHEN** 用户尝试向 smart album 添加 asset, 从 smart album 移除 asset 或重排 smart album items
- **THEN** 系统返回可恢复 domain error
- **AND** 系统不得写入 `album_items`

### Requirement: Core 定义 Gallery Sort 语义

系统 MUST 由 core 定义 Gallery sort 语义, 并至少支持 newest, oldest, rating_desc, title_asc, provider_asc 和 album_order. `album_order` MUST 仅在 query 能确定单个 manual album context 时有效. `album_order` 用于 `Any`, `Unassigned`, multi-album union 或 smart album query 时 MUST 返回 `InvalidGalleryQuery`, 且不得执行 fallback 排序.

#### Scenario: 按最新排序

- **WHEN** 用户选择 newest sort
- **THEN** 系统按 asset 或首选 version 的最新相关时间降序返回 Gallery asset 列表

#### Scenario: Sort 值无效

- **WHEN** 用户提交不支持的 sort 值
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

#### Scenario: Album Order 用于非 Album 查询

- **WHEN** 用户提交 `album_order` sort 但没有单个 manual album context
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

#### Scenario: Album Order 用于多 Album Union 查询

- **WHEN** 用户提交 `album_order` sort 且 album filter 包含多个 album ids
- **THEN** 系统返回 `InvalidGalleryQuery` 且不执行 fallback 排序

### Requirement: Gallery Album Filter 输入校验

系统 MUST 校验 Gallery album filter 输入. `Unassigned` 与具体 album ids MUST 互斥. Unknown album id MUST 返回明确 domain error. UI 或 adapter 层 MAY 接受 legacy single album id 输入, 但 internal service semantics MUST 转换为 explicit album filter.

#### Scenario: Unassigned 与 Album Ids 互斥

- **WHEN** 用户提交同时包含 `Unassigned` 和具体 album ids 的 Gallery album filter
- **THEN** 系统返回 `InvalidGalleryQuery`
- **AND** 系统不得执行 fallback 查询

#### Scenario: Unknown Album Id

- **WHEN** 用户提交不存在于当前 library 的 album id
- **THEN** 系统返回明确 domain error
- **AND** 系统不得静默返回全库结果

#### Scenario: Legacy Album Id Compatibility

- **WHEN** adapter 收到 legacy single album id 查询输入
- **THEN** adapter 将其转换为 explicit single album filter
- **AND** core service 使用 explicit album filter 执行查询

### Requirement: Gallery Query 支持 Review Status

系统 SHALL 支持按 review status 查询需要 metadata review 的 asset.

#### Scenario: 查询待 Review 资产

- **WHEN** 用户选择 review pending filter
- **THEN** 系统只返回存在 pending metadata suggestion 的 Gallery asset

### Requirement: Smart Album Live Preview

系统 SHALL 支持 Smart Album builder 的 core-backed live preview. Preview MUST 使用与保存 smart album 相同的 typed query validation 和 Gallery query semantics. UI MUST NOT 在前端临时猜测 smart query 结果作为最终语义.

#### Scenario: Preview Valid Smart Query
- **WHEN** 用户在 Smart Album builder 中设置 text, tags, providers, min rating, review status, category, status, created date range 或 sort
- **THEN** 系统通过 core validation 返回 preview count 和可选 preview asset items

#### Scenario: Preview Invalid Smart Query
- **WHEN** Smart Album builder 包含非法字段, 非法 date range, 非法 rating 或不支持 sort
- **THEN** 系统返回 `InvalidSmartAlbumQuery`, UI 展示可恢复错误, 不保存 smart album

### Requirement: Album-Scoped Asset Board Context

Album-scoped asset board SHALL 展示 album context, 包括 album name, kind, item count, query summary, manual order 或 smart rule state. Asset item MUST 保留 review state, version summary, provider/model 和 task origin.

#### Scenario: Manual Album Board Context
- **WHEN** 用户查看 manual album detail
- **THEN** Workspace 展示 album name, manual kind, item count, manual order affordance 和 album-scoped asset board

#### Scenario: Smart Album Board Context
- **WHEN** 用户查看 smart album detail
- **THEN** Workspace 展示 album name, smart kind, query summary, preview count 和 album-scoped asset board

### Requirement: Albums Workspace 状态覆盖

Albums workspace SHALL 覆盖 album list, album detail, smart builder 和 album-scoped asset board 的 loading, empty, error 和 recovery states.

#### Scenario: Empty Albums
- **WHEN** 当前 library 尚未创建 album
- **THEN** Albums workspace 展示 empty state 和创建 manual/smart album 的入口

#### Scenario: Album Detail Error
- **WHEN** album detail 或 album-scoped query 加载失败
- **THEN** Albums workspace 保留 album list context, 展示可恢复错误和 retry 操作
