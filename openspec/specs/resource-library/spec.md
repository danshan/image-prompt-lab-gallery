## Purpose

Define local resource library layout, integrity checking, repair, import/export, and file context behavior.
## Requirements
### Requirement: 创建受管理资源库

系统 SHALL 支持在指定本地路径创建一个受管理资源库, 包含 `manifest.json`, `library.sqlite`, `originals`, `derivatives`, `sidecars`, `exports` 和 `trash`. Runtime adapters MUST call the library lifecycle application owner for this workflow.

#### Scenario: 创建新资源库

- **WHEN** 用户请求在空目录创建资源库并提供名称
- **THEN** 系统创建资源库目录结构, 写入 manifest, 初始化 SQLite schema, 并返回资源库 id 和路径

### Requirement: 打开和校验资源库

系统 SHALL 在打开资源库时校验 manifest, schema version 和必要目录, 并拒绝打开不兼容或损坏的资源库. Runtime adapters MUST call the library lifecycle application owner for this workflow.

#### Scenario: 打开 schema 不兼容的资源库

- **WHEN** 用户打开 schema version 高于当前应用支持版本的资源库
- **THEN** 系统返回 `SchemaMismatch` 错误且不修改资源库内容

### Requirement: 管理资源库 registry

系统 SHALL 维护本机 app-level registry, 记录最近打开的资源库, display name, root path, hidden 状态和 last opened time. 系统 SHALL 支持按本机 alias 重命名 registry display name, 并支持取消注册资源库. 取消注册 MUST 从默认 registry 列表中移除资源库, 但不得删除资源库目录或 SQLite 数据. 新 Settings UI MUST NOT 暴露 hide 功能; hidden 状态仅作为既有 registry 数据兼容语义保留. Runtime adapters MUST call the library lifecycle application owner for registry lifecycle workflows.

#### Scenario: 隐藏资源库

- **WHEN** 旧客户端或兼容路径隐藏一个已注册资源库
- **THEN** 系统将该资源库从默认列表中隐藏, 但不删除资源库目录或 SQLite 数据

#### Scenario: 取消注册资源库

- **WHEN** 用户在 Settings / Libraries 中 Close 一个已注册资源库
- **THEN** 系统将该资源库从 registry 默认列表中移除, 且不删除资源库目录或 SQLite 数据

#### Scenario: 重命名 Registry Display Name

- **WHEN** 用户在 Settings / Libraries 中重命名一个已注册资源库
- **THEN** 系统更新 registry display name, 且不修改资源库 manifest

### Requirement: 取消注册资源库

系统 SHALL 支持从 app-level registry 取消注册一个已注册资源库. 取消注册 MUST NOT 删除资源库目录, `manifest.json`, `library.sqlite`, managed 图片文件, sidecars, exports 或任何其他磁盘文件.

#### Scenario: 取消注册已注册资源库

- **WHEN** 用户在 Settings / Libraries 中 Close 一个已注册资源库
- **THEN** 系统从 app-level registry 中移除该资源库, 且不修改或删除该资源库磁盘目录中的任何文件

#### Scenario: 取消注册不存在的资源库

- **WHEN** 用户请求取消注册一个 registry 中不存在的 library id
- **THEN** 系统返回 `LibraryNotFound` 错误

### Requirement: 重命名本机资源库别名

系统 SHALL 支持修改 app-level registry 中的资源库 display name. 该操作 MUST 只修改本机 registry display name, MUST NOT 修改资源库 `manifest.json` 中的 name, id 或其他 portable identity 字段.

#### Scenario: 修改资源库本机显示名

- **WHEN** 用户在 Settings / Libraries 中重命名一个已注册资源库
- **THEN** 系统更新 registry display name, 并保持该资源库 `manifest.json` 内容不变

#### Scenario: 拒绝空显示名

- **WHEN** 用户提交空白资源库显示名
- **THEN** 系统拒绝该请求, 且不修改 registry 或 manifest

### Requirement: 导出完整资源库备份 Zip

