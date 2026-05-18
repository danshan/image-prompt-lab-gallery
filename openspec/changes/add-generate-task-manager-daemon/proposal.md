## Why

当前 Generate 只是一条轻量前端入口加 Tauri 内存队列, 无法支撑真实多任务生成工作流. 用户需要在 Generate, Tasks Queue 和 Review Inbox 之间形成完整闭环: 批量提交, 人工排序, 并发控制, 自动重试, 执行日志, 任务详情, 以及生成结果到 Review 的可追溯跳转.

本变更将 Generate 从单次操作升级为本地任务系统. 任务执行由独立 daemon 负责, desktop 通过 loopback local HTTP 调度和观察任务, 从而支持后台执行, 任务恢复和未来 CLI 复用.

## What Changes

- 新增独立 `imglab-daemon` worker, 负责 task scheduler, provider execution, retry, concurrency slots, structured timeline, log streaming 和 interrupted task recovery.
- 新增 core task model 和 task persistence, 覆盖 `image_generation`, `metadata_field_generation`, `metadata_suggestion_generation`.
- 新增 loopback local HTTP API, 包含 daemon health, capabilities, task enqueue, batch enqueue, reorder, cancel, retry, duplicate, detail, events 和 log tail.
- 重构 desktop Generate workspace 为 Queue-Centric 三栏工作流: Batch Composer, Tasks Queue, Task Detail.
- 支持 explicit task draft cards 和 batch enqueue, 不用 newline 或 blank line 拆分多行 prompt.
- 支持仅对 `queued` tasks 人工排序.
- 支持 global 和 provider concurrency limits, 默认 `global = 2`, `codex-cli = 1`, `fake = 4`.
- 支持 transient error 自动 retry, non-transient error manual retry, 并展示 attempts, next retry time 和 wait reason.
- 将 Review Inbox 的 metadata field generation 和 full suggestion regeneration 纳入统一 task manager, 同时保持 review-first 语义.
- 将 task detail 作为执行日志主入口, 展示 structured timeline, live log tail, raw log preview 和 output links.
- 移除或归档当前 Tauri in-memory `GenerationJobState` 长任务执行路径.
- 不引入 cloud sync, 多用户协作, 远程 daemon 或局域网 HTTP API.

## Capabilities

### New Capabilities

- `task-manager-daemon`: 本地 daemon, task model, scheduler, retry, persistence, local HTTP API, logs 和 recovery.

### Modified Capabilities

- `desktop-workbench`: Generate workspace, Generation Queue 和 Review Inbox 联动变为 Queue-Centric task workflow.
- `image-generation`: image generation 通过 task manager 执行, task completion 绑定 asset/version/generation event/review suggestion output links.
- `metadata-review`: metadata field generation 和 full suggestion regeneration 通过 task manager 执行, 完成后仍只影响 review draft 或 pending suggestion, 不直接写 canonical metadata.
- `app-logs`: task detail 成为任务日志主入口, log tail 和 raw log preview 必须限制为 app-owned task attempt logs.

## Impact

- Rust workspace: 新增 `imglab-daemon` crate, 扩展 `imglab-core` task schema, scheduler service, retry classification 和 task repository.
- SQLite resource library: 新增 `tasks`, `task_attempts`, `task_events`, `task_outputs` 等持久化结构和迁移.
- Tauri desktop: 增加 daemon lifecycle 管理, local HTTP client, token handling, error mapping, 移除 long-running generation 直接执行职责.
- React desktop UI: 重构 Generate workspace, task draft composer, task queue, task detail, Review Inbox task state mirror 和 cross-links.
- Provider adapters: image generation 和 metadata generation 被 daemon worker 调用, 并向 task attempt logs 写入 stdout/stderr 和 provider result.
- Tests: 增加 core scheduler/state transition/recovery tests, daemon API/auth/log tests, desktop state/UI tests, Review handoff tests.
- Docs/specs: 新增 task manager daemon spec, 更新 desktop-workbench, image-generation, metadata-review 和 app-logs specs.
