## Why

当前 desktop UI 已经承载 Gallery, Albums, Prompts, Review, Queue, Settings 和 Inspector 等核心能力, 但界面模型仍继承旧的 sidebar / context panel / workspace / inspector 组合. 这套结构在小屏和复杂 workflow 下容易让导航, 详情, 筛选和操作互相挤压, 也缺少系统级 theme, locale 和 responsive contract.

本变更将整个桌面应用重新设计为简洁高效且更现代的 app-native 工作台界面: 完全重做 shell, 配色, 布局和交互组织, 保留当前业务能力和 workflow ownership, 并把 light/dark, 多语言和多视口无重叠作为一等验收条件. 新设计 MUST 屏弃传统 website / web admin dashboard 的页面感, 有明确设计感, 但不得复杂臃肿.

## What Changes

- 将当前 sidebar, 右侧 inspector, workspace, palette 和页面布局全部替换为新的 desktop interface system.
- 引入 `Command Bar + Workspace Switcher + Workflow Surface + Context Drawer` 信息架构.
- 将 library switch, global search, primary create/run, review/queue indicators, theme toggle 和 language switch 统一收敛到轻量 Command Bar.
- 将 Gallery, Albums, Prompts, Review, Queue 和 Settings 保持为一级 workspace, 但重新设计每个 workspace 的内部布局和交互.
- 将右侧常驻 inspector 改为 Context Drawer: 大屏可以 dock, compact / small viewport 下转换为 drawer 或 bottom sheet, 不得遮挡主操作.
- 引入 tokenized light/dark visual system, 使用现代 app-native visual language, 低干扰高对比的工作台配色, compact controls, 稳定 spacing, focus 和 status tokens.
- 屏弃传统 web site / admin template 风格, 避免 hero, 大卡片堆叠, marketing-style section, 过度圆角, 过重阴影和复杂装饰.
- 引入 desktop frontend locale boundary, 至少支持 `en` 和 `zh-CN`, UI 文案不得继续散落在 workflow components 中.
- 明确 responsive contract: 大屏, compact desktop 和小屏均必须保证主要导航, 当前 workflow 主操作, 详情和错误恢复操作可达.
- 保留当前业务能力: resource library lifecycle, Gallery filters, Albums add-to-album, Prompt Workspace, Review metadata draft, Queue batch/tasks, Settings libraries/providers/updates/logs, asset detail/version/lineage/actions.
- 不改变 Rust core 业务语义, SQLite schema, provider execution, daemon task model, asset lineage, review-first metadata invariant 或 resource library 持久化契约.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: 修改 desktop shell, navigation, workflow presentation, context detail model, responsive behavior, visual system, theme switching, locale boundary 和多视口验证要求.

## Impact

- 主要影响 `apps/desktop/src/app`, `apps/desktop/src/studio-shell.tsx`, `apps/desktop/src/studio-navigation.tsx`, `apps/desktop/src/styles.css`, `apps/desktop/src/studio-icons.tsx` 和 workflow screen modules.
- 需要新增或替换 frontend-only design system modules, 例如 theme tokens, locale dictionaries, command bar, workspace switcher, context drawer 和 responsive layout helpers.
- 可能需要调整 React component props 和 controller wiring, 但不要求修改 Rust persistence, resource library schema, provider crates 或 daemon API 语义.
- 需要新增 frontend tests 覆盖 theme, locale, responsive shell state, drawer behavior 和 workflow reachability.
- 需要通过 dev server 或等价 browser QA 检查大屏和小屏布局, 重点验证无组件重叠, 无关键操作被遮挡, 长文本不撑破布局.