系统 SHALL 支持将一个完整 managed resource library 导出为 zip 备份包. 备份包 MUST 用于恢复一个可打开的资源库, 并 MUST 包含 `manifest.json`, `library.sqlite`, required dirs 和 managed library 文件. 该 workflow MUST 与按资源库或相册导出图片及 sidecar metadata 的内容导出语义保持分离.

#### Scenario: 导出完整资源库备份

- **WHEN** 用户在 Settings / Libraries 中对一个有效资源库执行 Export Zip
- **THEN** 系统生成一个包含该资源库 manifest, database, required dirs 和 managed files 的 zip 文件

#### Scenario: 导出前校验资源库布局

- **WHEN** 用户对 layout 损坏或路径不存在的资源库执行 Export Zip
- **THEN** 系统拒绝导出并返回可恢复错误, 且不报告导出成功

#### Scenario: 避免半成品 Zip 被视为成功

- **WHEN** Export Zip 在写入过程中失败
- **THEN** 系统不得将部分写入的 zip 作为成功结果返回

### Requirement: 导入完整资源库备份 Zip

系统 SHALL 支持将完整资源库备份 zip 导入到新的本地资源库目录. 导入 MUST 校验 zip 内容是完整资源库备份, MUST NOT 将备份内容合并进当前资源库. 导入只有在解压, 校验和必要 manifest 更新全部成功后才可注册资源库.

#### Scenario: 导入有效备份 Zip

- **WHEN** 用户选择一个有效资源库备份 zip 和目标目录
- **THEN** 系统解压并校验该资源库, 注册导入后的资源库, 并返回可打开的资源库 summary

#### Scenario: 拒绝无效备份 Zip

- **WHEN** 用户选择的 zip 缺少 `manifest.json`, `library.sqlite` 或 required dirs
- **THEN** 系统返回 `InvalidLibraryBackup` 错误, 且不注册部分导入的资源库

#### Scenario: 导入目标目录不可安全使用

- **WHEN** 用户选择的导入目标目录非空且不能安全承载新的资源库
- **THEN** 系统返回 `ImportDestinationNotEmpty` 错误, 且不覆盖目标目录内容

#### Scenario: 导入时资源库 Id 冲突自动克隆

- **WHEN** 导入备份 zip 中的 `manifest.id` 已存在于 app-level registry
- **THEN** 系统为导入后的资源库生成新的 library id, 重写导入目录中的 manifest, 并将其作为独立 clone 注册

#### Scenario: 导入失败不留下已注册半成品

- **WHEN** 导入在解压, 校验, manifest 更新或移动阶段失败
- **THEN** 系统不得向 app-level registry 写入该失败导入的资源库

### Requirement: 按日期分桶存储 Managed 图片文件

系统 SHALL 在写入新的 managed 图片文件时使用 UUID 文件名, 并将文件保存到按创建日期拆分的相对路径 `$year/$month/$filename`.

#### Scenario: 导入图片写入日期分桶路径

- **WHEN** 用户导入一个本地图片文件
- **THEN** 系统复制图片到 managed library 的 `originals/$year/$month/$uuid.$extension` 路径, 且后续不依赖原始外部路径

#### Scenario: 生成图片写入日期分桶路径

- **WHEN** generation flow 产生一个新的图片 asset version
- **THEN** 系统将生成图片写入 managed library 的 `originals/$year/$month/$uuid.$extension` 路径

#### Scenario: 文件名不复用外部源文件名

- **WHEN** 两个不同源文件具有相同文件名
- **THEN** 系统为每个 managed 图片文件生成不同 UUID 文件名, 并在 asset version 中记录各自相对路径

### Requirement: 使用 SHA-256 Checksum 校验 Asset Version 文件
系统 SHALL 对新写入的 asset version 文件计算 SHA-256 checksum, 并在 integrity check 和 file context 中使用对应 checksum algorithm 校验文件. 系统 MUST 保持对历史 MD5 metadata 的兼容读取, 但当前标准 checksum algorithm 是 `SHA-256`. Checksum 计算 MUST 使用 bounded memory streaming read, 不得把完整大文件读入内存后再计算 digest.

