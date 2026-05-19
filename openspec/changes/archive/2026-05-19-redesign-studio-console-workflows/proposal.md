## Why

当前桌面 Workbench 已经具备 Gallery, Albums, Review, Queue, Settings 和 Inspector 的 MVP 能力, 但产品模型仍然分散: Gallery 侧重列表浏览, Review 侧重表单编辑, Queue 侧重任务状态, Inspector 侧重资产详情, 它们之间的 task, asset version, pending suggestion 和 canonical metadata 关系没有形成一个清晰的 Studio Console 工作流.

本变更将桌面端重新定义为本地优先的 AI image asset studio console: 图片资产是主工作对象, 生成任务, metadata review, version lineage, album curation 和 diagnostics 通过稳定 read model 与跨 workflow 跳转连接起来.

## What Changes

- 将桌面端 shell 从旧的 `Library Sidebar | Workspace | Inspector` 表达升级为 Studio Console: `Studio Rail | Library Context | Workspace | Inspector | Activity Strip`.
- 引入新的视觉系统和设计 tokens: graphite chrome, warm ivory canvas, cobalt secondary action, vermilion primary generation action, lime/amber/red/green status colors, compact controls 和 image-first asset board.
- 将 Gallery 重新设计为 asset board, 每个 asset item 展示 image preview, current version, version count, review state, provider/model, task origin 和 album context.
- 将 Albums 重新设计为 collection management workspace, 支持 manual ordering, smart rule preview, album-scoped asset board, batch add, remove, rename/delete 和 smart query validation.
- 将 Review 重新设计为 staged metadata workbench, 明确 pending suggestion, local review draft, generated result apply/ignore, history compare 和 canonical metadata accept 边界.
- 将 Queue 重新设计为 operations console, 支持 batch composer, queued ordering, status-specific actions, task detail, attempts, timeline, logs, outputs, asset links 和 review links.
- 将 Settings 聚焦为 library lifecycle, provider diagnostics 和 global logs, task debugging 通过 deep link 回到 Task Detail.
- 将 Inspector 定义为 selected asset command surface, 展示 prompt, file integrity, album memberships, version lineage, pending review state, task origin 和 variation entry.
- 增加或调整 Rust core / Tauri read models, 为 Studio Console 暴露稳定屏幕级 payload, 避免 React 从多个低级 command 临时拼装跨 workflow 语义.
- 重构 React desktop 前端边界, 将 `main.tsx` 收敛为 bootstrap 和 top-level orchestration, 将 shell, workflows, data hooks 和纯状态 helper 拆分为可维护模块.
- 保留 review-first metadata invariant: AI generated metadata suggestion 在用户接受前不得写入 canonical asset metadata.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: 修改桌面 shell, visual system, workflow information architecture, responsive behavior, component boundaries, Studio read model usage, Gallery, Albums, Review, Queue, Settings 和 Inspector 的可观察行为.
- `metadata-review`: 修改 Review draft 语义, generated result apply/ignore, stale result handling, suggestion history compare, confidence display 和 canonical-write-only-on-accept 要求.
- `image-generation`: 修改 generation task completion 到 asset version, pending suggestion, task outputs, Gallery/Inspector/Review deep link 的可追溯行为.
- `albums-search`: 修改 Albums workspace, smart album live preview, album-scoped asset board 和 album context read model 行为.
- `app-logs`: 修改 diagnostics 和 log-to-task linking 要求, 明确 Settings Logs 与 Task Detail 的职责边界.
- `resource-library`: 修改 Studio Overview 所需 library/provider status read model 行为, 支持 Library Context 和 Settings diagnostics.

## Impact

- 主要影响 `apps/desktop/src/main.tsx`, `apps/desktop/src/styles.css`, 以及后续拆出的 React shell/workflow/components/hooks/state modules.
- 影响 `apps/desktop/src-tauri` command boundary 和 `crates/imglab-core` read model/service boundary, 需要新增或调整 Studio Console 屏幕级 read models.
- 可能影响 `crates/imglab-core` 中 Gallery, Review, Album, Task, App logs 和 Library registry 的 read APIs, 但不要求重写 daemon, provider adapter, SQLite persistence model 或 resource library layout.
- 需要更新 desktop frontend tests, core read model tests, task/review transition tests, TypeScript build, Rust tests 和多视口视觉 QA.
- 视觉 demo 位于 `apps/desktop/design-demos/`, 仅作为参考, production implementation 必须使用真实 React 组件和真实数据流.
