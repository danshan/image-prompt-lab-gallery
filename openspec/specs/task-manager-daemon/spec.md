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

### Requirement: Daemon task manager routes through application task owner

Daemon task manager workflows SHALL route task operations through the application task owner while preserving loopback API behavior.

#### Scenario: Task HTTP routes use task owner

- **WHEN** daemon HTTP routes create, list, reorder, cancel, retry, duplicate, or inspect tasks
- **THEN** task persistence operations SHOULD go through the task application owner
- **AND** HTTP request parsing and response mapping remain daemon-owned

### Requirement: 支持 Task State Machine

系统 SHALL 使用明确 task state machine 管理 queued, running, retry waiting, failed, canceled, interrupted 和 completed 状态.

#### Scenario: Daemon delegates task transition decisions

- **WHEN** daemon recovery, cancel, successful attempt, failed attempt, or canceled attempt paths decide the next task status
- **THEN** daemon SHOULD use the core task domain policy to decide the status
- **AND** daemon MAY still own cancellation marker IO, retry timestamp calculation, event persistence orchestration, and HTTP response mapping

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

### Requirement: Background Automation Daemon
系统 SHALL 支持 background automation daemon. 用户在 Settings 开启后, 系统 MUST 安装或启用 OS-level background agent, 使 daemon 能在 desktop app 未启动时继续执行 automation-enabled libraries 中的 scheduled generation jobs.

#### Scenario: Enable Background Daemon
- **WHEN** 用户在 Settings 开启 background daemon
- **THEN** 系统启用 macOS LaunchAgent
- **AND** daemon 启动后写入 runtime file 和 token file

#### Scenario: Disable Background Daemon
- **WHEN** 用户在 Settings 关闭 background daemon
- **THEN** 系统禁用 LaunchAgent
- **AND** daemon 停止领取新的 schedules 和 tasks
- **AND** 当前 running attempt 默认允许 graceful drain

### Requirement: Automation Enabled Library Scanning
Background daemon SHALL only open and scan libraries that are explicitly marked `automation_enabled`.

#### Scenario: Scan Opted-In Libraries
- **WHEN** background daemon 启动
- **THEN** daemon 从 app registry 读取 automation-enabled libraries
- **AND** 只为这些 libraries 执行 schedule runner

#### Scenario: Skip Non-Opted-In Libraries
- **WHEN** library 未开启 automation
- **THEN** background daemon 不得打开或扫描该 library 的 schedules

### Requirement: Schedule Runner Loop
Daemon SHALL run a schedule runner loop separate from the existing task scheduler loop. Schedule runner SHALL create scheduled generation runs, perform prompt expansion, enqueue image generation tasks, reconcile linked task completion, and perform output post-processing.

#### Scenario: Due Schedule Enqueues Task
- **WHEN** schedule runner 发现 due scheduled generation job
- **THEN** daemon 创建 run
- **AND** daemon 创建普通 image generation task

#### Scenario: Linked Task Completes
- **WHEN** linked image generation task completed
- **THEN** schedule runner 执行 album/tag post-processing
- **AND** run 更新为 completed

### Requirement: Daemon Discovery Priority
Desktop daemon client SHALL prefer a healthy background daemon over app-owned sidecar when background daemon is enabled.

#### Scenario: Background Daemon Healthy
- **WHEN** desktop app 启动且 background daemon runtime file 指向健康 daemon
- **THEN** desktop app 复用 background daemon
- **AND** 不启动重复 sidecar

#### Scenario: Background Daemon Misconfigured
- **WHEN** background daemon enabled 但 health check 失败
- **THEN** Settings Automation 展示 recoverable diagnostic
- **AND** UI 提供 repair action

### Requirement: Daemon startup failures are bounded and actionable
Desktop daemon startup and background daemon service control SHALL fail with recoverable diagnostics when required local daemon infrastructure is missing or OS service commands do not return promptly.

#### Scenario: Daemon binary missing before sidecar startup
- **WHEN** desktop needs to start an app-owned daemon sidecar and the daemon binary cannot be found
- **THEN** desktop returns a recoverable error that identifies the missing daemon binary condition
- **AND** desktop MUST NOT attempt to spawn a non-existent sidecar process

#### Scenario: Daemon binary missing before background daemon installation
- **WHEN** Settings Automation attempts to start, repair, or install background daemon support and the daemon binary cannot be found
- **THEN** desktop returns a recoverable error before writing a LaunchAgent that points to a non-existent binary

#### Scenario: Packaged app locates bundled daemon binary
- **WHEN** desktop runs from a packaged app bundle
- **THEN** daemon binary discovery includes the app bundle resources location and Tauri external sidecar target-triple filename
- **AND** release packaging includes the daemon binary in that location

#### Scenario: Background daemon service command times out
- **WHEN** an OS-level background daemon service command does not complete within the bounded service-control timeout
- **THEN** desktop returns a recoverable timeout diagnostic
- **AND** Settings Automation controls become usable again after the command returns
