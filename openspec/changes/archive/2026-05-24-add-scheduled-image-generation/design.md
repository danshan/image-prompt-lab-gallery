## Context

当前应用已经具备 local-first resource library, SQLite persistence, manual albums/tags, prompt workspace, image generation provider boundary, 以及独立 local daemon task manager. 现有 daemon 主要由 desktop 启动为 app-owned sidecar, 负责一次性 image generation 和 metadata generation task.

本次变更把定时图片生成作为一等能力加入系统. 用户需要在 app 未启动时继续自动生成图片, 因此不能把 schedule runner 放在 React/Tauri foreground process 中. 同时, 生成结果必须进入现有 manual album 并应用用户指定 tags, 不能改变 managed file layout 或把文件夹语义误建模为 filesystem path.

## Goals / Non-Goals

**Goals:**

- 支持 fixed prompt 和 dynamic prompt 两种定时生成模式.
- 支持每 N 分钟, 每 N 小时, 每天指定时间.
- 将 schedule 建模为独立 domain, 将每次执行建模为 run, 将实际图片生成复用现有 image generation task.
- 新增 prompt expansion provider 抽象, MVP 支持 `fake` 和 `codex-cli`.
- 支持 background daemon 和 macOS LaunchAgent, app 未启动时仍可执行 due schedules.
- 支持 per-library automation opt-in, background daemon 只扫描显式开启 automation 的 libraries.
- 提供独立 `Schedules` workflow 和 Settings Automation section.
- 保持 task retry, logs, output links, generation event 和 asset persistence 的现有 ownership.

**Non-Goals:**

- 不做通用 automation platform.
- 不支持远程 daemon, 多用户协作或 cloud sync.
- 不把目标 album 改成 filesystem folder.
- 不绕过现有 task queue 直接写 image generation output.
- 不默认补跑 missed runs.
- 不默认强杀正在运行的 provider process.
- 不把 prompt expansion provider 扩展到稳定 native OpenAI/Grok client.

## Decisions

### 1. Schedule domain 与 task domain 分离

Schedule 表达周期生成意图和触发策略. Task 表达一次具体长任务执行. Schedule runner 只负责把 due schedule 转换为普通 image generation task, 并在 task 完成后做 album/tag post-processing.

替代方案是把 schedule config 放入 task input. 该方案改动小, 但会让 run history, missed policy, overlap policy 和 post-processing 幂等语义混在一次性 task 中, 不利于长期维护.

### 2. Prompt expansion provider 独立于 image generation provider

Dynamic prompt 先调用 prompt expansion provider 生成最终 image prompt, 再把 expanded prompt 快照传给 image generation task. Prompt expansion provider 只返回文本和 metadata, 不创建 asset, task 或 generation event.

替代方案是复用 image generation provider 或直接在 daemon scheduler 内调用 Codex. 这会混淆 provider capability, 并让后续接入其他 LLM provider 时扩展困难.

### 3. Background daemon 使用 per-library automation opt-in

Background daemon 启动后从 app registry 找到 automation-enabled libraries, 并只扫描这些 libraries. 这样 app 关闭时仍有明确的 automation scope.

替代方案是扫描所有 registered libraries 或只扫描 active library. 前者太隐式, 后者在 app 未启动时语义不稳定.

### 4. Missed run 不补跑, overlap 默认 skip

Daemon 离线或 app 未启动导致错过 schedule 时, 系统记录 missed/no-catch-up 诊断并从当前时间计算下一次. 如果上一轮还未终态, 新触发记录 skipped run.

替代方案是补跑最近一次或全部 missed runs. 对本地图片生成来说, provider 成本和队列积压不可预测, MVP 不采用.

### 5. macOS-first LaunchAgent service manager

Settings Automation 管理 macOS LaunchAgent install/uninstall/status. Tauri 后端通过平台 adapter 封装 service management. 未来 Windows Task Scheduler 或 systemd user service 可作为新 adapter 加入.

替代方案是只保持手动启动 daemon. 用户明确要求 app 不启动也能动态生成图片, 因此需要 OS-level background agent.

### 6. Album/tag post-processing 作为 run output 幂等阶段

Schedule runner 在 linked image task completed 后读取 task outputs, 将 output assets 加入目标 manual album, 应用用户指定 tags, 并记录 run outputs. Album add 和 tag attach 复用现有 no-op/UPSERT 语义.

替代方案是在 image generation use case 内直接写 album/tags. 这会把 schedule-specific output routing 泄漏到 generation core path.

## Risks / Trade-offs

- Background daemon 与 existing sidecar discovery 可能冲突 -> Desktop discovery 必须优先复用健康 background daemon, 只有 background daemon disabled 或不可用时才 fallback sidecar.
- LaunchAgent install/upgrade/uninstall 可能受签名和路径变化影响 -> Service manager 保存可诊断状态, Settings 提供 Repair action, release 文档需明确 macOS-first behavior.
- Dynamic prompt provider failure 会让 run 没有 image task -> Run 记录必须保存 error, 并且推进 next run, 避免 schedule 卡死.
- Post-processing 可能在 daemon crash 后只完成一部分 -> `scheduled_generation_run_outputs` 记录每个 output 的处理状态, runner 可幂等恢复.
- Target album 被删除会让 schedule 无法归档输出 -> 系统将 job pause, run failed, 并要求用户选择新的 manual album.
- Timezone 和 DST 复杂 -> MVP 保存 `timezone_id`, daily invalid local time 跳到下一次 valid occurrence, ambiguous local time 取第一次 occurrence 并记录诊断.

## Migration Plan

1. 增加 SQLite schema version, 创建 schedule jobs, schedule runs 和 run outputs tables.
2. 扩展 registry schema 或 registry metadata, 增加 per-library `automation_enabled`.
3. 旧 libraries migration 后默认没有 schedules, `automation_enabled = false`.
4. Backup/restore 必须保留 schedule tables 和 automation opt-in 元数据.
5. Rollback 时旧应用会遇到 schema version mismatch, 需要使用现有 schema mismatch 保护而不是降级打开新 library.

## Open Questions

- macOS LaunchAgent plist 的最终 label 和 runtime directory 需要在 implementation 中固定, 并与 release packaging 保持一致.
- `codex-cli` prompt expansion 的 exact prompt template 需要在 provider implementation 中以测试固定, 但不影响 OpenSpec 行为契约.
- 是否在 Schedules list 中显示 generated output thumbnails 可以在实现中按 UI 容量决定, 但 run history 必须能跳转到 output assets 和 linked task.
