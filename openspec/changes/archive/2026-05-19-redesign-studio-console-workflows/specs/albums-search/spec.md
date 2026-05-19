## MODIFIED Requirements

### Requirement: 提供 Album List Read Model

系统 SHALL 提供 albums 列表 read model, 用于 Studio Console Albums workspace 和 Library Context collections. Album list item SHALL 包含 album id, name, kind, item count, sort order, smart/manual status 和可选 preview context. Album list MUST 按 persisted sort order 返回, 并在 sort order 相同时按 name 稳定排序.

#### Scenario: 列出 Albums For Studio Console
- **WHEN** 桌面端请求当前 library 的 albums 列表
- **THEN** 系统返回该 library 中可见 albums 的 id, name, kind, item count, sort order 和 display context

### Requirement: Gallery Query 支持 Album Detail

系统 SHALL 使用 core Gallery query 的 album filter 展示 album 内容. 打开 manual album 或 smart album 后, Studio Console SHALL 展示 album-scoped asset board. Manual album 在 sort 为 `album_order` 时 MUST 按 `album_items.sort_order` 返回 assets. Smart album MUST 使用 typed smart query 计算内容, 不通过 `album_items` 持久化 membership.

#### Scenario: 打开 Manual Album Asset Board
- **WHEN** 用户打开一个 manual album
- **THEN** 系统使用该 album id 查询 Gallery, 并返回 album-scoped asset board items

#### Scenario: 打开 Smart Album Asset Board
- **WHEN** 用户打开一个 smart album
- **THEN** 系统验证并执行该 smart album query, 返回满足条件的 asset board items

## ADDED Requirements

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
