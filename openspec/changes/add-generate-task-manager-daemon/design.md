## Context

当前 Generate 入口由 React composer, 前端 `queue` state 和 Tauri in-memory `GenerationJobState` 组成. 这可以支持单次生成和基础状态展示, 但无法支撑多任务管理, 持久化 job history, 并发限制, retry, cancel, structured timeline, live logs, 以及 Review Inbox 中 metadata generation 的统一可观察性.

本变更已在 `docs/superpowers/specs/2026-05-18-generate-task-manager-daemon-design.md` 中完成产品和架构设计. OpenSpec change 将该设计转为可实施 artifact. 本次选择 immediate daemon: 独立本地 worker / daemon 作为长任务执行事实来源, desktop 通过 loopback local HTTP 调度任务.

## Goals / Non-Goals

**Goals:**

- 新增本地 `imglab-daemon`, 通过 loopback local HTTP 提供 task API.
- 在 `imglab-core` 中定义 task model, scheduler policy, retry classification, task repository 和 recovery 语义.
- 支持 `image_generation`, `metadata_field_generation`, `metadata_suggestion_generation` 三类 task.
- 支持 batch enqueue, queued-only reorder, global / provider concurrency limits, transient auto retry, manual retry, duplicate, cancel.
- 让 Generate workspace 以 Batch Composer, Tasks Queue, Task Detail 的 Queue-Centric 结构展示多任务工作流.
- 让 Review Inbox 的 metadata generation 使用同一 task manager, 同时保持 review-first 写入 canonical metadata 的边界.
- 让 task detail 成为执行日志主入口, 展示 structured timeline, live log tail, raw log preview 和 output links.
- 支持 daemon restart 后对 queued, retry waiting, running 和 committed output tasks 做恢复或 reconcile.

**Non-Goals:**

- 不暴露远程 HTTP API, 只允许 loopback local API.
- 不引入 cloud sync, 多用户协作或远程 worker.
- 不把 accept / reject suggestion, update rating, add to album 这类短事务改成 task.
- 不在本次引入完整后台 service installer 或系统登录项管理.
- 不实现 provider 级高级 quota 管理或复杂 priority rules, 第一版只保留 priority + queue position.

## Decisions

### 独立 daemon 执行长任务

选择新增 `imglab-daemon`, 而不是继续由 Tauri command 直接 spawn generation thread.

原因:

- 长任务状态不应依赖 React 或 Tauri in-memory map.
- daemon 可以在 desktop 重启或未来 CLI 接入时复用同一套 task API.
- scheduler, retry, log tail 和 recovery 需要一个稳定执行 owner.

替代方案:

- Frontend-enhanced queue: 改动最小, 但状态丢失且调度语义散落在 UI.
- Desktop-hosted worker: 比 frontend queue 更稳, 但仍把长期任务执行绑在 desktop process lifecycle 上.

### Loopback local HTTP 作为 transport

第一版 daemon API 使用 versioned loopback local HTTP, 例如 `/v1/tasks`.

原因:

- desktop 和 CLI 都容易接入.
- 调试和测试比 Tauri-only IPC 更直接.
- API contract 与 Tauri 解耦.

约束:

- daemon MUST 只绑定 loopback address.
- client MUST 使用本地 session token.
- runtime file 记录 port, token file path, pid 和 API version.
- token MUST NOT 写入日志.

替代方案:

- Unix domain socket / named pipe: 本地安全边界更强, 但跨平台实现和调试成本更高.
- Tauri sidecar IPC only: desktop 集成简单, 但削弱未来 CLI 和独立 daemon 能力.

### Task persistence 落在 resource library

任务表写入当前 resource library, 而不是全局 app DB.

原因:

- task output 与 asset, version, generation event, metadata suggestion 强绑定.
- library 迁移, 备份和审计需要包含 task history.
- 多 library 任务不能混在无上下文全局队列里.

需要新增表:

