## Context

当前桌面应用已经实现本地 resource library, Gallery/Search, Albums, Review Inbox, Task Queue, Settings Libraries/Logs 和 Inspector. 这些能力的业务语义大多已经存在, 但前端结构仍集中在 `apps/desktop/src/main.tsx` 和 `apps/desktop/src/styles.css`, UI 也更像功能堆叠后的管理台.

用户已经确认新的高保真 demo 方向, 并选择 Big-bang 产品工作流重构. 因此本 change 允许修改 UI shell, workflow IA, read model boundary 和必要的 core/Tauri API, 但仍保持 local-first, single-user, Rust core 作为写操作事实来源, review-first metadata 和 daemon task manager 的既有架构原则.

## Goals / Non-Goals

**Goals:**

- 将桌面端重构为 Studio Console 产品模型, 统一 Gallery, Albums, Review, Queue, Settings 和 Inspector 的信息架构.
- 建立新的视觉系统, 让图片资产, 生成任务, metadata review 和 version lineage 成为第一层产品信号.
- 新增或调整屏幕级 read models, 让 React 使用稳定 product semantics, 不从多个低级 command 拼装跨 workflow 状态.
- 明确 pending suggestion, local review draft, generated result 和 canonical metadata 的边界.
- 拆分前端组件和状态边界, 避免 `main.tsx` 持续承载全部 workflow rendering 和 IPC orchestration.
- 通过多视口视觉 QA 和行为测试覆盖 Big-bang 改动风险.

**Non-Goals:**

- 不引入 cloud sync, 多用户协作, resource library encryption 或 advanced backup/migration.
- 不实现 daemon, IPC 或 local HTTP API 的整体重写.
- 不实现 Photoshop-style image editing 或高级 graph lineage visualization.
- 不实现 stable native OpenAI / Grok image clients.
- 不改变 Codex CLI provider 的根本执行策略.
- 不把视觉 demo HTML 直接作为 production code 使用.

## Decisions

### 1. Studio Console shell

