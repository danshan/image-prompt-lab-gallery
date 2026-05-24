## Context

当前 desktop 已经具备多个稳定能力: Gallery 是 all-assets browser, Albums 拥有 add-to-album workflow, Prompts 管理 prompt draft/version/run, Review 保持 review-first metadata, Queue 管理 daemon tasks, Settings 管理 libraries/providers/updates/logs. 这些能力应保留, 但现有界面结构和视觉语言不再作为设计约束.

本 change 选择完全重做 desktop interface system, 而不是在旧 sidebar 和 inspector 上继续修补. 设计目标是让用户把注意力放在图片资产, prompt, metadata review 和 task execution 上, 让导航和工具保持轻量, 稳定, 可扫描. 视觉方向应更现代, 更 app-native, 有设计感但不复杂臃肿, 并明确避免传统 website 或 web admin dashboard 的页面气质.

## Goals / Non-Goals

**Goals:**

- 推翻当前 sidebar, 右侧边栏, workspace 组织, 配色和页面布局.
- 保留当前所有基础功能和业务流程, 但重新组织交互路径和页面内部布局.
- 建立现代 app-native visual language, 避免传统 website / web admin dashboard 风格.
- 支持 `en` 和 `zh-CN` locale, 并为后续语言扩展保留明确边界.
- 支持 light/dark theme, 且 theme token 覆盖 surface, text, border, action, status, focus 和 overlay.
- 在大屏, compact desktop 和小屏下保证导航, 主操作, detail 和 recovery action 可达.
- 避免大按钮, 重装饰, marketing-style panel 和低信息密度布局.
- 将 UI 文案, theme, layout state 和 workflow state 分离, 降低后续维护成本.

**Non-Goals:**

- 不修改 resource library format, SQLite schema 或 migration semantics.
- 不修改 provider execution, daemon task persistence 或 generation planning 语义.
- 不引入 cloud sync, 多用户协作或 remote service 模型.
- 不把当前页面设计作为兼容目标.
- 不强制在本 change 中引入新 icon dependency; 可以继续使用现有 SVG icon boundary, 但实现必须是可替换的.

## Decisions

### 0. Canvas Deck Demo 是本轮生产重构的视觉输入

本 change 的实现 MUST 以 `apps/desktop/design-demos/canvas-deck-redesign.html` 中的 Canvas Deck demo 作为当前视觉和布局输入. 如果既有实现与 demo 冲突, 以 demo 表达的产品形态为准, 但不得改变本文件列出的业务边界和持久化 non-goals.

Canvas Deck 的核心形态是:

- 顶部 Command Bar 承载 brand, library, global search, compact stats, primary generate, review / queue indicators, theme, locale 和 settings.
- Workspace Switcher 在主工作区上方以水平 segmented tabs 呈现, 而不是旧 sidebar 或永久左 rail.
- Workflow Surface 使用 image/workflow-first board, compact panels 和稳定 drawer/sheet, 避免 marketing page section 和大型装饰 card.
- Gallery 允许保留 workflow-local filter stack 或 compact filter surface, 但它是 Gallery 的局部工具, 不是全局 Sidebar.
- 大屏和小屏都必须保持组件稳定尺寸, 语言切换不得造成按钮, stats, badges 或 tabs 被裁切.

### 1. 新 Shell: Command Bar + Workspace Switcher + Workflow Surface + Context Drawer

顶层界面由四个稳定区域组成:

- `Command Bar`: 当前 library, global search, create/run, review/queue indicators, theme toggle, language switch 和 app-level status.
- `Workspace Switcher`: Gallery, Albums, Prompts, Review, Queue, Settings 的一级导航. 大屏默认以水平 icon + label segmented tabs 呈现, 小屏可压缩为 icon-only 或底部/顶部 segmented navigation.
- `Workflow Surface`: 当前 workflow 的唯一主工作区, 不再被旧 Library Context 和右侧常驻 Inspector 持续挤压.
- `Context Drawer`: asset, task, prompt lineage, review suggestion, album add 和 diagnostics detail 的上下文容器. 大屏允许 dock, 小屏以 drawer / sheet 呈现.