#### Scenario: 新导入文件记录 SHA-256
- **WHEN** 用户导入一个图片文件
- **THEN** 系统记录该 managed 文件的 checksum algorithm 为 `SHA-256`, 并记录 64 位十六进制 SHA-256 digest

#### Scenario: Integrity Check 使用记录的算法
- **WHEN** integrity check 校验一个 asset version 文件
- **THEN** 系统使用该 version 记录的 checksum algorithm 重新计算 digest, 并与记录的 checksum 比较

#### Scenario: SHA-256 不匹配
- **WHEN** integrity check 发现文件当前 SHA-256 与记录的 SHA-256 不一致
- **THEN** 系统报告 `FileIntegrityMismatch` 并标明受影响的 version id

#### Scenario: 历史 MD5 Metadata 保持可读
- **WHEN** integrity check 校验一个历史 asset version 且其 checksum algorithm 为 `MD5`
- **THEN** 系统使用 MD5 重新计算 digest, 并与该 version 记录的 checksum 比较

#### Scenario: 大文件 Checksum 使用 Bounded Memory
- **WHEN** 系统对 asset version 文件计算 checksum
- **THEN** 系统按固定大小 buffer streaming 读取文件, 内存峰值不得随文件大小线性增长

### Requirement: 提供资源库实际存储大小

系统 SHALL 提供当前 resource library 的实际 managed storage size, 且该值不得包含固定容量上限.

#### Scenario: 读取 Library Storage Size

- **WHEN** 桌面端请求 Library Status read model
- **THEN** 系统返回当前 Library managed files 占用大小, 且不返回固定容量上限或百分比上限

### Requirement: 写入 Asset Version 时记录图片分辨率
系统 SHALL 在导入或生成图片写入新的 asset version 时解析图片文件头, 并在能够可靠读取时记录 width 和 height. 对无法识别或无法可靠读取尺寸的文件, 系统 MUST 将 width 和 height 保持为空, 不得伪造分辨率. 图片尺寸解析 MUST 只读取必要 header 范围, 不得为了读取尺寸而加载完整大图片.

#### Scenario: 导入 PNG 图片记录分辨率
- **WHEN** 用户导入一个包含有效 PNG header 的图片文件
- **THEN** 系统在 asset version 中记录该图片的 width 和 height

#### Scenario: 生成图片记录分辨率
- **WHEN** generation flow 产生一个可识别分辨率的图片文件
- **THEN** 系统在输出 asset version 中记录该图片的 width 和 height

#### Scenario: 图片尺寸不可用
- **WHEN** 系统无法从图片文件头可靠读取 width 或 height
- **THEN** 系统写入 asset version 时保持 width 和 height 为空

#### Scenario: 大图片尺寸解析使用 Header Read
- **WHEN** 系统解析 PNG, JPEG 或 WebP 文件尺寸
- **THEN** 系统只读取识别格式所需的 header 或 bounded prefix, 不得整文件读入内存

### Requirement: 修复历史 Asset Version 文件元数据

系统 SHALL 提供显式的资源库 repair 操作, 用于将历史 asset version 文件和 SQLite metadata 修复到当前 managed library 标准. repair MUST 支持 dry run, 并在执行模式下同步修复文件路径, checksum metadata 和图片分辨率. 当前 checksum 标准 MUST 为 SHA-256.

#### Scenario: Dry Run 检查历史脏数据

- **WHEN** 用户对资源库执行 repair dry run
- **THEN** 系统返回将要移动的文件数量, 将要更新的 checksum 数量, 将要更新的 dimensions 数量和无法自动修复的问题列表, 且不修改文件系统或 SQLite

#### Scenario: 修复非标准文件路径

- **WHEN** 某个 asset version 的文件存在, 但 `file_path` 不符合 `originals/$year/$month/$uuid.$extension`
- **THEN** repair 操作将文件移动到标准路径, 并同步更新 SQLite 中该 version 的 `file_path`

