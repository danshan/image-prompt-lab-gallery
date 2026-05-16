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