旧 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` 不再是目标布局. Activity 信息并入 Command Bar 和 Queue indicator, Library context 并入 Command Bar 与 Settings/Libraries.

### 2. Workflow Presentation

每个 workflow 重新定义页面结构:

- Gallery: image-first board, 顶部 compact filter/search/sort bar, selection action bar, asset detail drawer.
- Albums: album list + selected album surface + add-to-album drawer. Gallery scope 不继承 album selection, Albums 自己管理 collection workflow.
- Prompts: prompt library, editor, run preview 在大屏使用三列或可调 split, compact 下使用 section tabs / stacked panels.
- Review: inbox list, metadata draft editor, confidence/history/task context drawer, batch action bar.
- Queue: batch compose, queue list, task detail 使用 split 或 segmented panels, daemon offline/error state 明确.
- Settings: Libraries, Providers, Updates, Logs 作为 page-local sections, 使用紧凑表格和行内 icon actions.

Workflow 内部可以使用局部 drawer, split panel 或 segmented control, 但不得重新引入全局右侧边栏作为唯一详情入口.

### 3. Responsive Contract

布局至少定义三档:

- `wide`: `>= 1280px`. Command Bar 固定顶部, workspace switcher 可展示 label, Context Drawer 可以 dock 到右侧.
- `compact`: `960px - 1279px`. Workflow Surface 优先, drawer 默认 overlay 或临时 dock, workspace switcher 压缩, filter/action bar 换行但不覆盖内容.
- `narrow`: `< 960px`. 单列或 stacked panels, drawer/bottom sheet 承载详情, 主操作和 recovery action 必须可达, 允许内部滚动但禁止页面横向溢出.

任何 fixed, sticky, absolute 或 overlay 元素都必须使用统一 z-index scale. 长 path, prompt, JSON, checksum, log, task id 必须通过 wrap, truncation 或 bounded scroll 处理.

### 4. Theme System

Theme 使用 semantic tokens, 而不是页面内硬编码颜色. 视觉语言应偏 app-native: 更接近精密桌面工具, command surface 和轻量 workspace, 而不是传统 web page section. 最小 token 集:

- `color.background`, `color.surface`, `color.surfaceSubtle`, `color.text`, `color.textMuted`.
- `color.border`, `color.focus`, `color.overlay`.
- `color.actionPrimary`, `color.actionSecondary`, `color.actionDanger`.
- `color.statusSuccess`, `color.statusWarning`, `color.statusDanger`, `color.statusInfo`, `color.statusNeutral`.
- `shadow`, `radius`, `spacing`, `controlHeight`, `fontSize`, `zIndex`.

Light theme 使用清爽中性色和少量功能色. Dark theme 使用低亮度 background, 保证 text, border, focus 和 status 对比度. 禁止使用一整套单色调紫蓝渐变或装饰性 orb 作为主视觉.

设计边界:

- 使用 refined contrast, thin dividers, subtle depth, clear interaction states 和 image/workflow 内容本身形成视觉重点.
- 可以使用轻量 translucency, inset surface, command palette style 和 docked drawer motion, 但必须服务于信息组织.
- 避免 hero, landing-page section, oversized cards, bento-for-decoration, floating marketing panels, heavy gradients, decorative blobs, excessive shadows 和过度复杂的按钮结构.
- 按钮以 icon, icon + short label, segmented control, row action 为主. Primary action 必须醒目但小而稳定.
- 卡片只用于 asset tile, repeated item, modal/drawer content group, 不把 page section 套成一层层 card.

### 5. Locale Boundary

Desktop frontend 必须提供 locale dictionary 和 translator boundary:

- 初始支持 `en` 和 `zh-CN`.
- Locale preference 可存于 frontend local preference 或现有 app setting boundary, 不得影响 library data.
- UI components 使用 translation keys, 不直接写面向用户的英文字符串.
- Dynamic labels 可以通过 formatter helpers 处理 count, date, bytes 和 status.
- 业务数据, prompt 内容, metadata suggestion 内容, tags, category 和 logs 不翻译.

### 6. Interaction Principles

- 高频操作优先 icon button, compact text button 或 row click, 不使用大面积 CTA.
- Primary action 每个 workflow 同屏最多保持一个明确主操作.
- Batch action 只在 selection 存在时出现, 不长期占用主工作区.
- Drawer open/close, selection, active workflow 和 compact mode 属于 UI state; Gallery query, Review draft, Prompt draft 等仍归对应 workflow state.
- 所有 icon-only controls 必须有 accessible label.

### 7. Implementation Boundary

Implementation 应优先重构 frontend shell 和 workflow layout, 不碰 Rust core 除非发现现有 Tauri payload 无法承载当前功能. 这次 redesign 不要求新增 persistence 字段.

建议模块:

- `app/design-system/tokens.ts`
- `app/i18n/dictionaries.ts`
- `app/i18n/use-locale.ts`
- `app/shell/command-bar.tsx`
- `app/shell/workspace-switcher.tsx`
- `app/shell/context-drawer.tsx`
- `app/shell/responsive-layout.ts`
- workflow-owned layout modules under `app/workflows/*`

保持 existing controller hooks 和 pure state helpers 可复用, 但不要让旧 `studio-navigation.tsx` 和 `studio-shell.tsx` 的结构约束新设计.

## Risks / Trade-offs

- [Risk] Scope 过大导致实现长期半成品. Mitigation: 先实现 shell, theme, locale 和 responsive contract, 再逐个 workflow 替换.
- [Risk] 保留功能时不小心保留旧交互. Mitigation: tasks 明确要求替换旧 sidebar / inspector / palette, 只保留业务能力.
- [Risk] i18n 改造牵动大量组件. Mitigation: 从 shell 和 visible workflow labels 开始, 后续逐步替换所有 UI copy, 但本 change 验收要求不能留下主要 UI hardcoded copy.
- [Risk] Overlay drawer 在小屏遮挡主操作. Mitigation: drawer open 时必须有明确 close, focus containment 和 backdrop, 且主操作可通过关闭 drawer 恢复.
- [Risk] 现代化设计滑向 website 或复杂 dashboard. Mitigation: spec 明确禁止 hero/marketing section/装饰性大卡片, 并通过 QA 检查按钮密度, surface 层级和主工作区信息密度.
- [Risk] Theme token 覆盖不完整. Mitigation: CSS 中禁止新增页面级 hardcoded palette, 验证时扫描关键颜色使用.

## Verification Strategy

- `npm test --prefix apps/desktop`: 覆盖 locale, theme, layout state 和 workflow state helpers.
- `npm run build --prefix apps/desktop`: 验证 TypeScript / Vite build.
- Browser QA: dev server 下检查至少 `1440px`, `1280px`, `960px`, `768px`, `390px` 宽度.
- Visual QA pages: Gallery, Albums, Prompts, Review, Queue, Settings, asset drawer, task drawer, review drawer.
- Layout audit: 检查无横向滚动, 无主要控件重叠, drawer 不永久遮挡主操作, long text 有 wrap/truncate/scroll 策略.
- Accessibility spot check: icon-only labels, focus visible, theme contrast, keyboard reachable drawer close.

## Open Questions

无阻塞问题. 如果实现中发现当前 Tauri command payload 无法支撑某个现有功能的新交互展示, 应先补充本 change 的 tasks/spec, 再扩大到 Rust read model.