#### Scenario: 回填历史图片分辨率

- **WHEN** 某个 asset version 文件可读取图片尺寸, 且 SQLite 中 width 或 height 缺失或不一致
- **THEN** repair 操作更新 SQLite 中该 version 的 width 和 height

#### Scenario: 修复历史 Checksum Metadata

- **WHEN** 某个 asset version 文件存在, 但 checksum algorithm 或 checksum 不符合当前 SHA-256 标准
- **THEN** repair 操作重新计算文件 SHA-256, 并更新该 version 的 checksum algorithm 和 checksum

#### Scenario: 文件缺失无法自动修复

- **WHEN** repair 操作发现某个 asset version 记录的文件不存在
- **THEN** 系统在 repair summary 中报告该 version 的问题, 且不删除该 SQLite 记录

### Requirement: 导入和导出资源库内容

系统 SHALL 支持将外部图片导入 managed library, 并支持按资源库或相册导出图片和 sidecar metadata. 导入时, 系统 MUST 将图片复制到按日期分桶的 managed file layout, 文件名 MUST 使用 UUID, 不得复用外部源文件名作为 managed 文件名.

#### Scenario: 导入外部图片

- **WHEN** 用户导入一个本地图片文件
- **THEN** 系统复制图片到 `originals/$year/$month/$uuid.$extension`, 计算 SHA-256 checksum, 写入 SQLite, 并且后续不依赖原始外部路径

### Requirement: 校验资源库完整性

系统 SHALL 支持检查 SQLite 记录与 managed file layout 的一致性, 包括文件存在性和 checksum. 校验 checksum 时, 系统 MUST 使用 asset version 记录的 checksum algorithm.

#### Scenario: 原始文件丢失

- **WHEN** integrity check 发现某个 asset version 的文件不存在
- **THEN** 系统报告 `FileIntegrityMismatch` 并标明受影响的 version id

#### Scenario: Checksum 不匹配

- **WHEN** integrity check 发现某个 asset version 的当前 digest 与记录 checksum 不一致
- **THEN** 系统报告 `FileIntegrityMismatch` 并在问题信息中包含记录的 checksum algorithm

### Requirement: 提供 File Context Read Model

系统 SHALL 在 asset detail read model 中提供当前 version 的文件上下文, 包括 filename, relative location, MIME type, checksum algorithm, checksum 和 integrity status.

#### Scenario: 查看文件上下文

- **WHEN** 用户在 Inspector 查看 File section
- **THEN** 系统返回当前 version 的文件名, 相对位置, MIME type, checksum algorithm, checksum 和 integrity 状态

#### Scenario: 查看文件分辨率

- **WHEN** 当前 version 记录了 width 和 height
- **THEN** 系统在 File Context Read Model 中返回该 width 和 height

#### Scenario: 展示 SHA-256 Checksum

- **WHEN** 当前 version 的 checksum algorithm 为 `SHA-256`
- **THEN** UI 展示 `Checksum    SHA-256: $hash`

### Requirement: File Context 允许缺失的派生字段为空

系统 SHALL 在无法可靠获得 file size, dimensions 或 generation duration 时返回空值, 不得伪造真实文件元数据.

#### Scenario: 文件尺寸信息不可用

- **WHEN** 当前资源库没有记录某个 version 的尺寸信息
- **THEN** 系统返回空尺寸字段, UI 展示 unavailable 状态

### Requirement: 支持 Inspector 触发完整性复查

系统 SHALL 提供从桌面端触发资源库或当前 asset version 完整性复查的能力.

#### Scenario: 重新校验当前文件

- **WHEN** 用户在 Inspector File section 点击 re-verify
- **THEN** 系统校验当前 version 文件存在性和 checksum, 并返回更新后的 integrity 状态

### Requirement: SQLite Schema 提供热路径索引
系统 SHALL 为 Gallery, Search, Metadata Review, Album 和 Task 热路径创建必要 SQLite indexes, 并使用 idempotent migration 保证既有资源库可升级.

