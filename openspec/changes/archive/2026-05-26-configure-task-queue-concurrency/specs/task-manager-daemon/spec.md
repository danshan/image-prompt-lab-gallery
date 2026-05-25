## ADDED Requirements

### Requirement: Daemon 提供 Task Queue 并发配置

daemon SHALL expose an app-level task queue concurrency setting that controls the scheduler global maximum number of simultaneously running tasks. The setting MUST be validated before persistence, MUST survive daemon restart when app-level settings storage is available, and MUST fall back to the default scheduler configuration when the persisted value is missing or unreadable.

#### Scenario: 读取 Task Queue 并发配置
- **WHEN** desktop client 请求当前 task queue settings
- **THEN** daemon 返回当前最大并发数, 默认值, 最小值, 最大值和当前 effective scheduler value

#### Scenario: 更新 Task Queue 并发配置
- **WHEN** desktop client 提交合法最大并发数
- **THEN** daemon 持久化该 app-level setting
- **AND** 后续 scheduler tick 使用新的全局最大并发数

#### Scenario: 拒绝非法并发配置
- **WHEN** desktop client 提交小于最小值, 大于最大值或无法解析为整数的最大并发数
- **THEN** daemon 拒绝该请求并返回可恢复 validation error
- **AND** 当前 effective scheduler value 保持不变

#### Scenario: 降低并发数不取消 Running Tasks
- **WHEN** 用户将最大并发数降低到低于当前 running task count
- **THEN** daemon 不取消已经 running 的 tasks
- **AND** scheduler 不再启动新 task, 直到 running task count 低于新的最大并发数

## MODIFIED Requirements

### Requirement: Scheduler 支持并发限制和人工排序

系统 SHALL 由 daemon scheduler 根据可配置全局并发, provider 并发, priority 和 queue position 选择可执行 task, 并只允许人工重排 queued tasks. Scheduler MUST support multiple simultaneously running tasks when global and provider slots are available, and MUST keep provider-level concurrency limits as safety constraints even when the global maximum is higher.

#### Scenario: Provider Slot 已满

- **WHEN** codex-cli provider 已达到并发上限且存在 queued codex-cli task
- **THEN** daemon 保持该 task queued, 并写入 wait reason 表示正在等待 codex-cli slot

#### Scenario: Global Slot 已满

- **WHEN** global running task 数量已达到当前配置的全局并发上限
- **THEN** daemon 不再启动新的 task, 并为可执行 queued task 返回 global concurrency wait reason

#### Scenario: Multiple Global Slots Available

- **WHEN** 当前 running task 数量低于配置的全局并发上限
- **AND** 多个 queued tasks 满足 provider 并发, priority 和 queue position 规则
- **THEN** daemon scheduler 启动多个 tasks, 直到没有 eligible task 或达到全局并发上限

#### Scenario: Reorder Queued Tasks

- **WHEN** client 对 queued tasks 提交新的 queue order
- **THEN** daemon 更新这些 queued tasks 的 queue position, 且 scheduler 按新顺序领取 task

#### Scenario: Reject Reorder Running Task

- **WHEN** client 尝试重排 running, retry waiting, completed 或 failed task
- **THEN** daemon 拒绝该 reorder 请求或忽略非 queued task, 且不得改变执行顺序
