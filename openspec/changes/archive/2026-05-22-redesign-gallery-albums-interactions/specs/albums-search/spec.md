## MODIFIED Requirements

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

## ADDED Requirements

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