#### Scenario: 打开旧资源库时补索引
- **WHEN** 用户打开缺少热路径索引的旧资源库
- **THEN** 系统通过 migration 创建缺失索引, 且不改变既有业务数据

#### Scenario: Gallery 热路径使用索引
- **WHEN** Gallery/Search 查询读取 assets, asset_versions, generation_events, metadata_suggestions, album_items, asset_tags 或 tags
- **THEN** schema 必须存在支持常用 filter, join 和 ordering 的索引

### Requirement: Studio Overview Library Status

系统 SHALL 提供 Studio Overview 所需的当前 library status read model, 至少包含 current library display name, root path, schema version, storage size, integrity status, integrity issue count, registered library count 和 missing library count. 该 read model SHALL 用于 Library Context 和 Settings diagnostics.

#### Scenario: 加载 Library Context Summary
- **WHEN** 桌面应用加载 Studio Console Library Context
- **THEN** 系统返回当前 library display name, root path, storage size, integrity status 和 registered library summary

#### Scenario: Missing Libraries Summary
- **WHEN** app-level registry 中存在 root path missing 的 registered libraries
- **THEN** Studio Overview 返回 missing library count, Settings diagnostics 可展示对应恢复入口

### Requirement: Provider Health Summary

系统 SHALL 提供 Studio Overview 所需 provider health summary. Provider health summary SHALL 至少包含 provider id, display name, availability, supported operations, credential/configuration state 和 recoverable error message. 该 summary 仅表达 UI-visible health, 不要求改变 provider execution strategy.

#### Scenario: Provider Available
- **WHEN** provider 配置可用且支持至少一个 operation
- **THEN** provider health summary 将该 provider 标记为 available, 并返回 supported operations

#### Scenario: Provider Configuration Error
- **WHEN** provider 缺失 credential, CLI 不可用或 configuration invalid
- **THEN** provider health summary 返回 unavailable 或 degraded state 和可恢复错误信息, 不发起 generation request

#### Scenario: Provider Health 不改变 Provider Strategy
- **WHEN** 系统刷新 provider health summary
- **THEN** 系统只执行轻量配置/capability 检查, 不改变 provider adapter execution strategy, 不创建 generation event

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

### Requirement: Asset metadata mutation preserves existing library storage

Asset metadata mutation SHALL preserve current library schema and compatibility behavior unless an explicit migration is specified.

#### Scenario: Tag add preserves storage semantics

- **WHEN** a tag is added through the application owner
- **THEN** existing tag rows are reused by tag name
- **AND** the asset-tag relation is upserted without changing the SQLite schema

#### Scenario: Tauri tag add preserves storage semantics

- **WHEN** a tag is added through the Tauri command adapter
- **THEN** the command delegates to the asset application owner
- **AND** existing tag storage and asset-tag upsert semantics are preserved

### Requirement: Persistence and query engine changes require a decision gate

Resource library persistence or query engine changes SHALL pass a documented decision gate before implementation. The decision gate MUST compare local-first portability, transaction correctness, workload fit, backup/restore behavior, migration and rollback complexity, rebuild/repair story, desktop distribution cost, testability, and observability.

#### Scenario: SQLite remains sufficient

- **WHEN** tuned SQLite meets target gallery, search, smart album, version tree, and task queue workloads
- **THEN** SQLite remains the primary resource library store
- **AND** any supplemental index is deferred or scoped with a rebuild/repair plan

#### Scenario: Supplemental index is selected

- **WHEN** FTS5, projection tables, Tantivy, DuckDB, PostgreSQL, or another engine is selected
- **THEN** the design records migration, rollback, backup/restore, rebuild, repair, and compatibility behavior before implementation

### Requirement: Scheduled Generation Persistence
Resource library SHALL persist scheduled generation jobs, runs, and run outputs in SQLite. Migration MUST preserve existing assets, versions, generation events, tasks, albums, tags, and metadata suggestions.

