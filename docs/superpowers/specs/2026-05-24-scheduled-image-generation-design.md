# Scheduled Image Generation Design

## 目标

为 Image Prompt Lab 增加一等定时图片生成能力. 用户可以创建定时任务, 让 background daemon 在 app 未启动时继续按 schedule 生成图片. 每次生成结果进入指定 manual album, 并打上用户指定 tags.

本设计覆盖:

- 固定 prompt 定时生成.
- Dynamic prompt 定时生成: 每次执行前先用 LLM 基于 base prompt 和 dynamic prompt 生成最终 image prompt.
- 每 N 分钟, 每 N 小时, 每天指定时间.
- 独立 `Schedules` workflow.
- Settings 中的 background daemon 开关, launch-at-login 管理, per-library automation opt-in.
- OpenSpec 后续实现和验证边界.

## 已确认决策

- 指定文件夹语义: 使用现有 manual album, 生成成功后自动加入目标 album.
- Dynamic prompt 边界: 新增 prompt expansion provider 抽象, MVP 支持 `fake` 和 `codex-cli`.
- Overlap policy: 如果上一轮仍在运行, 新触发 skip, 记录 skipped run.
- Missed run policy: app 或 daemon 离线期间不补跑, 下次启动从当前时间计算下一次执行.
- UI 入口: 新增独立 `Schedules` workflow.
- Background daemon: Settings 开启后安装/启用 OS-level background agent, 支持系统重启或登录后自动启动.
- Daemon library scope: 只服务显式开启 `automation_enabled` 的 libraries.
- 推荐架构: schedule domain + daemon runner, 复用现有 image task pipeline.

## 非目标

- 不做通用 automation platform.
- 不支持远程 daemon 或多用户协作.
- 不改变 managed library file layout.
- 不把 album 语义改成 filesystem folder.
- 不让 schedule runner 绕过现有 task queue 直接生成图片.
- 不默认补跑所有 missed triggers.
- 不默认强杀正在运行的 provider process.

## 架构

新增 `scheduled generation` bounded context. Schedule 是生成意图和触发策略. Task 是一次具体执行. 两者通过 run/task link 关联, 不互相替代.

### `imglab-core`

Core 拥有业务语义:

- schedule definition.
- schedule run record.
- next-run calculation.
- overlap skip.
- missed no-catch-up.
- target album validation.
- explicit tags as confirmed metadata.
- run output idempotence contract.

Core 不拥有:

- OS launch agent install.
- Codex process execution.
- Tauri window or React state.
- loopback HTTP transport.

### `imglab-daemon`

Daemon 新增 schedule runner loop, 与现有 task scheduler loop 并行:

- 扫描 automation-enabled libraries.
- 发现 due schedules.
- 创建 schedule run.
- 执行 prompt expansion.
- 创建普通 `image_generation` task.
- 观察 linked task 状态.
- task completed 后执行 album/tag post-processing.
- 更新 run status 和 run outputs.

Daemon 仍只绑定 loopback, 所有 route 继续使用 local token auth.

### Prompt Expansion Providers

新增 prompt expansion provider capability, 与 image generation provider 分离.

MVP providers:

- `fake`: 用于测试和 deterministic local behavior.
- `codex-cli`: 调用 Codex CLI 生成最终 image prompt.

Prompt expansion provider 只返回 expanded prompt 和 provider metadata. 它不创建 image task, 不写 asset, 不写 generation event.

### Tauri Backend

Tauri 是 desktop client adapter 和 service manager:

- 调用 daemon schedule APIs.
- 映射 schedule/run/service views.
- 管理 background daemon enable/disable.
- 管理 macOS LaunchAgent install/uninstall/status.
- 提供 graceful stop/restart command.

Tauri 不执行 schedule runner, prompt expansion, image generation 或 post-processing.

### Desktop Frontend

新增 `Schedules` workflow:

- 展示 schedule list.
- 编辑 schedule definition.
- 展示 run history.
- 从 run 跳转到 linked Queue task.

Settings 新增 `Automation` section:

- background daemon state.
- launch-at-login state.
- automation-enabled libraries.
- Start, Stop, Restart, Repair launch agent actions.

Queue 继续展示具体 image tasks, 不承担 schedule management.

## 数据模型

### `scheduled_generation_jobs`

保存用户定义的周期生成任务.

关键字段:

- `id`
- `library_id`
- `name`
- `status`: `active`, `paused`, `disabled`
- `prompt_mode`: `fixed`, `dynamic`
- `fixed_prompt`
- `negative_prompt`
- `base_prompt`
- `dynamic_prompt`
- `prompt_expander_provider`
- `prompt_expander_model`
- `image_provider`
- `image_model`
- `parameters_json`
- `schedule_kind`: `interval_minutes`, `interval_hours`, `daily_time`
- `schedule_value_json`
- `timezone_id`
- `target_album_id`
- `tags_json`
- `overlap_policy`: initially `skip`
- `missed_run_policy`: initially `no_catch_up`
- `last_run_at`
- `next_run_at`
- `created_at`
- `updated_at`
- `paused_at`

