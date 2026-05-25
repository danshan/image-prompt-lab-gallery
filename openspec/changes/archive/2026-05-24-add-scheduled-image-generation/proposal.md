## Why

当前应用已经有长任务 daemon 和图片生成队列, 但用户仍需要手动提交每次生成. 定时图片生成可以把重复的 prompt workflow 自动化, 并允许 app 未启动时由 background daemon 按计划继续生成和归档图片.

这项能力现在需要落地, 因为 Prompt Workspace, Queue, Albums 和 Settings 已经具备足够的基础边界, 可以把 schedule 设计为一等自动化能力, 而不是把周期逻辑塞进一次性 task input.

## What Changes

- 新增一等 Scheduled Image Generation 能力:
  - 支持固定 prompt 定时生成.
  - 支持 dynamic prompt: 每次执行前先用 prompt expansion provider 基于 base prompt 和 dynamic prompt 生成最终 image prompt.
  - 支持每 N 分钟, 每 N 小时, 每天指定时间.
  - 支持 overlap skip 和 missed no-catch-up 策略.
  - 生成结果自动加入指定 manual album, 并打上用户指定 tags.
- 新增 prompt expansion provider 抽象, MVP 支持 `fake` 和 `codex-cli`.
- 新增 daemon schedule runner loop, 将 due schedule 转换为普通 image generation task, 并在 task 完成后执行 album/tag post-processing.
- 新增 background automation daemon lifecycle:
  - Settings 可开启或关闭 background daemon.
  - macOS 使用 LaunchAgent 支持登录或系统重启后自动启动.
  - Background daemon 只扫描显式开启 automation 的 libraries.
- 新增 desktop `Schedules` workflow, 用于管理 schedule jobs 和 run history.
- Settings 新增 Automation section, 用于 daemon 状态, launch-at-login 状态和 per-library automation opt-in.
- 不改变 managed library file layout, 不把 album 当成 filesystem folder.

## Capabilities

### New Capabilities

- `scheduled-image-generation`: 定义定时图片生成 job, run history, trigger policy, prompt expansion, task handoff, output album/tag post-processing 和 Schedules workflow 行为.

### Modified Capabilities

- `task-manager-daemon`: 增加 background daemon service management, launch-at-login, schedule runner loop, daemon discovery priority 和 automation-enabled library scanning.
- `image-generation`: 增加 prompt expansion provider contract, schedule-origin prompt snapshot, 以及 dynamic prompt 到 image generation task 的 handoff 语义.
- `albums-search`: 增加 scheduled output 自动加入 manual album 和应用用户指定 tags 的幂等语义.
- `desktop-workbench`: 增加独立 `Schedules` workflow 和 Settings Automation section.
- `resource-library`: 增加 schedule persistence, run persistence, run output persistence, schema migration 和 per-library automation opt-in.

## Impact

- Rust core:
  - 新增 scheduled generation domain, use cases, DTOs, repository ports 和 SQLite persistence.
  - 扩展 provider ports 支持 prompt expansion.
  - 扩展 asset/album metadata mutation path 以支持 schedule-owned post-processing.
- Daemon:
  - 新增 schedule routes, schedule runner loop, prompt expansion execution, run/task reconciliation, graceful background shutdown.
  - 调整 daemon startup/discovery 以支持 background daemon 和 automation-enabled libraries.
- Tauri backend:
  - 新增 schedule commands 和 automation daemon service management commands.
  - 新增 macOS LaunchAgent adapter.
- Desktop frontend:
  - 新增 `Schedules` workflow.
  - Settings 新增 Automation section.
  - Queue task detail 增加回跳到 schedule run 的上下文.
- Persistence:
  - SQLite schema version migration.
  - App/library registry 增加 automation opt-in.
- Verification:
  - 增加 core schedule policy tests, daemon runner tests, Tauri service management tests, frontend workflow tests, OpenSpec strict validation.
