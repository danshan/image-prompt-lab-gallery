## ADDED Requirements

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
