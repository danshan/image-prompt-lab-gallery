## Purpose

Define app-owned generation log listing and safe log content reading behavior.
## Requirements
### Requirement: App-owned 日志列表
系统 SHALL 提供查询最近 app-owned 生成日志的能力. 日志列表 SHALL 至少包含日志路径, 日志类型, 修改时间, 文件大小和内容预览. 系统 MUST 支持 Codex image generation 日志和 Codex metadata generation 日志. 日志列表实现 MUST 只扫描 app-owned roots, 不得扫描整个 system temp directory.

#### Scenario: 查询最近 App Logs
- **WHEN** 桌面应用请求最近 app logs
- **THEN** 系统返回按修改时间倒序排列的 app-owned 日志列表

#### Scenario: 日志类型归类
- **WHEN** 日志文件名匹配 `imglab-codex-cli-*.log`
- **THEN** 系统将该日志标记为 `codex-image-generation`

#### Scenario: Metadata 日志类型归类
- **WHEN** 日志文件名匹配 `imglab-codex-metadata-*.log`
- **THEN** 系统将该日志标记为 `codex-metadata-generation`

#### Scenario: 不扫描 System Temp Root
- **WHEN** Settings Logs 刷新日志列表
- **THEN** 系统只枚举 app-owned log roots 和已知 provider log roots, 不得遍历整个 system temp directory

### Requirement: 安全读取 App Log 内容
系统 SHALL 提供读取单个 app-owned 日志内容的能力. 系统 MUST 只允许读取允许目录中匹配已知 app log pattern 的文件, 并 MUST 拒绝任意其他路径.

#### Scenario: 读取允许的日志
- **WHEN** 桌面应用请求读取允许目录中匹配 app log pattern 的日志
- **THEN** 系统返回该日志的内容预览

#### Scenario: 拒绝任意路径读取
- **WHEN** 桌面应用请求读取不在允许目录或不匹配 app log pattern 的路径
- **THEN** 系统拒绝读取并返回可恢复错误

#### Scenario: 限制大型日志读取
- **WHEN** app-owned 日志文件超过内容预览上限
- **THEN** 系统只返回安全大小范围内的内容预览, 不阻塞 Settings UI

### Requirement: Task Detail 提供 Task Attempt Logs

系统 SHALL 将 task attempt logs 作为 task detail 的主要日志入口, 并通过 daemon 或 desktop API 读取 app-owned log content.

#### Scenario: 查看 Task Attempt Log Preview

- **WHEN** 用户在 Task Detail 中选择某个 attempt
- **THEN** 桌面应用展示该 attempt 的 raw log preview, 包含 stdout/stderr 或 provider adapter 记录的执行输出

#### Scenario: 查看 Running Task Live Log Tail

- **WHEN** 用户查看 running task detail
- **THEN** 桌面应用展示当前 attempt 的 live log tail, 并随着 daemon 返回的新内容更新

#### Scenario: 拒绝读取非 Task Log

- **WHEN** client 请求读取不属于 app-owned task attempt 的路径
- **THEN** daemon 或 desktop log API 拒绝请求, 且不返回文件内容

### Requirement: Settings Logs 保留全局日志浏览

系统 SHALL 保留 `Settings / Logs` 作为 app-owned logs 的全局浏览入口, 但 task execution 排查应优先通过 Task Detail 展示结构化 timeline, attempts, output links 和 attempt logs. `Settings / Logs` MUST 与 library lifecycle 操作和 provider diagnostics 分区清晰.

#### Scenario: Settings Logs 展示 App Logs
- **WHEN** 用户打开 `Settings / Logs`
- **THEN** 桌面应用展示 app-owned logs, task attempt logs 和 metadata generation logs, 包含 kind, modified time, size 和 preview

#### Scenario: Settings Log Deep Links To Task
- **WHEN** 某条 log 可关联到 daemon task 或 task attempt
- **THEN** Settings Logs 提供 Open task detail 入口

#### Scenario: Task Detail 提供主要调试上下文
- **WHEN** 用户从 Task Detail 查看日志
- **THEN** 桌面应用同时展示 task status, attempts, timeline events 和 output links, 不要求用户回到 Settings 才能理解失败原因

### Requirement: Diagnostics Overview

系统 SHALL 为 Settings 和 Library Context 提供 diagnostics overview, 至少包含 daemon status, provider health summary, recent app log summary 和可恢复配置错误. Diagnostics overview MUST NOT 暴露任意文件系统扫描能力, 仍只读取 app-owned roots 和已知 provider log roots.

#### Scenario: 查看 Provider Diagnostics
- **WHEN** 用户打开 Settings diagnostics
- **THEN** UI 展示 provider health, credential/capability status, daemon status 和可恢复错误信息

#### Scenario: Diagnostics 不扫描 System Temp Root
- **WHEN** 系统刷新 diagnostics overview
- **THEN** 系统只枚举 app-owned log roots 和已知 provider log roots, 不遍历整个 system temp directory

### Requirement: Logs Workflow 状态覆盖

Settings Logs 和 diagnostics SHALL 覆盖 loading, empty, error 和 recovery states.

#### Scenario: Logs Empty State
- **WHEN** 当前没有 app-owned logs
- **THEN** Settings Logs 展示 empty state, 不展示旧 log preview

#### Scenario: Log Preview Read Error
- **WHEN** log preview 读取失败或路径不被允许
- **THEN** Settings Logs 展示可恢复错误, 并保留 logs list 可用