#### Scenario: Migrate Existing Library
- **WHEN** 系统打开旧 schema library
- **THEN** migration 创建 scheduled generation tables
- **AND** 旧数据保持可读

#### Scenario: New Library Has Empty Schedules
- **WHEN** 系统创建新 library
- **THEN** scheduled generation tables 存在
- **AND** schedule list 为空

### Requirement: Library Automation Opt-In
Resource library registry SHALL persist per-library automation opt-in. Default value MUST be disabled for existing and newly registered libraries.

#### Scenario: Default Disabled
- **WHEN** library 被创建或注册
- **THEN** `automation_enabled` 默认为 false

#### Scenario: Enable Library Automation
- **WHEN** 用户在 Settings Automation 开启某个 library 的 automation
- **THEN** registry 保存 `automation_enabled = true`

### Requirement: Backup Restore Includes Schedules
Library backup and restore SHALL preserve scheduled generation job, run, and run output records inside the library database.

#### Scenario: Export Backup
- **WHEN** 用户导出 library backup zip
- **THEN** backup 包含 scheduled generation persistence tables

#### Scenario: Restore Backup
- **WHEN** 用户恢复包含 schedules 的 backup
- **THEN** restored library 保留 scheduled generation records
- **AND** automation opt-in 仍由 app registry 控制

### Requirement: 归档 Asset Lifecycle

系统 SHALL 支持对 asset 执行 archive 和 restore. Archive MUST 是可逆的 soft delete, MUST 保留 asset rows, asset versions, managed files, album memberships, tags, metadata suggestions, generation events 和 prompt lineage. 归档状态 MUST 由 resource library 持久化, 不得只在 desktop frontend 中过滤.

#### Scenario: 归档 Asset 后默认不可见

- **WHEN** 用户归档一个 active asset
- **THEN** 系统记录该 asset 的 archived time
- **AND** 默认 Gallery query, Albums detail query 和 add-to-album source query 不再返回该 asset
- **AND** asset versions, managed files, generation events, tags, album memberships 和 metadata suggestions 保持可恢复

#### Scenario: 恢复归档 Asset

- **WHEN** 用户恢复一个 archived asset
- **THEN** 系统清除该 asset 的 archived state
- **AND** 后续默认 Gallery query 和相关 album-scoped query 可以再次返回该 asset

#### Scenario: 拒绝归档不存在的 Asset

- **WHEN** 用户请求归档不存在的 asset id
- **THEN** 系统返回明确 domain error
- **AND** 不修改 resource library

### Requirement: Archived Content Read Model

系统 SHALL 提供 archived content read model, 用于 Settings 展示 archived assets 和 archived prompt documents. Read model MUST 至少返回 item id, item type, display title, archived time, dependency summary 和 estimated file impact.

#### Scenario: 列出归档内容

- **WHEN** 用户打开 Settings Archived Content
- **THEN** 系统返回当前 library 中 archived assets 和 archived prompt documents
- **AND** 每个 item 包含可用于 restore 和 permanent delete dry-run 的 stable id

#### Scenario: 空归档列表

- **WHEN** 当前 library 没有 archived assets 或 archived prompt documents
- **THEN** 系统返回空列表
- **AND** 不把 active content 混入 archived list

### Requirement: Permanent Delete Archived Content

系统 SHALL 支持对 archived asset 或 archived prompt document 执行 permanent delete. Permanent delete MUST 先提供 dry-run summary, MUST 只允许 archived item, MUST 采用级联删除语义删除相关 SQLite facts 和 managed files. Active item MUST NOT 被 permanent delete.

#### Scenario: Permanent Delete Dry Run

- **WHEN** 用户对 archived item 请求 permanent delete dry-run
- **THEN** 系统返回 deletion summary
- **AND** summary 包含将删除的 SQLite facts, managed files, relationship rows 和 lineage/history references count
- **AND** dry-run 不修改 SQLite 或文件系统

#### Scenario: 拒绝 Permanent Delete Active Item

