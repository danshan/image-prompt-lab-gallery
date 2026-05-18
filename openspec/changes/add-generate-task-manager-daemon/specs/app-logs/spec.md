## ADDED Requirements

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
系统 SHALL 保留 Settings Logs 作为 app-owned logs 的全局浏览入口, 但 task execution 排查应优先通过 Task Detail 展示结构化 timeline 和 attempt logs.

#### Scenario: Settings Logs 展示 Task Logs
- **WHEN** 用户打开 Settings Logs
- **THEN** 桌面应用可以展示 task attempt logs 和 metadata generation logs, 包含 kind, modified time, size 和 preview

#### Scenario: Task Detail 提供上下文
- **WHEN** 用户从 Task Detail 查看日志
- **THEN** 桌面应用同时展示 task status, attempts, timeline events 和 output links, 不要求用户回到 Settings 才能理解失败原因