Canonical storage can store intervals in minutes while UI shows minutes or hours. Daily schedule stores local `HH:mm` plus `timezone_id`.

### `scheduled_generation_runs`

保存每一次触发的审计记录.

关键字段:

- `id`
- `job_id`
- `library_id`
- `status`
- `scheduled_for`
- `started_at`
- `completed_at`
- `skip_reason`
- `error_code`
- `error_message`
- `expanded_prompt`
- `prompt_expansion_provider_metadata_json`
- `image_task_id`
- `output_asset_count`
- `tagged_asset_count`
- `album_added_asset_count`

Run status:

- `pending`
- `expanding_prompt`
- `task_queued`
- `task_running`
- `post_processing`
- `completed`
- `skipped`
- `failed`

### `scheduled_generation_run_outputs`

保存 run 和 output assets 的幂等 post-processing 状态.

关键字段:

- `run_id`
- `asset_id`
- `asset_version_id`
- `generation_event_id`
- `album_added`
- `tags_applied_json`
- `created_at`
- `updated_at`

### Registry Automation Opt-In

App registry 或 library registry 增加 per-library `automation_enabled`.

Background daemon 启动后只打开和扫描 `automation_enabled = true` 的 libraries. 如果 library path missing 或 schema mismatch, daemon 记录 service diagnostic, 不扫描该 library.

## 执行流

### Schedule Runner Loop

1. Daemon 周期扫描 automation-enabled libraries.
2. 查询 `next_run_at <= now` 且 active 的 schedules.
3. 对每个 due job 检查是否存在非终态 run 或 linked image task.
4. 如果存在, 写入 skipped run, `skip_reason = previous_run_active`, 并推进 `next_run_at`.
5. 如果 daemon 离线导致 `next_run_at < now`, 不补跑历史 ticks. 记录 `missed_no_catch_up`, 然后按当前时间计算下一次.
6. 创建 `scheduled_generation_runs`.
7. 进入 prompt resolve.

### Prompt Resolve

Fixed mode:

- 使用 `fixed_prompt` 作为 final image prompt.

Dynamic mode:

- 调用 prompt expansion provider.
- `base_prompt` 表达稳定主题和约束.
- `dynamic_prompt` 表达每次变化策略.
- 保存 `expanded_prompt` 和 provider metadata.

Expansion 失败:

- run 进入 `failed`.
- 不创建 image task.
- 推进 `next_run_at`.

### Image Task Handoff

1. Daemon 创建普通 `image_generation` task.
2. run 保存 `image_task_id`.
3. Queue 显示该 task.
4. Schedules run history 显示 linked task 状态.
5. task 执行继续走现有 retry, attempts, output links, logs 和 cancellation 机制.

### Post-Processing

Task completed 后:

1. Schedule runner 读取 task outputs.
2. 对每个 output asset 加入 `target_album_id`.
3. 应用 schedule tags.
4. 写入 `scheduled_generation_run_outputs`.
5. 更新 run counters.
6. run 进入 `completed`.

Post-processing 必须幂等. Daemon 重启后可继续处理未完成 output, 但不得创建重复 output row 或破坏 album/tag semantics.

## Background Daemon Lifecycle

### Enable

用户在 Settings 开启 background daemon 后:

- 安装/启用 macOS LaunchAgent.
- Daemon 使用 app registry path, 不使用 temp-only runtime registry.
- Daemon 写入 runtime file 和 token file.
- Daemon 启动后自动发现 automation-enabled libraries.

### Disable

用户在 Settings 关闭 background daemon 后:

- 禁用 LaunchAgent.
- 请求 daemon graceful shutdown.
- Daemon 停止领取新的 schedules 和 tasks.
- 当前 attempt 默认允许完成.

MVP 不强杀 provider process. 后续可增加 explicit stop-now action.

### Discovery Priority

Desktop 启动时:

1. 优先发现健康 background daemon.
2. 如果 background daemon disabled 或不可用, 对普通 Queue 操作可以沿用 app-owned sidecar fallback.
3. 如果 background daemon enabled 但 misconfigured, Settings Automation 显示 recoverable diagnostic 和 Repair action.

## UI 设计

### Schedules Workflow

主视图结构:

- Schedule list.
- Schedule detail/editor.
- Run history.

Schedule list 展示:

- name.
- status.
- prompt mode.
- next run.
- last run.
- target album.
- tags summary.

Editor 支持:

- prompt mode segmented control.
- fixed prompt textarea.
- dynamic base prompt textarea.
- dynamic prompt textarea.
- prompt expander provider/model.
- image provider/model.
- parameters JSON.
- schedule kind.
- interval value or daily `HH:mm`.
- album selector.
- tag chips.
- enable/disable.
- run now.
- duplicate.
- delete.

Run history 展示:

- status.
- scheduled time.
- started/completed time.
- expanded prompt snapshot.
- linked image task.
- output assets.
- skip/error reason.

### Settings Automation Section

展示:

- Background daemon: enabled/disabled, running/offline, pid, last heartbeat.
- Launch at login: enabled/disabled, installed/missing/misconfigured.
- Automation libraries: registered libraries with `automation_enabled` toggles.
- Actions: Start, Stop, Restart, Repair launch agent.

## API Surface

Daemon schedule routes:

```text
GET /v1/schedules?library_id=...
POST /v1/schedules
PUT /v1/schedules/{id}
POST /v1/schedules/{id}/enable
POST /v1/schedules/{id}/disable
POST /v1/schedules/{id}/run-now
GET /v1/schedules/{id}/runs
GET /v1/schedule-runs/{id}
```

Tauri commands:

```text
get_automation_daemon_status
set_automation_daemon_enabled
set_library_automation_enabled
restart_automation_daemon
repair_automation_launch_agent
list_scheduled_generation_jobs
create_scheduled_generation_job
update_scheduled_generation_job
enable_scheduled_generation_job
disable_scheduled_generation_job
run_scheduled_generation_job_now
list_scheduled_generation_runs
get_scheduled_generation_run
```

## Error Handling

- Prompt expansion failed: run = `failed`, no image task, error saved.
- Image task failed final: run = `failed`, linked task preserved.
- Image task retry waiting/running: run remains linked and reflects task state.
- Post-processing partial failure: run = `failed`, output rows preserve processed state; runner can retry incomplete post-processing idempotently.
- Target album deleted: job is paused and run fails with recoverable error.
- Tags empty: allowed.
- Provider unavailable: run fails with recoverable provider error.
- Library missing or schema mismatch: daemon records service diagnostic and skips that library.

## Time Semantics

- MVP uses local timezone per job, stored as `timezone_id`.
- Default timezone is current system timezone at creation time.
- `daily_time` uses `HH:mm`.
- DST invalid local time: skip to next valid local occurrence and record policy in run diagnostic.
- DST ambiguous local time: use first occurrence.

## OpenSpec Changes

Create a single change, likely `add-scheduled-image-generation`.

Affected specs:

- `task-manager-daemon`: background daemon service, schedule runner, launch-at-login, schedule/task handoff.
- `image-generation`: prompt expansion provider, generated prompt snapshot, schedule-origin generation event context.
- `albums-search`: scheduled output album membership and tag application semantics.
- `desktop-workbench`: Schedules workflow and Settings Automation section.
- `resource-library`: schema migration, library automation opt-in, backup/restore compatibility if needed.

## Verification Plan

Core tests:

- Schedule CRUD validation.
- Interval minutes/hours next-run calculation.
- Daily `HH:mm` timezone next-run calculation.
- Missed no-catch-up.
- Overlap skip.
- Target album validation.
- Run output idempotence.

Daemon tests:

- Due schedule creates one run and one image task.
- Dynamic prompt expansion creates expanded prompt snapshot.
- Expansion failure does not create image task.
- Completed image task applies album and tags.
- Daemon restart resumes post-processing without duplicate output rows.
- Only automation-enabled libraries are scanned.
- Background daemon discovery prefers service daemon over sidecar.

Tauri tests:

- Service status mapping.
- LaunchAgent install/uninstall/status adapter behavior.
- Library automation opt-in command mapping.
- Recoverable diagnostics for missing launch agent or offline daemon.

Frontend tests:

- Schedules workflow renders list/detail/run history.
- Fixed and dynamic prompt modes validate required fields.
- Schedule kind controls validate interval and daily time.
- Settings Automation toggles daemon and per-library automation.
- Run history links to Queue task detail.

Validation commands:

```bash
cargo fmt --all --check
cargo test -p imglab-core
cargo test -p imglab-daemon
cargo test -p imglab-desktop
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
openspec validate add-scheduled-image-generation --strict
openspec validate --specs --strict
git diff --check
```

## Implementation Order

1. Create OpenSpec change and delta specs.
2. Add core schedule domain, DTOs, repository ports and SQLite migrations.
3. Add prompt expansion provider port and fake/codex-cli implementations.
4. Add daemon schedule routes and schedule runner loop.
5. Add post-processing handoff to album/tag owners.
6. Add Tauri service management and schedule commands.
7. Add Schedules workflow.
8. Add Settings Automation section.
9. Run full validation and archive OpenSpec change after implementation is proven.
