## ADDED Requirements

### Requirement: 升级历史 Library Version Metadata

系统 SHALL 在打开缺少数字 version number 的历史 library 时执行 schema migration, 为每个 asset 的历史 versions 回填确定性的数字版本号.

#### Scenario: 旧 Library 自动升级 Version Number

- **WHEN** 用户打开上一支持 schema version 的 library
- **THEN** 系统为每个 asset 的 versions 按 `created_at ASC, id ASC` 回填 `1..N` 的 version number, 并在 migration 成功后打开 library

#### Scenario: 未来 Schema 仍被拒绝

- **WHEN** 用户打开 schema version 高于当前应用支持版本的 library
- **THEN** 系统返回 `SchemaMismatch`, 且不修改 library 内容

#### Scenario: Migration 不重写稳定引用

- **WHEN** 系统执行 version number migration
- **THEN** 系统不得重写 asset id, asset version id, file path, checksum, parent version link, generation event id 或 task output link

#### Scenario: Migration 失败不报告成功

- **WHEN** 系统无法完成 version number 回填或无法创建唯一约束
- **THEN** 系统返回可恢复 migration error, 且不得报告 library 已成功打开

### Requirement: 管理 Reference Asset 可见性

系统 SHALL 将 uploaded reference images 保存为 managed reference assets, 并默认从普通 Gallery 查询中排除.

#### Scenario: Reference Asset 默认不进入 Gallery

- **WHEN** Gallery 执行默认查询
- **THEN** 系统返回 generated 和 imported content assets, 且不返回 `status = reference` 的 assets

#### Scenario: Source Link 可打开 Reference Asset

- **WHEN** 用户从 output asset 的 generation source 打开 reference source
- **THEN** 系统返回 reference asset detail 和 reference version file context

#### Scenario: Backup Restore 保留 Reference Asset

- **WHEN** 用户备份并恢复包含 reference asset 的 library
- **THEN** 恢复后的 library 保留 reference asset, reference version, checksum metadata 和 generation source link
