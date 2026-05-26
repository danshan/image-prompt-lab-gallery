## Context

当前桌面前端实际由 `StudioAppController` 组装 `CommandBar`, `WorkspaceSwitcher`, `WorkflowSurface` 和 `ContextDrawer`. `WorkspaceSwitcher` 是一级导航来源, `GenerationComposer` 作为 inline section 渲染在每个 workspace 顶部附近, `StudioOverviewBand` 也常驻在主内容流中.

这与本次目标存在两个冲突:

- 一级菜单在顶部 / 横向切换区占用主内容的垂直空间, 且折叠能力有限.
- Generate composer 和 overview counters 在 Gallery, Albums, Review 等页面持续占用主工作区, 降低 960px compact desktop 下的信息密度.

仓库中还有旧 `studio-navigation.tsx` sidebar, 但它不属于当前 render path. 本变更不复活旧 shell, 而是在当前 `AppShell` boundary 内演进.

## Goals / Non-Goals

**Goals:**

- 用 responsive sidebar 替换当前 `WorkspaceSwitcher`, 作为 Gallery, Albums, Prompts, Schedules, Review, Queue, Settings 的唯一一级导航.
- Sidebar 支持展开和折叠, 折叠后保留 icon-only buttons 和必要 count label.
- Sidebar 承载全局 theme 和 language actions, 并跟随整体 theme tokens 切换.
- 将 Generate composer 从主内容流迁移到轻量 modal.
- Generate modal 支持 provider, prompt 和 reference images.
- Reference image thumbnails 高度受限, 不撑高 modal.
- 移除主内容流里的 `StudioOverviewBand`, 将关键计数迁移到 shell count label 或 compact status.
- 保持现有 generation task, daemon, provider 和 persistence boundary 不变.

**Non-Goals:**

- 不重建旧 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` shell.
- 不把 Generate modal 扩展为完整任务工作台.
- 不改 provider capability, image generation use case, daemon scheduler 或 SQLite schema.
- 不引入新的 UI framework 或 icon library.

## Decisions

### 1. 在当前 AppShell boundary 内实现 sidebar

采用当前 `AppShell` 作为 shell ownership, 增加或替换 `WorkspaceSwitcher` slot 为 `WorkspaceSidebar`. `StudioAppController` 继续负责 active view, counts, Generate open handler 和 theme/locale action wiring. Sidebar 同时接收 theme, locale, theme toggle 和 locale toggle, 使折叠态也能访问全局偏好操作.

替代方案是复用旧 `studio-navigation.tsx`, 但该组件承载旧 Library Context / second-level sidebar 语义, 与当前 shell spec 和实际 render path 不一致. 复用它会把已废弃 ownership 带回生产路径.

### 2. Sidebar collapsed state 属于 shell/controller 层

新增 `sidebarCollapsed` state, 初始值由 viewport breakpoint 推导:

- `>= 1280px` 默认展开.
- `960px - 1279px` 默认折叠.
- `< 960px` 保持 compact 可用降级.

用户手动 toggle 后, 当前会话内优先尊重用户选择. 本次不要求持久化到 `localStorage`, 以免把 presentation preference 和现有 locale/theme persistence 混在一起.

### 3. Generate modal 复用现有 generation state 和 submit path

Modal 使用现有 `prompt`, `provider`, `composerOpen`, `composerInputVersionId`, `composerInputFile`, `composerInputFileName`, `generationSubmitting` 和 `startGeneration`. `openComposerForTextGeneration`, `openComposerForReferenceGeneration`, `openComposerForVersionGeneration` 继续作为入口, 只改变 presentation.

这能避免 GUI adapter 重新实现 generation request planning, 也不会影响 daemon image task boundary.

### 4. Reference images 使用 fixed-height compact strip

Reference area 固定高度约 `48px`, thumbnail 固定约 `40px` square. 超出数量使用横向 overflow 或 `+N` indicator, 不增加 modal 高度. 这样 reference images 可见, 但不会把轻量 modal 变成大面板.

### 5. Overview counters 迁移到 shell 信息架构

`StudioOverviewBand` 从主内容流移除. Gallery, Albums, Prompts, Schedules, Review 和 Queue 数量由 sidebar count label 表示, assets/status 使用 command bar compact text 或 page-local toolbar 文本表达. Integrity issues 回到 Settings / library status 语义中, 不再每页常驻.

### 6. Sidebar 使用 theme-aware tokens

Sidebar 不使用固定黑白值表达全部状态, 而是定义 sidebar-specific theme tokens. Light theme 下保持高对比 dark rail, dark theme 下使用与 app surface 协调的深色 rail, active item, muted label, hover 和 border 都从 tokens 派生. 这样 theme 切换不会出现 active item 或按钮文字对比错误.

## Risks / Trade-offs

- `styles.css` 已有多轮 shell/nav override → 实现时优先约束在当前 `.desktop-app-shell`, `.command-bar`, `.workflow-sidebar`, `.workflow-surface`, `.generation-modal` selector, 避免修改旧 `.workbench` 路径.
- Sidebar 改变既有 spec 中 horizontal Workspace Switcher 要求 → 本 change 提供 delta spec 明确新 requirement, archive 时同步主 spec.
- 折叠态 icon-only 可能降低可发现性 → 每个 nav button 必须提供 `aria-label`, `title` 或等价 tooltip, count label 保持可读.
- Modal focus 管理遗漏会造成 keyboard trap 或焦点丢失 → 实现时复用 `ContextDrawer` 的 focus return 思路, 为 modal 添加 Escape close 和 focus return.
- 移除 overview band 可能减少即时 KPI 可见性 → 保留 sidebar count label 和 compact status, 将低频 integrity 状态放回 Settings.

## Migration Plan

这是 presentation-only frontend change, 不需要数据迁移.

实施顺序:

1. 新增 responsive sidebar shell 组件和 state wiring.
2. 将 Generate composer 渲染路径迁移为 modal.
3. 从 workspace 主内容流移除 inline composer 和 overview band.
4. 更新 CSS 和 i18n.
5. 更新 frontend tests 和手工 responsive 验证.

回滚策略是恢复 `WorkspaceSwitcher` slot, inline `GenerationComposer` 和 `StudioOverviewBand` render path. 因为不修改持久化或 backend contract, 回滚不需要 library migration.

## Open Questions

无. 用户已确认 responsive sidebar 采用宽屏展开 / compact 折叠, Generate modal 采用轻量形态, 且 reference thumbnails 必须限制高度.
