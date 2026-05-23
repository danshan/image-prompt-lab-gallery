## ADDED Requirements

### Requirement: Library Schema 支持 Promoted Version Source

系统 SHALL 在 resource library schema 中支持记录 promoted version source relation. Migration MUST 对历史 library 幂等创建 relation storage, 并保持没有 promoted rows 的 library 可正常打开.

#### Scenario: 历史 Library Migration 创建 Source Relation Storage

- **WHEN** 用户打开缺少 promoted source relation storage 的历史 library
- **THEN** 系统执行 schema migration
- **AND** 创建可记录 `promoted_from` source relation 的 storage
- **AND** 不重写现有 asset version parent links

#### Scenario: Migration 幂等

- **WHEN** 用户重复打开已经迁移过的 library
- **THEN** migration 不重复创建 relation storage
- **AND** library 正常打开

### Requirement: Promote Workflow 保持文件与数据库一致

系统 SHALL 在 promote version as new asset workflow 中保持 managed file 和 database record 一致. 文件复制或校验失败 MUST NOT 创建普通 Gallery 可见的半成品 asset.

#### Scenario: Source File Missing

- **WHEN** 用户 promote 的 source version 文件缺失
- **THEN** 系统返回 recoverable domain error
- **AND** 不创建新 Gallery asset

#### Scenario: Promote 成功创建 New Root

- **WHEN** source version 文件可读且校验通过
- **THEN** 系统复制 source file 到新的 managed version path
- **AND** 创建新 asset 和 root version
- **AND** root version 可被普通 Gallery 查询展示
