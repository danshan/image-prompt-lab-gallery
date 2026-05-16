## ADDED Requirements

### Requirement: 创建受管理资源库

系统 SHALL 支持在指定本地路径创建一个受管理资源库, 包含 `manifest.json`, `library.sqlite`, `originals`, `derivatives`, `sidecars`, `exports` 和 `trash`.

#### Scenario: 创建新资源库

- **WHEN** 用户请求在空目录创建资源库并提供名称
- **THEN** 系统创建资源库目录结构, 写入 manifest, 初始化 SQLite schema, 并返回资源库 id 和路径

### Requirement: 打开和校验资源库

系统 SHALL 在打开资源库时校验 manifest, schema version 和必要目录, 并拒绝打开不兼容或损坏的资源库.

#### Scenario: 打开 schema 不兼容的资源库

- **WHEN** 用户打开 schema version 高于当前应用支持版本的资源库
- **THEN** 系统返回 `SchemaMismatch` 错误且不修改资源库内容

### Requirement: 管理资源库 registry

系统 SHALL 维护本机 app-level registry, 记录最近打开的资源库, display name, root path, hidden 状态和 last opened time.

#### Scenario: 隐藏资源库

- **WHEN** 用户隐藏一个已注册资源库
- **THEN** 系统将该资源库从默认列表中隐藏, 但不删除资源库目录或 SQLite 数据

### Requirement: 导入和导出资源库内容

系统 SHALL 支持将外部图片导入 managed library, 并支持按资源库或相册导出图片和 sidecar metadata.

#### Scenario: 导入外部图片

- **WHEN** 用户导入一个本地图片文件
- **THEN** 系统复制图片到 `originals`, 计算 hash, 写入 SQLite, 并且后续不依赖原始外部路径

### Requirement: 校验资源库完整性

系统 SHALL 支持检查 SQLite 记录与 managed file layout 的一致性, 包括文件存在性和 hash.

#### Scenario: 原始文件丢失

- **WHEN** integrity check 发现某个 asset version 的文件不存在
- **THEN** 系统报告 `FileIntegrityMismatch` 并标明受影响的 version id
