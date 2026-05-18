# Generate Task Manager Daemon Design

## Goal

本设计重新定义 Generate 交互, 使 Generate, Tasks Queue 和 Review Inbox 形成完整闭环. 新系统需要支持多任务提交, 人工排序, 并发控制, 自动重试, 执行日志, 任务详情, 结构化 timeline, 以及生成结果到 Review Inbox 的可追溯跳转.

本次范围选择 **immediate daemon**: 设计并实现独立 worker / daemon, desktop 通过 loopback local HTTP 调度任务. 这不是单纯 UI 改版, 而是本地任务执行架构升级.

## Current Context

当前 desktop 已有:

- `GenerationComposer`: 单 prompt 输入和 provider 选择.
- `GenerationQueue`: 前端内存队列, 展示 provider, prompt, status, log path 和 error.
- Tauri `GenerationJobState`: in-memory job map, 由 Tauri 线程直接执行 generation.
- `Settings Logs`: app-owned Codex image generation 和 metadata generation logs 浏览.
- `Review Inbox`: pending suggestions, field-level metadata regeneration, full suggestion regeneration, batch accept / reject, suggestion history.

现状问题:

- Generation queue 不持久化, app 重启后 job 状态丢失.
- Scheduler 语义不明确, 没有全局 / provider 并发上限.
- Retry, cancel, duplicate, reorder 没有统一任务状态机.
- Image generation 和 Review metadata generation 是两套异步模型.
- 日志在 Settings 中可查看, 但 task detail 不能解释执行过程, retry 和 output links.

## Architecture

推荐架构是 **independent local daemon + daemon-owned scheduler + desktop client UI**.

模块边界:

- `imglab-core`: domain model, library persistence, generation / metadata review service boundary, task schema, retry classification, scheduler policy.
- `imglab-daemon`: 本地 daemon / worker, 负责 task scheduler, provider execution, retry, concurrency slots, timeline events, log streaming, interrupted task recovery.
- `apps/desktop/src-tauri`: daemon lifecycle, client calls, local token handling, error mapping. 不再直接执行 long-running generation / metadata jobs.
- `apps/desktop/src`: Queue-Centric Generate workspace, batch draft composer, task queue reorder, task detail, structured timeline, live logs, Review Inbox cross-links.
- CLI: 后续可通过同一 local HTTP API 提交和观察 tasks, 不需要复用 Tauri commands.

Daemon transport 选择 **loopback local HTTP**:

- 只绑定 `127.0.0.1` 或平台等价 loopback address.
- 使用本地 session token, token 不写日志.
- 提供 versioned API, 例如 `/v1`.
- Desktop 启动时发现或启动 daemon sidecar.
- API contract 不依赖 Tauri, 便于 CLI 复用.

明确不引入:

- Cloud sync.
- 多用户协作.
- 远程 daemon.
- 通用 HTTP API server 暴露到局域网.

## Task Types

统一任务模型覆盖三类 task:

- `image_generation`: 文生图 / 图生图. 输出 asset version, generation event, metadata suggestion.
- `metadata_field_generation`: Review Inbox 单字段再生成. 输出 field result, 由 Review UI 安全 apply 到本地 draft.
- `metadata_suggestion_generation`: 整条 suggestion 再生成. 输出新的 pending suggestion record, 保留历史.

短事务仍不进入 task manager:

- accept suggestion.
- reject suggestion.
- add asset to album.
- update rating or canonical metadata.

这些操作应继续通过 core service 直接执行, 因为它们是可预期的短事务写操作, 不需要排队, retry backoff 或 live logs.

## Task State Machine

核心状态:

```text
draft -> queued -> running -> completed
              \-> retry_waiting -> queued
              \-> cancel_requested -> canceled
              \-> failed_retryable -> retry_waiting | queued
              \-> failed_final
running -> interrupted_retryable | interrupted_final
```

状态语义:

- `draft`: 只存在于 desktop UI, 未提交 daemon. 支持多行 prompt, 参数快照, batch enqueue.
- `queued`: 已持久化, 未开始执行. 只有这个状态支持人工排序.
- `running`: daemon 已领取 slot 并创建 attempt. 不允许排序, 可请求 cancel.
- `retry_waiting`: transient failure 后等待 backoff 到期. 不参与人工排序, 到期后回到 `queued`.
- `failed_retryable`: 自动 retry 耗尽后仍可 manual retry.
- `failed_final`: invalid params, unsupported capability, missing input version, non-recoverable schema validation failure 等不可自动 retry.
- `cancel_requested`: running task 的 best-effort cancel 已发出.
- `canceled`: commit 前取消成功, 或 queued / retry_waiting 直接取消.
- `completed`: 输出 links 已写入, task 终态.
- `interrupted_retryable`: daemon crash / host reboot 后, running task 没有 confirmed committed output, 且可以安全重试.
- `interrupted_final`: daemon crash / host reboot 后, output 状态不确定或需要人工处理.

Cancel 语义:

- `queued`: 直接变 `canceled`.
- `retry_waiting`: 直接变 `canceled`.
- `running`: 变 `cancel_requested`, daemon best-effort 终止 attempt.
- 如果 cancel requested 后 output 已经 committed, task 不能静默删除结果. 第一版显示 `completed_after_cancel_requested`, 并在 timeline 中记录冲突事实.

Retry / duplicate 语义:

- `Retry`: 保留同 task id 和 task history, 新增 attempt.
- `Duplicate`: 创建新 task id, 拷贝 input snapshot, 进入 draft 或 queued.

## Task Data Model

每个 task 需要包含:

- Identity: `task_id`, `library_id`, `task_type`, `created_by`, `created_at`, `updated_at`.
- Execution: `status`, `queue_position`, `priority`, `provider`, `operation`, `concurrency_group`, `attempt_count`, `max_attempts`, `next_retry_at`.
- Input snapshot: prompt, negative prompt, provider params, input version id, review suggestion id, review field, base revision.
- Output links: generation event id, asset id, version id, suggestion id, field result id or serialized field output.
- Observability: current attempt id, log path, timeline events, last error, error classification, wait reason.

建议新增 library tables:

```text
tasks
task_attempts
task_events
task_outputs
```

任务持久化落在 resource library, 因为 task 与 asset, version, suggestion, generation event 强绑定. Daemon 自身只保存 runtime discovery 和进程状态.

## Scheduler

Scheduler 由 daemon 拥有, desktop 不能自行决定执行顺序.

默认策略:

- Global concurrency limit: `2`.
- Provider concurrency limit: `codex-cli = 1`, `fake = 4`.
- Concurrency group: image generation 和 metadata generation 可以共享 provider slot. 第一版建议所有 Codex CLI image / metadata tasks 共用 `codex-cli` slot.
- Queue selection: `priority desc, queue_position asc, created_at asc`.
- Manual reorder: 只允许 `queued` tasks. Running, retry waiting, completed, failed 不参与人工排序.
- Wait reason: daemon 负责写入可解释原因, 例如 `Waiting for global concurrency slot`, `Waiting for codex-cli slot`, `Waiting until retry backoff expires`.

Retry policy:

- 仅 transient errors 自动重试.
- 默认 `max_attempts = 3`.
- Backoff 示例: `30s`, `2m`, `5m`, with jitter.
- 自动 retry 必须记录 timeline events: `attempt_failed`, `retry_scheduled`, `attempt_started`.

Transient examples:

- process timeout.
- rate limit.
- temporary network failure.
- worker crash before commit.
- daemon interrupted before confirmed output.

Non-transient examples:

- invalid prompt / params.
- unsupported provider capability.
- missing input version.
- credential missing until user fixes settings.
- output parse failure after provider returned invalid contract.
- JSON schema prompt parse failure that requires prompt / constraint change.

## Local HTTP API

第一版 API 保持窄接口:

```text
GET  /v1/health
GET  /v1/capabilities
POST /v1/libraries/open
POST /v1/tasks
POST /v1/tasks/batch
GET  /v1/tasks?library_id=...
GET  /v1/tasks/{task_id}
POST /v1/tasks/{task_id}/cancel
POST /v1/tasks/{task_id}/retry
POST /v1/tasks/{task_id}/duplicate
POST /v1/tasks/reorder
GET  /v1/tasks/{task_id}/events
GET  /v1/tasks/{task_id}/logs/tail
```

端口发现:

- Daemon 写入 app-owned runtime file, 包含 port, token file path, daemon pid, API version.
- Desktop 优先读取 runtime file 并 health check.
- Health check 失败则启动 daemon sidecar.
- Local HTTP 只接受 loopback request.
- Token 不写入日志, 不展示在 UI.

后续可以增加 SSE 或 WebSocket. 第一版可以用 polling + log tail endpoint, 只要 UI 能展示近实时状态.

## Generate UI

主视图采用 Queue-Centric 三栏 workspace:

### Batch Composer

左栏是 explicit draft composer:

- `Add task` 创建 draft card.
- 每个 draft card 拥有独立多行 prompt editor.
- 每个 draft card 保存 provider, operation, params, source version snapshot.
- 支持 duplicate draft, remove draft.
- 支持 structured JSON import 作为高级入口, 但不是主路径.
- `Enqueue all` 将多个 draft 提交到 daemon.

JSON import 的用途是显式批量创建 draft, 不能把 newline 或 blank line 作为 task delimiter. 导入后仍进入 composer 供用户检查和修改.

### Tasks Queue

中栏展示 tasks:

- running.
- queued.
- retry waiting.
- completed.
- failed.
- canceled.

每条 task 显示:

- task type.
- prompt 摘要.
- provider / operation.
- status.
- wait reason.
- attempt count.
- next retry time.
- quick actions.

只有 `queued` tasks 展示 drag handle / move up / move down.

### Task Detail

右栏展示 selected task:

- header: status, provider, attempts, actions.
- input snapshot: prompt, params, source version, review suggestion / field.
- structured timeline.
- live log tail.
- raw log preview.
- output links.
- retry / duplicate / cancel actions.

## Review Inbox Integration

Image generation completed 后:

