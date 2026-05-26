## Why

当前 Desktop shell 的一级导航和 Generate 输入占用了过多主工作区空间. Gallery, Albums 等高频页面需要更稳定的左侧导航, 同时每个页面顶部常驻的 Generate composer 和 overview counters 会压缩资产浏览与工作流操作区域.

本变更将一级导航改为 responsive sidebar, 并将 Generate 输入迁移到轻量 modal, 以提升 960px compact desktop 目标下的信息密度和可维护性.

## What Changes

- 将 Gallery, Albums, Prompts, Schedules, Review, Queue, Settings 的一级导航从 Workspace Switcher 调整为 responsive sidebar.
- Sidebar 支持展开与折叠:
  - 宽屏默认展开, 显示 icon, label 和 count label.
  - compact desktop 默认折叠, 显示 icon 和必要 count label.
- 在 shell 中提供 Generate 主入口, 打开轻量 modal.
- Generate modal 支持 provider, prompt 和 reference images 输入.
- Reference images 使用固定高度 compact strip, 缩略图不得撑高 modal.
- 移除页面主内容流中的 inline Generate composer 和常驻 overview counters.
- Gallery, Albums, Prompts, Schedules, Review 和 Queue 计数迁移到 sidebar count label, assets/status 等信息保留为更紧凑的 shell 或 page-local 文本.
- 不改变 generation task, provider adapter, daemon 或持久化契约.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `desktop-workbench`: 一级导航 shell, Generate 入口, overview counters 展示位置和 compact desktop 布局行为发生需求级变更.

## Impact

- 影响 `apps/desktop/src/app/shell/desktop-shell.tsx`, `apps/desktop/src/app/StudioAppController.tsx`, `apps/desktop/src/app/screens/workflows/composer.tsx`, `apps/desktop/src/app/screens/workflows/chrome.tsx`, `apps/desktop/src/app/i18n/dictionaries.ts` 和 `apps/desktop/src/styles.css`.
- 需要更新或新增 desktop frontend tests, 覆盖 sidebar 导航, Generate modal 和 reference image strip.
- 不影响 Rust core, daemon task manager, provider crates, SQLite schema, resource library 格式或 OpenSpec 中 generation provider boundary.
