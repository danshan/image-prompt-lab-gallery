## MODIFIED Requirements

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
