## 1. Core Task Model and Persistence

- [x] 1.1 定义 task, attempt, event, output link DTO 和 domain enums, 覆盖 task type, status, error classification 和 output link type.
- [x] 1.2 扩展 resource library schema, 新增 `tasks`, `task_attempts`, `task_events`, `task_outputs` 表和必要索引.
- [x] 1.3 实现 task repository, 支持 batch create, get detail, list by library, update status, append event, append attempt, append output.
- [x] 1.4 实现 queued-only reorder repository 方法, 拒绝或忽略非 queued task.
- [x] 1.5 实现 output commit 幂等辅助方法, 用于根据 output links reconcile completed task.
- [x] 1.6 为 schema migration, repository CRUD, output link 和 queued-only reorder 添加 Rust 测试.

## 2. Scheduler and Retry Policy

- [x] 2.1 实现 scheduler policy, 按 priority, queue position, created time 选择可执行 queued tasks.
- [x] 2.2 实现 global concurrency 和 provider concurrency slot 计算, 默认 `global = 2`, `codex-cli = 1`, `fake = 4`.
- [x] 2.3 实现 wait reason 计算, 覆盖 global slot, provider slot 和 retry backoff.
- [x] 2.4 实现 retry classification 和 backoff policy, transient errors 自动进入 retry waiting.
- [x] 2.5 实现 manual retry 和 duplicate task 语义, retry 保留 task history, duplicate 使用新 task id.
- [x] 2.6 为 scheduler slot selection, wait reason, retry backoff, manual retry 和 duplicate 添加 Rust 测试.

## 3. Local Daemon Foundation

- [x] 3.1 新增 `crates/imglab-daemon` crate, 接入 workspace, 定义 daemon entrypoint 和 configuration.
- [x] 3.2 实现 loopback HTTP server skeleton, 提供 `/v1/health` 和 `/v1/capabilities`.
- [x] 3.3 实现 runtime file 写入和读取格式, 包含 port, token file path, daemon pid 和 API version.
- [x] 3.4 实现 local session token 校验, 确保未授权请求不能读取 task 或日志.
- [x] 3.5 确保 daemon 只绑定 loopback address.
- [x] 3.6 为 health, auth token required, loopback binding 和 runtime file 添加测试.

## 4. Daemon Task API

- [x] 4.1 实现 `POST /v1/libraries/open`, 使 daemon 可打开并管理 resource library task context.
- [x] 4.2 实现 `POST /v1/tasks` 和 `POST /v1/tasks/batch`, 创建持久化 tasks 和 submitted events.
- [x] 4.3 实现 `GET /v1/tasks` 和 `GET /v1/tasks/{task_id}`, 返回 queue row 和 task detail view.
- [x] 4.4 实现 `POST /v1/tasks/reorder`, 只接受 queued tasks 的新顺序.
- [x] 4.5 实现 `POST /v1/tasks/{task_id}/cancel`, 覆盖 queued, retry waiting 和 running task.
- [x] 4.6 实现 `POST /v1/tasks/{task_id}/retry` 和 `POST /v1/tasks/{task_id}/duplicate`.
- [x] 4.7 实现 `GET /v1/tasks/{task_id}/events` 和 `GET /v1/tasks/{task_id}/logs/tail`.
- [x] 4.8 为 task API status code, error mapping, auth, reorder 和 log path restrictions 添加集成测试.

## 5. Daemon Worker Execution

- [x] 5.1 实现 daemon scheduler loop, 周期性 claim 可执行 task 并创建 attempt.
- [x] 5.2 接入 fake provider execution, 先验证 completed, failed, retry waiting 和 canceled 状态流.
- [x] 5.3 将 Codex CLI image generation adapter 接入 daemon worker, attempt log 写入 app-owned log file.
- [x] 5.4 将 image generation output commit 到 core library, 写入 generation event, asset version 和 task output links.
- [x] 5.5 实现 running task cancel 的 best-effort 子进程终止和 completed-after-cancel-requested 记录.
- [x] 5.6 为 worker state transition, output commit, cancel 和 provider failure 添加 Rust 测试.

## 6. Daemon Recovery

- [x] 6.1 实现 daemon startup recovery, 扫描 opened libraries 的 queued, retry waiting 和 running tasks.
- [x] 6.2 对 retry waiting tasks 根据 next retry time 保持等待或转回 queued.
- [x] 6.3 对 running tasks 根据 output links reconcile completed 或标记 interrupted retryable / interrupted final.
- [x] 6.4 确保 recovery 不重复调用 provider 或重复创建 output version / suggestion.
- [x] 6.5 为 interrupted task recovery 和 committed output reconcile 添加 Rust 测试.

