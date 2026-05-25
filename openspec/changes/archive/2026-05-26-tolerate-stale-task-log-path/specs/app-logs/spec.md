## MODIFIED Requirements

### Requirement: Task Detail 提供 Task Attempt Logs

系统 SHALL 将 task attempt logs 作为 task detail 的主要日志入口, 并通过 daemon 或 desktop API 读取 app-owned log content. 当 task detail 的结构化信息可用但 attempt log tail 因 stale path, missing file 或当前 runtime root 不匹配而不可读时, 桌面应用 MUST 展示 task detail 并在日志区域展示可恢复的 unavailable message, 不得把整个 task detail 视为加载失败.

#### Scenario: 查看 Task Attempt Log Preview

- **WHEN** 用户在 Task Detail 中选择某个 attempt
- **THEN** 桌面应用展示该 attempt 的 raw log preview, 包含 stdout/stderr 或 provider adapter 记录的执行输出

#### Scenario: 查看 Running Task Live Log Tail

- **WHEN** 用户查看 running task detail
- **THEN** 桌面应用展示当前 attempt 的 live log tail, 并随着 daemon 返回的新内容更新

#### Scenario: Task Log Tail 不可用时保留 Detail

- **WHEN** 用户查看 task detail 且 daemon 拒绝读取 stale 或越界 attempt log path
- **THEN** 桌面应用展示 task status, attempts, timeline events 和 output links
- **AND** 日志区域展示 log tail unavailable message

#### Scenario: 拒绝读取非 Task Log

- **WHEN** client 请求读取不属于 app-owned task attempt 的路径
- **THEN** daemon 或 desktop log API 拒绝请求, 且不返回文件内容
