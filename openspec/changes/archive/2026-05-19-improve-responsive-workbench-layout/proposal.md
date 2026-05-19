## Why

当前桌面端 Workbench 依赖固定三栏和多处页面内固定 grid 宽度, 在 compact desktop 窗口下容易出现控件挤压, 详情区域消失, popover 覆盖关键操作, 以及长文本撑破布局的问题. 这会直接影响 Gallery, Albums, Review, Queue 和 Settings 的日常可用性, 也会让后续页面继续堆叠 ad hoc breakpoint.

本变更将 `960px` 作为一等 compact desktop 目标, 用统一的 shell-first collapse system 重新定义全页面响应式行为.

## What Changes

- 调整 Workbench 顶层 shell: 宽屏保留 `Sidebar | Workspace | Inspector`, compact desktop 下 Sidebar 和 Inspector 都可折叠, Workspace 优先.
- 为 Gallery, Albums, Review, Queue 和 Settings 定义统一响应式布局规则, 避免页面级固定宽度互相漂移.
- Gallery 在 compact desktop 下通过 Inspector drawer 查看选中 asset 详情, 搜索和 filter toolbar 可换行.
- Albums 和 Review 使用 shared split-workspace 行为, compact desktop 下主 detail 区优先, list/selector 可折叠或堆叠.
- Queue 在 compact desktop 下使用本地 panel 切换, 确保 `Compose`, `Queue`, `Detail` 都可达.
- Settings Libraries 在 compact desktop 下从 fixed grid table 退化为 row cards, Logs preview 使用 bounded internal scroll.
- 收敛 CSS breakpoint 和 layout token, 明确长文本, path, JSON, log preview 的 wrapping, truncation 或 scroll 策略.
- 在响应式骨架之上收敛为 Studio Workbench 设计模板: 中性专业配色, image-first Gallery card, view-specific toolbar hierarchy, 统一 icon button 表达, 以及更符合桌面生产力工具的 Queue 中屏布局.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: 增加全页面 compact desktop 响应式行为要求, 包括 shell collapse, Inspector drawer, split workspace, Queue panel switching, Settings table-to-card layout, 以及视觉验证要求.
- `desktop-workbench`: 增加 Studio Workbench 视觉模板要求, 包括 Gallery 资产浏览优先级, 全局工具栏层级, icon-only affordance, restrained neutral palette, 以及 Queue 在宽屏和中屏下的并排工作效率.

## Impact

- 主要影响 `apps/desktop/src/main.tsx` 和 `apps/desktop/src/styles.css`.
- 可能增加少量 layout-only React state 和组件, 但不改变 Rust core, Tauri command, DTO, SQLite schema 或 provider 行为.
- 需要更新 `openspec/specs/desktop-workbench/spec.md` 的 delta spec.
- 需要通过 desktop build/test 和多视口人工或 browser screenshot 检查验证.
