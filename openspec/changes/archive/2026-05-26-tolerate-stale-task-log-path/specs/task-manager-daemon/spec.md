## MODIFIED Requirements

### Requirement: Task Logs 和 Structured Timeline

系统 SHALL 为每个 task attempt 维护 app-owned log, 并为 task 维护 structured timeline, task detail 可以读取 live log tail 和 raw log preview. 系统 MUST 保持 task 结构化 detail 与 attempt log tail 的可用性边界: task detail 的状态, attempts, timeline 和 output links 不得因为历史 attempt log path 不属于当前 daemon app-owned log root 而不可用.

#### Scenario: Running Task Log Tail

- **WHEN** client 请求 running task 的 log tail
- **THEN** daemon 返回当前 attempt 的 app-owned log tail, 并拒绝读取非 task attempt log path

#### Scenario: Stale Task Log Path Does Not Hide Detail

- **WHEN** client 已能读取 task detail, 但该 task 的历史 attempt log path 不属于当前 daemon app-owned log root
- **THEN** task detail 的结构化信息仍可被 Desktop 展示
- **AND** daemon 不得返回该越界 log path 的文件内容

#### Scenario: Timeline 展示 Retry

- **WHEN** task 发生失败并安排 retry
- **THEN** task events 包含 attempt failed 和 retry scheduled, UI 可展示失败原因和下一次 retry 时间

#### Scenario: Completed Task Outputs

- **WHEN** task completed
- **THEN** task detail 展示 output links, 包含 asset, version, generation event, metadata suggestion 或 field result
