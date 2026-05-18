## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: 管理资源库 registry

系统 SHALL 维护本机 app-level registry, 记录最近打开的资源库, display name, root path, hidden 状态和 last opened time. 系统 SHALL 支持按本机 alias 重命名 registry display name, 并支持取消注册资源库. 取消注册 MUST 从默认 registry 列表中移除资源库, 但不得删除资源库目录或 SQLite 数据. 新 Settings UI MUST NOT 暴露 hide 功能; hidden 状态仅作为既有 registry 数据兼容语义保留.

#### Scenario: 隐藏资源库

- **WHEN** 旧客户端或兼容路径隐藏一个已注册资源库
- **THEN** 系统将该资源库从默认列表中隐藏, 但不删除资源库目录或 SQLite 数据

#### Scenario: 取消注册资源库

- **WHEN** 用户在 Settings / Libraries 中 Close 一个已注册资源库
- **THEN** 系统将该资源库从 registry 默认列表中移除, 且不删除资源库目录或 SQLite 数据

#### Scenario: 重命名 Registry Display Name

- **WHEN** 用户在 Settings / Libraries 中重命名一个已注册资源库
- **THEN** 系统更新 registry display name, 且不修改资源库 manifest