## 7. Desktop Tauri Daemon Client

- [x] 7.1 在 `apps/desktop/src-tauri` 增加 daemon lifecycle manager, 支持发现健康 daemon 或启动 sidecar.
- [x] 7.2 增加 daemon local HTTP client 和 token handling, 将 daemon errors 映射到现有 `{ code, message, recoverable }` 形态.
- [x] 7.3 将现有 `start_generation` / `get_generation_job` 路径迁移到 daemon task API 或新增等价 task commands.
- [x] 7.4 保留旧 generation path 到 daemon parity 通过, 然后删除或归档 Tauri in-memory `GenerationJobState`.
- [x] 7.5 为 daemon client discovery, health failure fallback 和 error mapping 添加测试.

## 8. Generate Workspace UI

- [x] 8.1 重构 Generate workspace 为 Batch Composer, Tasks Queue, Task Detail 三栏布局.
- [x] 8.2 实现 task draft card state, 支持多行 prompt, provider, operation, params, source version snapshot.
- [x] 8.3 实现 Add task, duplicate draft, remove draft, structured JSON import 和 Enqueue all.
- [x] 8.4 实现 task list polling 或 subscription, 展示 status, wait reason, attempts, next retry time 和 quick actions.
- [x] 8.5 实现 queued-only drag / move up / move down, 调用 daemon reorder API.
- [x] 8.6 实现 task detail, 展示 input snapshot, attempts, structured timeline, live log tail, raw log preview, output links 和 errors.
- [x] 8.7 实现 responsive behavior, 在窄窗口中折叠 composer 或 detail, 保持 task queue 可用.
- [x] 8.8 为 multi-line prompt 不拆分, batch enqueue, queued-only reorder 和 task detail state 添加 frontend tests.

## 9. Review Inbox Task Integration

- [x] 9.1 将 Review field regeneration 改为创建 `metadata_field_generation` task, input snapshot 包含 suggestion id, field 和 base revision.
- [x] 9.2 将 full suggestion regeneration 改为创建 `metadata_suggestion_generation` task, 完成后创建新 pending suggestion record.
- [x] 9.3 在 Review Inbox 中展示 field generation task mirror 状态, 包含 generating, retry waiting, failed 和 Open task detail.
- [x] 9.4 实现 field generation result handoff, 仅当 suggestion id, field 和 base revision 匹配时自动 apply 到本地 draft.
- [x] 9.5 对 stale field result 显示 generated result available, 不覆盖用户编辑.
- [x] 9.6 为 Review field task, stale result guard, schema prompt validation 和 full suggestion output link 添加测试.

## 10. Logs and Settings Integration

- [x] 10.1 将 task attempt logs 纳入 app-owned log classification.
- [x] 10.2 在 Task Detail 中展示 live log tail 和 raw log preview.
- [x] 10.3 保留 Settings Logs 全局浏览入口, 并能展示 task attempt logs.
- [x] 10.4 确保 log APIs 拒绝非 app-owned task attempt path.
- [x] 10.5 为 log tail, raw preview, Settings Logs 列表和 path restriction 添加测试.

## 11. End-to-End Verification

- [x] 11.1 运行 core 和 daemon Rust tests, 覆盖 task schema, scheduler, API, worker 和 recovery.
- [x] 11.2 运行 desktop Tauri tests 或等价 command/client tests.
- [x] 11.3 运行 frontend tests, 覆盖 composer, queue, detail 和 Review task mirror.
- [x] 11.4 启动 desktop dev flow, 验证 fake provider batch enqueue, reorder, retry, cancel, completed output links.
- [ ] 11.5 使用 Codex CLI provider 手动验证 single-flight execution, attempt logs, output import 和 Review Inbox suggestion link.
- [x] 11.6 验证 daemon restart recovery: queued 保留, retry waiting 恢复, running interrupted reconcile, committed output 不重复.

## 12. Cleanup and Documentation

- [x] 12.1 更新 `docs/development.md`, 说明 daemon 启动, runtime file, local HTTP API 和测试命令.
- [x] 12.2 更新 `docs/providers.md`, 说明 provider execution 由 daemon worker 执行以及 Codex CLI 并发限制.
- [x] 12.3 移除旧 in-memory queue UI 和 Tauri `GenerationJobState` dead code.
- [x] 12.4 运行 OpenSpec verify 或 status, 确认 change artifacts apply-ready.
