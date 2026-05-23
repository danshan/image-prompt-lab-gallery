## Purpose

Define local daemon task management, scheduling, retry, recovery, logs, and structured task timeline behavior.
## Requirements
### Requirement: 提供本地 Task Manager Daemon
系统 SHALL 提供独立本地 daemon 执行长任务, daemon 只能通过 loopback local HTTP 接受本机 client 请求, 且所有请求必须通过本地 session token 鉴权. daemon implementation MUST keep request parsing and authentication outside expensive mutable state sections where possible.

#### Scenario: Desktop 启动并发现 Daemon
- **WHEN** desktop app 启动且 runtime file 指向健康 daemon
- **THEN** desktop app 通过 `/v1/health` 确认 API version 和 daemon 状态, 并复用该 daemon

#### Scenario: Desktop 启动 Daemon
- **WHEN** desktop app 找不到健康 daemon
- **THEN** desktop app 启动 app-owned daemon sidecar, 读取 runtime file 中的 port 和 token location, 并通过 loopback API 连接

#### Scenario: 拒绝非授权请求
- **WHEN** local HTTP request 未携带有效 session token
- **THEN** daemon 拒绝请求, 且不得返回 task, library 或日志内容

#### Scenario: 只绑定 Loopback
- **WHEN** daemon 启动 HTTP listener
- **THEN** daemon 只绑定 loopback address, 不暴露局域网或远程访问接口

### Requirement: 持久化 Task Model

系统 SHALL 在 resource library 中持久化 task, attempt, event 和 output link, 使 task history 与 asset, version, generation event 和 metadata suggestion 一起迁移和审计.

#### Scenario: Batch Enqueue Tasks

- **WHEN** client 提交多个 task inputs
- **THEN** daemon 为每个 input 创建持久化 task, 分配稳定 queue position, 写入 submitted event, 并返回 task views

#### Scenario: 查询 Task Detail

- **WHEN** client 请求某个 task detail
- **THEN** daemon 返回 task identity, status, input snapshot, attempts, timeline events, output links, last error 和 wait reason

#### Scenario: Output Link 持久化

- **WHEN** task 成功创建 asset version, generation event, metadata suggestion 或 field result
- **THEN** daemon 将 output link 写入 task outputs, 以便 Queue 和 Review Inbox 反向追踪来源 task

### Requirement: 支持 Task State Machine

系统 SHALL 使用明确 task state machine 管理 queued, running, retry waiting, failed, canceled, interrupted 和 completed 状态.

#### Scenario: Task 成功完成

- **WHEN** queued task 被 scheduler 领取并成功提交 output
- **THEN** task 状态依次记录 running 和 completed, 并写入 attempt started 和 completed timeline events

#### Scenario: Task Transient Failure

- **WHEN** running task 因 transient error 失败且未达到 max attempts
- **THEN** daemon 将 task 转为 retry waiting, 记录 retry scheduled event, 并设置 next retry time

#### Scenario: Task Final Failure

- **WHEN** running task 因 invalid params, unsupported capability 或 missing input version 失败
- **THEN** daemon 将 task 转为 failed final, 记录 error classification, 且不自动 retry

#### Scenario: Cancel Queued Task

- **WHEN** client cancel queued task
- **THEN** daemon 将 task 标记为 canceled, 且该 task 不再被 scheduler 领取

#### Scenario: Cancel Running Task

- **WHEN** client cancel running task
- **THEN** daemon 将 task 标记为 cancel requested 并 best-effort 终止 attempt, 最终写入 canceled 或 completed after cancel requested 状态

### Requirement: Scheduler 支持并发限制和人工排序

系统 SHALL 由 daemon scheduler 根据全局并发, provider 并发, priority 和 queue position 选择可执行 task, 并只允许人工重排 queued tasks.

#### Scenario: Provider Slot 已满

- **WHEN** codex-cli provider 已达到并发上限且存在 queued codex-cli task
- **THEN** daemon 保持该 task queued, 并写入 wait reason 表示正在等待 codex-cli slot

#### Scenario: Global Slot 已满

- **WHEN** global running task 数量已达到全局并发上限
- **THEN** daemon 不再启动新的 task, 并为可执行 queued task 返回 global concurrency wait reason

#### Scenario: Reorder Queued Tasks

- **WHEN** client 对 queued tasks 提交新的 queue order
- **THEN** daemon 更新这些 queued tasks 的 queue position, 且 scheduler 按新顺序领取 task

#### Scenario: Reject Reorder Running Task

- **WHEN** client 尝试重排 running, retry waiting, completed 或 failed task
- **THEN** daemon 拒绝该 reorder 请求或忽略非 queued task, 且不得改变执行顺序

### Requirement: Retry Policy 区分 Transient 和 Non-Transient Errors

系统 SHALL 只对 transient errors 自动 retry, 对 non-transient errors 保留 manual retry 或 duplicate 入口.

#### Scenario: 自动 Retry Transient Error

- **WHEN** task attempt 因 timeout, rate limit, temporary network failure 或 worker crash before commit 失败
- **THEN** daemon 按 backoff policy 安排下一次 attempt, 并展示 next retry time

#### Scenario: 不自动 Retry Non-Transient Error

- **WHEN** task attempt 因 unsupported capability, invalid params, missing input version 或 schema validation failure 失败
- **THEN** daemon 将 task 标记为 failed final 或 manual retryable, 且不自动消耗下一次 attempt

#### Scenario: Manual Retry

- **WHEN** 用户对 failed task 执行 retry
- **THEN** daemon 在同一个 task id 下创建新 attempt, 保留既有 task history 和 timeline

#### Scenario: Duplicate Task