- **WHEN** 用户对 active asset 或 active prompt document 请求 permanent delete
- **THEN** 系统返回 domain error
- **AND** 不删除 SQLite rows
- **AND** 不删除 managed files

#### Scenario: Permanent Delete Archived Asset

- **WHEN** 用户确认 permanent delete 一个 archived asset
- **THEN** 系统级联删除该 asset 的 versions, managed files, album memberships, tags, metadata suggestions, generation events 和相关 task output references
- **AND** 后续 archived content list 和 Gallery query 均不再返回该 asset

#### Scenario: Permanent Delete Archived Prompt

- **WHEN** 用户确认 permanent delete 一个 archived prompt document
- **THEN** 系统级联删除该 prompt document, prompt versions 和相关 prompt output history references
- **AND** 后续 archived content list 和 Prompt Library list 均不再返回该 prompt document

#### Scenario: Managed File 删除失败可恢复

- **WHEN** permanent delete 的 SQLite transaction 已成功但 managed file 删除失败
- **THEN** 系统返回可恢复 issue
- **AND** 不得报告完全成功
- **AND** issue 包含无法删除的 file path 或等价定位信息

### Requirement: Merge Library Into Current Library

系统 SHALL 支持将另一个 managed source library 的创作内容 copy-merge 到当前 target library. Source library MUST 保持不变. Merge MUST 先执行 dry-run compatibility check, 再由用户确认执行 apply. Merge MUST NOT 改变 existing `Import Zip` restore-as-separate-library 语义.

#### Scenario: Merge Dry Run 成功

- **WHEN** 用户选择一个有效 source library folder 并请求 merge dry-run
- **THEN** 系统校验 source layout, manifest 和 schema version
- **AND** 系统返回将迁移的 assets, versions, tags, albums, prompts, prompt versions, generation events, metadata suggestions, file count 和 file size
- **AND** 系统报告将跳过的 runtime state
- **AND** dry-run 不修改 source library 或 target library

#### Scenario: 拒绝 Unsupported Source Schema

- **WHEN** source library schema version 高于当前应用支持版本
- **THEN** 系统拒绝 merge dry-run 和 merge apply
- **AND** 不修改 source library
- **AND** 不修改 target library

#### Scenario: 拒绝 Invalid Source Layout

- **WHEN** source library 缺少 manifest, database 或 required managed directories
- **THEN** 系统拒绝 merge dry-run 和 merge apply
- **AND** 返回可恢复错误

#### Scenario: Merge Apply 复制创作内容

- **WHEN** 用户确认 merge apply
- **THEN** 系统将 source assets, asset versions, tags, albums, prompt documents, prompt versions, generation events, metadata suggestions 和相关 lineage 复制到 target library
- **AND** target 中每个迁入实体使用新的 target-local id
- **AND** managed files 被复制到 target library 标准路径
- **AND** source library 保持不变

#### Scenario: Merge Apply 重写内部引用

- **WHEN** source generation event, asset version, album membership, tag relation 或 prompt lineage 引用 source entity id
- **THEN** merge apply 按 source-to-target ID map 写入 target-local references
- **AND** target library 不保存指向 source library entity id 的 dangling references

#### Scenario: Merge Apply 跳过 Runtime State

- **WHEN** source library 包含 queue tasks, schedules, logs, diagnostics, registry alias 或 manifest identity
- **THEN** merge apply 不迁移这些 runtime state
- **AND** dry-run summary 报告这些 records 被跳过

#### Scenario: Merge 不按名称智能合并

- **WHEN** source album 或 prompt document 与 target 中已有 item 同名
- **THEN** merge apply 创建新的 target item
- **AND** 系统可以为 display name 添加 source marker 或后缀
- **AND** 不把同名 item 的 contents 或 versions 合并到 existing item

#### Scenario: Existing Import Zip 语义不变

- **WHEN** 用户执行现有 Import Zip workflow
- **THEN** 系统仍将 backup restore 成独立 library
- **AND** manifest id 冲突时仍按 clone-on-conflict 语义处理
- **AND** 不把 backup 内容 merge 到当前 library