- task output 链接到 asset, version, generation event.
- 如果创建 pending metadata suggestion, output 同时链接 suggestion.
- Queue row 和 Task Detail 提供 `Open asset` 和 `Open review suggestion`.

Review metadata generation:

- field-level regeneration 创建 `metadata_field_generation` task.
- full suggestion regeneration 创建 `metadata_suggestion_generation` task.
- Review Inbox 保留局部 loading 镜像, 但状态来源应能追溯到 task id.
- Review field button 显示 `Generating`, `Retry waiting`, `Failed`, `Open task`.

Metadata field generation 完成后不得直接写 canonical metadata. 它只产生 field result. Review UI apply 时必须检查:

- selected suggestion still matches.
- field still matches.
- base revision still matches, 或用户明确选择 apply generated result.

如果用户已经切换 suggestion 或修改字段, UI 应显示 `Generated result available`, 不得静默覆盖当前 draft.

Accept / reject suggestion 不进入 task manager. 它们仍然是 metadata review service 的短事务写操作.

## Logs and Timeline

每个 attempt 一个 app-owned log file. Task Detail 是主要日志入口:

- structured timeline 解释调度, retry, cancel, output commit.
- live log tail 展示 running attempt 日志.
- raw log preview 用于完整排查.

Settings Logs 可以保留作为全局日志浏览器, 但不应替代 task detail.

日志读取必须通过 daemon / Tauri API, UI 不直接读任意文件路径. Log tail endpoint 必须限制为 app-owned attempt logs.

## Recovery

Daemon 启动时扫描 opened library 中非终态 tasks:

- `queued`: 保持 queued.
- `retry_waiting`: 若已到 retry time, 转回 queued, 否则保留等待.
- `running`: 如果没有活跃 worker ownership, 进入 `interrupted_retryable` 或 `interrupted_final`.
- 有明确 committed output links 的 running task 可以 reconcile 成 `completed`.

Image generation recovery:

- 如果 output 已 commit 到 library, 不能重复生成.
- 根据 generation event 和 output links 判断是否完成.
- 没有 committed output 且 provider attempt 可安全重试时, 进入 `interrupted_retryable`.

Metadata recovery:

- 如果 field result 已生成但未 apply, 作为 task output 保留.
- 如果 suggestion record 已创建, 通过 output link reconcile.
- 不得在恢复时覆盖用户后续编辑的 Review draft.

## Error Handling

错误分类:

- `transient`: timeout, rate limit, temporary network failure, worker crash before commit, interrupted before confirmed output.
- `retryable_manual`: credential missing, provider unavailable, output parse failure, log read failure.
- `final`: invalid params, unsupported capability, missing input version, schema validation failure requiring user input change.
- `cancel`: user requested cancel.
- `conflict`: cancel requested but output already committed, duplicate output detected, stale review draft conflict.

UI 呈现:

- Queue row 显示短错误和分类.
- Task Detail 显示 last error, attempt history, raw log, structured timeline.
- 自动 retry 只用于 `transient`.
- Manual retry 保留 task history.
- Duplicate 创建新 draft 或 queued task.

## Testing Strategy

Core / daemon tests:

- task state transition: queued -> running -> completed / retry_waiting / failed_final / canceled.
- queued-only reorder.
- scheduler slot selection with global concurrency, provider concurrency, priority, queue position.
- retry classification and backoff.
- crash recovery and output reconciliation.
- idempotent output commit.

API tests:

- local HTTP auth token required.
- daemon binds only loopback.
- batch enqueue creates stable task records and queue positions.
- task detail returns attempts, timeline, logs, output links.
- log tail rejects non app-owned paths.

Desktop state / UI tests:

- batch draft composer supports multi-line prompt without splitting.
- enqueue all sends separate task inputs.
- queued task reorder calls daemon reorder API and updates order.
- task list displays wait reason, attempts, next retry, status.
- task detail displays timeline, live log, outputs.
- Review Inbox field generation opens task detail and applies result only when selected suggestion / base revision still matches.
- completed image generation links to Gallery asset and Review suggestion.

## Rollout Plan

1. Add OpenSpec change for daemon task manager and Generate workflow redesign.
2. Introduce task persistence and core scheduler tests.
3. Add `imglab-daemon` with loopback HTTP, health, auth, task APIs.
4. Move current Tauri generation execution into daemon worker path.
5. Add metadata generation task types and Review Inbox handoff guards.
6. Redesign Generate workspace UI around composer / queue / detail.
7. Add live logs and timeline.
8. Add recovery and retry hardening.
9. Archive old in-memory `GenerationJobState` path after parity.

## Open Decisions

The design fixes these decisions:

- Immediate daemon is in scope.
- Loopback local HTTP is the transport.
- Batch composer uses explicit task draft cards, not newline splitting.
- Only queued tasks can be manually reordered.
- Retry is automatic only for transient errors.
- Image generation and Review metadata generation share the unified task manager.

Implementation planning still needs to decide:

- Exact runtime file path for daemon discovery.
- Exact SQLite migration shape and indexes.
- Whether task updates use polling first or SSE first.
- Exact base revision model for Review field result handoff.