- **WHEN** 用户 duplicate task
- **THEN** daemon 或 desktop 创建新的 task draft 或 queued task, 拷贝 input snapshot, 但使用新的 task id 和 queue position

### Requirement: Task Logs 和 Structured Timeline

系统 SHALL 为每个 task attempt 维护 app-owned log, 并为 task 维护 structured timeline, task detail 可以读取 live log tail 和 raw log preview.

#### Scenario: Running Task Log Tail

- **WHEN** client 请求 running task 的 log tail
- **THEN** daemon 返回当前 attempt 的 app-owned log tail, 并拒绝读取非 task attempt log path

#### Scenario: Timeline 展示 Retry

- **WHEN** task 发生失败并安排 retry
- **THEN** task events 包含 attempt failed 和 retry scheduled, UI 可展示失败原因和下一次 retry 时间

#### Scenario: Completed Task Outputs

- **WHEN** task completed
- **THEN** task detail 展示 output links, 包含 asset, version, generation event, metadata suggestion 或 field result

### Requirement: Daemon Recovery

系统 SHALL 在 daemon 启动时恢复 resource library 中非终态 tasks, 并避免重复提交已 committed output.

#### Scenario: 恢复 Queued Task

- **WHEN** daemon 启动并发现 queued task
- **THEN** task 保持 queued, 可继续被 scheduler 领取

#### Scenario: 恢复 Retry Waiting Task

- **WHEN** daemon 启动并发现 retry waiting task
- **THEN** daemon 根据 next retry time 保持等待或转回 queued

#### Scenario: 恢复 Running Task Without Output

- **WHEN** daemon 启动并发现 running task 且没有 confirmed output link
- **THEN** daemon 将 task 标记为 interrupted retryable 或 interrupted final, 并记录 recovery event

#### Scenario: Reconcile Committed Output

- **WHEN** daemon 启动并发现 running task 已存在 confirmed output link
- **THEN** daemon 将 task reconcile 为 completed, 且不得再次执行 provider request

### Requirement: Daemon HTTP 查询在长任务期间保持响应
daemon SHALL 在长任务执行和 scheduler activity 期间保持 health, capabilities 和 task read API 可响应, 并避免为不需要 mutable state 的请求持有全局 state lock.

#### Scenario: Health Check 不等待长任务
- **WHEN** daemon 正在执行长任务
- **THEN** `/v1/health` 请求仍能快速返回 daemon health 和 API version

#### Scenario: Capabilities Check 不等待 Scheduler Tick
- **WHEN** scheduler tick 正在评估任务
- **THEN** `/v1/capabilities` 请求不得因为无关 mutable state lock 长时间阻塞

### Requirement: Scheduler 无任务时避免深拷贝 Daemon State
daemon scheduler SHALL 在无 opened library 或无 eligible task 时避免深拷贝完整 daemon state.

#### Scenario: 无 Opened Library
- **WHEN** daemon 没有 opened libraries
- **THEN** scheduler tick 直接返回 no work, 不 clone 完整 `DaemonState`

#### Scenario: 无 Runnable Task
- **WHEN** opened libraries 中没有 queued 或 retry-ready task
- **THEN** scheduler tick 避免执行昂贵 snapshot, 并保持下一轮 tick 可继续检查

### Requirement: Daemon Client Timeout 和 Backoff 区分请求类型
desktop daemon client SHALL 根据请求类型使用合适 timeout 和 transient failure backoff.

#### Scenario: Health Check 使用短 Timeout
- **WHEN** desktop 检查 daemon health
- **THEN** client 使用短 timeout, 失败后可尝试启动 sidecar

#### Scenario: Task Detail 使用较长 Timeout
- **WHEN** desktop 请求 task detail 或 log preview
- **THEN** client 使用足够长的 timeout, 避免在 daemon 短暂繁忙时误报失败

### Requirement: Daemon implementation SHALL separate transport, routing, scheduling, and execution
The daemon SHALL keep loopback HTTP transport, route dispatch, scheduler orchestration, task execution, runtime state, log handling, and response view mapping in separate implementation boundaries. These boundaries SHALL be represented as real Rust modules with explicit imports and `pub(crate)` sharing where needed, rather than root-level `include!` composition.

#### Scenario: Route behavior remains stable after split
- **WHEN** daemon route handling is refactored into route modules
- **THEN** existing endpoint paths, token authentication behavior, response status codes, and response JSON shapes must remain compatible

#### Scenario: Scheduler changes are isolated
- **WHEN** scheduler runnable-work checks, recovery, retry, or tick execution changes
- **THEN** the change must be implementable in scheduler or executor modules without editing raw HTTP parsing code

#### Scenario: Executor changes are isolated
- **WHEN** image generation or metadata task execution changes
- **THEN** the change must be implementable in executor modules without editing route parsing or runtime file handling

#### Scenario: Module boundaries are explicit
- **WHEN** developers inspect `crates/imglab-daemon/src/lib.rs`
- **THEN** it declares explicit modules for runtime, transport, routing, scheduling, execution, logs, task DTOs and views
- **AND** implementation files use normal Rust imports instead of depending on a shared root `include!` scope

### Requirement: Task transition ownership is centralized

Task status transition, retry classification, attempt lifecycle, event persistence, and output link semantics SHALL have a single core owner. The daemon scheduler SHALL execute ticks, provider process boundaries, cancellation markers, and log IO without duplicating core task decisions.

#### Scenario: Scheduler delegates task decisions

- **WHEN** the daemon executes, retries, cancels, or completes a task
- **THEN** state transition and output-link semantics are delegated to the core task/generation owner
- **AND** daemon-specific code only handles runtime execution concerns