- `tasks`
- `task_attempts`
- `task_events`
- `task_outputs`

### Unified task types

`image_generation`, `metadata_field_generation`, `metadata_suggestion_generation` 使用同一 task manager.

原因:

- 用户在 Generate, Queue 和 Review Inbox 中需要看到一致的 attempts, logs, retry 和 output links.
- Review field generation 也是 Codex CLI 长任务, 不应保持第二套隐藏异步系统.

短事务 accept / reject suggestion 不进入 task manager, 因为它们是同步 core write, 不需要 queue 或 retry backoff.

### Queued-only reorder

只有 `queued` task 可以人工排序.

原因:

- `running` 已占用 execution slot, 排序无意义.
- `retry_waiting` 受 `next_retry_at` 约束, 不应和普通 queued task 混用执行顺序.
- terminal tasks 只影响显示筛选, 不影响 scheduler.

### Transient-only auto retry

daemon 只对 transient errors 自动 retry. Non-transient errors 需要用户修复输入, credential 或环境后 manual retry / duplicate.

原因:

- 避免对 unsupported capability, invalid params, schema validation failure 等确定性失败浪费资源.
- 保留 task history, attempt logs 和 retry timeline, 便于用户判断下一步.

## Risks / Trade-offs

- [Risk] 本次范围从 UI redesign 扩展到 daemon architecture, 实施量显著增加. → Mitigation: tasks 分阶段落地, 先完成 core model 和 daemon API skeleton, 再迁移 generation, 最后接入 Review metadata generation.
- [Risk] Local HTTP token 或 runtime file 处理不当可能泄漏本地访问能力. → Mitigation: 只绑定 loopback, token 文件使用 app-owned runtime 目录, token 不进日志, API tests 覆盖 token required.
- [Risk] Daemon crash recovery 可能重复 commit output. → Mitigation: output commit 必须幂等, recovery 先根据 generation event / output links reconcile, 无 confirmed output 才 retry.
- [Risk] Review field generation 异步完成后覆盖用户后续编辑. → Mitigation: field output apply 必须检查 suggestion id, field 和 base revision, stale result 只显示 `Generated result available`.
- [Risk] Scheduler 与 SQLite 多进程访问引入 locking complexity. → Mitigation: daemon 作为唯一长任务 writer, desktop 短事务通过 API 或现有 core command 逐步收敛, repository tests 覆盖 concurrent claim.
- [Risk] UI 三栏 Generate workspace 信息密度高. → Mitigation: 保持 stable dimensions, compact row summaries, detail panel 按 section 展示, responsive 时折叠 composer 或 detail.

## Migration Plan

1. 新增 task schema 和 repository, 不改变现有 generation path.
2. 新增 daemon crate 和 health / auth / task list skeleton, desktop 能发现和启动 daemon.
3. 接入 task enqueue 和 scheduler, 先用 fake provider 验证 state transitions.
4. 将 image generation worker 从 Tauri in-memory job path 迁移到 daemon.
5. 接入 task output links 和 Gallery / Review refresh.
6. 将 metadata field / suggestion generation 迁移到 task manager.
7. 重构 Generate UI 到 Queue-Centric workspace.
8. 删除或归档旧 `GenerationJobState` path.

Rollback 策略:

- 在 image generation 迁移前, 旧 Tauri generation path 保持可用.
- daemon path 可先由 feature flag 或 internal command gate 控制.
- 如果 daemon API 不稳定, 可以保留旧 Generate button 作为 fallback, 直到 task manager parity 测试通过.

## Open Questions

- runtime file 的精确路径和 token file permission 策略.
- task updates 第一版使用 polling 还是直接实现 SSE.
- Review field result 的 `base_revision` 使用前端 form revision, suggestion updated_at, 还是独立 review draft revision.
- 是否在同一 change 中新增 CLI task commands, 或仅保证 API contract 为 CLI-ready.
