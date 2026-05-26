# Sidebar navigation and Generate modal design

## 目标

将 Gallery, Albums, Prompts, Schedules, Review, Queue, Settings 等一级菜单从当前顶部 / 侧向 switcher 体验调整为 responsive sidebar. Sidebar 需要支持折叠, 折叠后只显示 icon 按钮和必要 badge.

同时移除每个页面顶部常驻的 Generate composer 和 overview counters, 减少主工作区垂直占用. Generate 由 shell 中的 Generate 入口打开轻量 modal, modal 支持 provider, prompt, reference images 三类输入.

## 当前上下文

当前实际 render path 是 `apps/desktop/src/app/StudioAppController.tsx` 组装 `CommandBar`, `WorkspaceSwitcher`, `WorkflowSurface`, `ContextDrawer`, 由 `apps/desktop/src/app/shell/desktop-shell.tsx` 提供 shell components.

仓库中存在旧的 `apps/desktop/src/studio-navigation.tsx` sidebar, 但它不在当前 render path 上. 本次不复活旧组件, 而是在当前 `AppShell` boundary 内重构, 避免引入两套 shell ownership.

## UI / UX 设计依据

本设计基于 `ui-ux-pro-max` 检索结果和项目约定:

- Desktop app 属于 data-dense / operational dashboard, 应优先紧凑, 可扫描, 面向重复操作.
- Sidebar 和 modal 不应遮挡或挤压主要内容到不可用状态. 固定导航必须通过 grid layout 预留空间, 不能 overlay 主内容.
- 所有折叠态按钮必须有可访问名称, tooltip 或 `title`, 并保持 keyboard navigation 顺序与视觉顺序一致.
- Modal 需要 focus management: 打开后聚焦可操作元素, Escape 关闭, 关闭后焦点回到触发按钮.
- Hover / active state 使用颜色, 边框, 背景变化, 不使用会导致 layout shift 的 scale transform.

## Sidebar 设计

Sidebar 替换当前 `WorkspaceSwitcher`, 成为一级导航的唯一主入口. 顶部 `CommandBar` 保留全局搜索, library/status, theme, locale, settings 等全局动作, 但不再承担一级菜单切换.

Sidebar 项目:

- Gallery.
- Albums.
- Prompts.
- Schedules.
- Review, 带 pending review badge.
- Queue, 带 queue badge.
- Settings.
- Generate action, 作为高频主操作.

响应式行为:

- `>= 1280px`: 默认展开. 宽度约 `216px` 到 `240px`, 显示 icon, label, badge.
- `960px` 到 `1279px`: 默认折叠. 宽度约 `64px` 到 `72px`, 仅显示 icon 和 badge.
- `< 960px`: 不强行占据主工作区. 保持 compact icon rail 或等价可用降级, 目标是避免页面横向不可用.

状态 ownership:

- `sidebarCollapsed` 放在 shell / controller 层, 不进入 workflow-specific state.
- 初始值由 viewport breakpoint 推导.
- 用户手动 toggle 后, 当前会话内尊重用户选择.
- `localStorage` 持久化可以作为后续增强, 不作为本次必须项.

## Generate modal 设计

`GenerationComposer` 不再作为 inline section 出现在每个页面顶部. Shell 中的 Generate 按钮打开轻量 modal. Modal 只管理本次 generation request inputs, 不承载 task history, provider health, metadata review 或 queue detail.

Modal 内容:

- Header: `Generate`, close button, source context.
- Provider: 使用现有 provider state.
- Reference images: fixed-height compact strip.
- Prompt: textarea.
- Footer: request scope note 和 Run button.

Reference images 约束:

- Reference strip 高度固定为约 `48px`.
- Thumbnail 本体固定为约 `40px` square.
- 缩略图不能纵向撑高 modal.
- 超出数量使用横向 overflow 或 `+N` overflow indicator, 不增加 modal 高度.
- Reference images 属于本次生成请求输入, 不写入 canonical asset metadata.

Modal 行为:

- `CommandBar` Generate 和 sidebar Generate 都调用现有 `openComposerForTextGeneration(true)`.
- Inspector 中的 Generate variation / Generate from reference 继续调用现有 open 函数, 只是打开 modal 而不是 inline composer.
- 关闭 modal 只关闭 composer. 不因为关闭而清空 prompt 或 reference source, 避免误关导致输入丢失.
- Run 继续调用现有 `startGeneration(composerInputVersionId, composerInputFile)`.
- Disabled 条件保持当前逻辑: submitting 或 prompt 为空.

## Overview counters 设计

页面内 `StudioOverviewBand` 从主内容流移除. 关键状态迁移到 shell:

- Review 和 Queue 数量由 sidebar badge 表示.
- Assets 数量和 status 使用 command bar compact text 或当前页面 toolbar 的上下文文本表示.
- Integrity issues 不再作为每页常驻 KPI, 后续保留在 Settings / library status 语义下.

## 组件与文件影响

预计影响范围:

- `apps/desktop/src/app/shell/desktop-shell.tsx`: 增加 responsive sidebar shell slot 或替换 workspace switcher slot.
- `apps/desktop/src/app/StudioAppController.tsx`: 接入 sidebar collapsed state, 调整 slots, 将 `GenerationComposer` 渲染为 modal.
- `apps/desktop/src/app/screens/workflows/composer.tsx`: 将 composer UI 改造成 modal-friendly content, 加入 reference images compact strip.
- `apps/desktop/src/app/screens/workflows/chrome.tsx`: 移除或降级 `StudioOverviewBand` 常驻使用, 保留 toolbar 中必要 search / filter.
- `apps/desktop/src/app/i18n/dictionaries.ts`: 补齐 sidebar / modal accessibility 文案.
- `apps/desktop/src/styles.css`: 重写当前 `desktop-app-shell`, `workspace-switcher`, `composer`, modal 相关样式. 需要清理或覆盖后段重复 selector.

不涉及:

- Rust core generation use case.
- Daemon task scheduling.
- Provider adapter boundary.
- Library persistence contract.
- Asset metadata review persistence.

## 验证计划

实现后建议验证:

- `npm test --prefix apps/desktop`.
- `npm run build --prefix apps/desktop`.
- `git diff --check`.
- 手工或 browser 检查 `>= 1280px`, `960px` 到 `1279px`, `< 960px` 三档布局.
- 检查 Gallery, Albums, Review, Queue, Settings 页面顶部不再出现 inline composer 和 overview band.
- 检查 Generate modal 的 provider, prompt, reference images strip 在 960px 宽度下不溢出, reference thumbnails 不撑高 modal.
- 检查 keyboard behavior: sidebar tab order, modal Escape close, close 后 focus return.

## 风险与取舍

主要风险是 `apps/desktop/src/styles.css` 已存在多轮 shell / nav selector override. 实现时应以当前实际 render path 为准, 避免顺手重构旧 `studio-navigation.tsx` 或 `.workbench` 路径.

本设计选择最小可维护改动: shell ownership 集中在当前 `AppShell`, generation 业务路径不变, 只改变 presentation 和 interaction entry. 更完整的 Studio Console shell 重构暂不纳入本次范围.