桌面端顶层 shell 采用 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` 的产品模型.

- `Studio Rail` 承载顶层 workflow navigation 和稳定全局入口.
- `Library Context` 展示当前 resource library, storage/review/task/provider summary, collections 和 active jobs.
- `Workspace` 承载当前 workflow 的主要工作区.
- `Inspector` 承载 selected asset 或 selected workflow entity 的上下文详情和命令.
- `Activity Strip` 展示 active generation/review/task activity, 并提供到 Queue 的入口.

这不是简单重命名旧三栏. 旧 Sidebar 同时承担 library selector, nav 和 status card; 新 shell 将 navigation 和 library context 分离, 让 workspace 的主任务更稳定.

### 2. 视觉系统采用 Studio Console palette

新的视觉系统使用 graphite chrome, warm ivory canvas, cobalt secondary action, vermilion primary generation action, lime/amber/red/green status colors. 目标是比旧 neutral-teal workbench 更有产品辨识度, 但仍保持桌面生产力工具的信息密度.

实现上必须 token 化颜色, spacing, border radius, control sizes, status colors 和 focus styles. 图标使用一致的 SVG affordance, 后续可以迁移到 `lucide-react`, 但本 change 不强制引入依赖.

### 3. 屏幕级 read model boundary

React 不应继续从多个低级 command 临时拼装跨 workflow 语义. Rust core/service boundary 应提供稳定 read models:

- `StudioOverview`: 当前 library summary, review pending count, active/running/failed task counts, provider health, storage/integrity summary.
- `AssetBoardItem`: image path, title, current version, version count, review state, provider/model, task origin, album context, rating, tags.
- `AssetInspectorDetail`: canonical metadata, pending suggestion summary, file integrity, album memberships, version lineage, generation/task origin.
- `ReviewDraftDetail`: suggestion, local draft seed, confidence, history, generated field results, related tasks, asset context.
- `TaskDetail`: task, attempts, timeline, log tail, outputs, asset links, version links, review suggestion links.
- `DiagnosticsOverview`: provider health, app-owned log summaries, daemon status, library lifecycle status.

这些 read models 应由 Rust core 或 Tauri service boundary 组合, 不在 React 内部复制 SQL/read path 语义.

### 4. Frontend component boundary

`main.tsx` 收敛为 bootstrap, top-level app orchestration 和 minimal route/workflow state. 目标组件边界:

- Shell: `AppShell`, `StudioRail`, `LibraryContextPanel`, `WorkspaceFrame`, `InspectorFrame`, `ActivityStrip`.
- Workflows: `GalleryWorkspace`, `AlbumsWorkspace`, `ReviewWorkspace`, `QueueWorkspace`, `SettingsWorkspace`.
- Inspector sections: prompt/file/review/albums/lineage/task-origin/variation sections.
- Hooks/state: `useLibraryRegistry`, `useStudioOverview`, `useGalleryQuery`, `useReviewDrafts`, `useTaskQueue`, `useDiagnostics`, 以及可测试的 pure state modules.

拆分应跟随现有代码能力边界, 不为了目录美观引入空抽象. 任何新增 abstraction 必须减少真实复杂度或复用 workflow pattern.

### 5. Review draft invariant

Review workflow 拥有本地 editable draft. Pending suggestion, generated field result, suggestion history row 和 canonical asset metadata 必须在 UI 和 read model 中明确区分.

Invariant: AI metadata suggestion 在用户接受前不得写入 canonical asset metadata. Field generation task completed 后只能产生 generated result 或 draft patch. 如果结果 stale, UI 必须保留用户当前编辑, 展示 generated result available, 并允许显式 apply 或 ignore.

### 6. Cross-workflow transitions

Studio Console 必须显式支持这些跳转:

- Task completed -> open asset -> open output version -> open review suggestion.
- Task Detail -> Open asset, Open review suggestion, Open output version, Open attempt log.
- Review suggestion -> Open source task, Open asset, Open history row.
- Inspector -> Generate variation, Add to album, Open pending review, Open source task.
- Settings diagnostics/logs -> Open related task detail when relation exists.

这些关系需要稳定 link payload, 不应依赖 UI 按 id 约定猜测目标类型.

### 7. Workflow behavior

Gallery 是 image-first asset board, 不再是 metadata-heavy card grid. Albums 是 collection management workspace. Review 是 staged metadata workbench. Queue 是 operations console. Settings 是 library/provider/log diagnostics. Inspector 是 selected asset command surface.

每个 workflow 必须覆盖 normal, loading, empty, error 和 recovery states. 每个可变长内容, 包括 path, task id, prompt, schema prompt, checksum 和 log tail, 必须有 truncation, wrapping 或 bounded internal scroll 策略.

### 8. Responsive and visual QA

`960px` 仍是 first-class compact desktop minimum. 新 shell 在宽屏展示完整 rail/context/workspace/inspector, compact desktop 下 Workspace 优先, Library Context 和 Inspector 可以折叠或变成 drawer/rail. Activity Strip 必须保持当前 activity 可达.

视觉 QA 至少覆盖 `1440px` 和 `960px`, 并检查 Gallery, Albums, Review, Queue, Settings 和 Inspector 没有关键操作不可达, 文本重叠, 控件跳动或对比度不足.

## Risks / Trade-offs

- [Risk] Big-bang scope 太大. → Mitigation: tasks 按 spec, read models, shell, workflow, QA gates 排序, 每个 gate 都可单独 review.
- [Risk] UI 混淆 pending suggestion 和 canonical metadata. → Mitigation: `metadata-review` spec 拥有 invariant, read models 显式标注 canonical/staged/generated.
- [Risk] read models 复制业务逻辑. → Mitigation: read models 在 Rust core/service boundary 组合, React 只消费产品语义 payload.
- [Risk] 新视觉系统牺牲信息密度. → Mitigation: compact controls, stable dimensions, image-first but metadata-aware tiles, 多视口视觉 QA.
- [Risk] `main.tsx` 拆分过度. → Mitigation: 以 workflow 和 state ownership 为边界, 不创建无行为的空 wrapper.
- [Risk] Settings diagnostics 与 Task Detail 职责重叠. → Mitigation: Task Detail 是 task debugging 主入口, Settings Logs 是全局浏览和 deep-link 来源.

## Migration Plan

1. 写入并验证 OpenSpec delta, 明确 product semantics 和验收标准.
2. 新增 Studio Console read models 和 Tauri command payload, 先以现有底层查询组合实现.
3. 拆分前端 shell, workflow components 和 hooks, 保持现有业务能力可运行.
4. 引入 design tokens 和新 shell visual system.
5. 逐个替换 Gallery, Albums, Review, Queue, Settings 和 Inspector workflow UI.
6. 接入 cross-workflow deep links 和 activity strip.
7. 补齐 loading, empty, error, recovery states.
8. 运行 Rust tests, frontend state tests, TypeScript build 和多视口视觉 QA.

回滚策略是回退本 change 的前端组件, CSS tokens 和新增 read model commands. 如新增 read model 只是组合现有持久化数据, 不需要数据迁移回滚.

## Open Questions

无阻塞问题. 实施中如果发现某个 read model 需要 SQLite schema 变更, 必须先更新本 change 的 design/spec/tasks, 再进入 schema migration 实现.
