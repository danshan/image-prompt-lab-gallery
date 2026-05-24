## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供重新设计后的 desktop interface system, 不再以旧 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` 或旧 sidebar / 右侧常驻 inspector 为目标布局. 新 shell SHALL 表达为 `Command Bar | Workspace Switcher | Workflow Surface | Context Drawer`, 其中 Workflow Surface 是稳定主工作区, Context Drawer 在大屏 MAY dock, 在 compact / narrow viewport 下 MUST 可转换为 overlay drawer 或 bottom sheet. Workbench implementation MUST 按 shell, workflow, design system, locale boundary, controller hooks 和 pure state helpers 划分职责, 避免单个入口组件承载 Gallery, Albums, Prompts, Review, Queue, Settings, drawer 和 IPC orchestration 的全部逻辑. 桌面应用 MUST 将 `960px` 作为一等 compact desktop 最小宽度目标, 并支持低于 `960px` 的 narrow fallback.
当前 production redesign MUST 对齐 `apps/desktop/design-demos/canvas-deck-redesign.html` 的 Canvas Deck 产品形态: Command Bar 顶部固定, Workspace Switcher 在主工作区上方以水平 segmented tabs 为默认大屏形态, Gallery filter stack 仅作为 workflow-local 工具存在, 不得恢复为全局旧 sidebar.

#### Scenario: 新 Desktop Interface Shell
- **WHEN** 用户在 normal desktop 打开桌面应用
- **THEN** 应用展示 Command Bar, Workspace Switcher, Workflow Surface 和 Context Drawer capability
- **AND** 不依赖旧 sidebar, 旧 Library Context panel 或旧右侧常驻 inspector 承载主要导航

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** shell components, Gallery, Albums, Prompts, Review, Queue, Settings, drawer, design tokens, locale dictionaries, controller hooks 和 pure state helpers 位于职责明确的模块中

#### Scenario: Command Bar 承载全局工作台入口
- **WHEN** 用户查看任意 workspace
- **THEN** Command Bar 提供当前 library, global search 或等价搜索入口, create/run 主入口, review/queue indicators, theme toggle, language switch 和 app-level status

#### Scenario: Workspace Switcher 承载一级导航
- **WHEN** 用户在 Gallery, Albums, Prompts, Review, Queue 和 Settings 之间切换
- **THEN** Workspace Switcher 保持为一级导航来源
- **AND** wide viewport 下默认显示为水平 segmented tabs, 而不是旧垂直 sidebar 或永久 left rail
- **AND** 切换 workspace 不会隐式修改 Gallery query, Review draft 或 Prompt draft

#### Scenario: Compact Desktop Shell
- **WHEN** 桌面窗口宽度处于 `960px - 1279px`
- **THEN** Workflow Surface 保持为主工作区
- **AND** Context Drawer 可折叠, overlay 或临时 dock
- **AND** 主要导航, 当前 workflow 主操作, detail 和 recovery action 均可达

#### Scenario: Narrow Viewport Fallback
- **WHEN** 桌面窗口宽度小于 `960px`
- **THEN** 应用允许 workflow 退化为单列, stacked panels, segmented panels 或 bottom sheet
- **AND** 页面不得出现永久横向溢出, 主要导航不可达, 主操作被固定元素遮挡或 recovery action 不可达

### Requirement: Sidebar 支持 Active View 二级导航

桌面应用 SHALL 用 Workspace Switcher 和 workflow-local navigation 替代旧 Sidebar active-view second-level context panel. Album items MUST 只在 Albums workflow 的局部 album manager 中展示. Settings sections MUST 在 Settings workflow 内作为 page-local sections 展示. Gallery MUST 继续作为 all-assets browser, 不因 Albums 的 selected album 隐式改变 scope.

#### Scenario: Gallery 不展示 Album 二级树
- **WHEN** 用户打开 Gallery
- **THEN** Gallery 展示 all-assets browser 和显式 filters
- **AND** 不展示 Albums selected item tree 作为 Gallery scope

#### Scenario: Albums 拥有 Album 导航
- **WHEN** 用户打开 Albums
- **THEN** Albums workflow 展示 album search, create album, album list 和 selected album surface
- **AND** 点击 album item 保持 active workspace 为 Albums
- **AND** Gallery query 不因该点击而改变

#### Scenario: Settings 拥有 Page-Local Sections
- **WHEN** 用户打开 Settings
- **THEN** Settings workflow 展示 `Libraries`, `Providers`, `Updates` 和 `Logs` sections
- **AND** sections 不依赖旧 Sidebar second-level context panel

### Requirement: 提供 Gallery 和 Albums 视图

桌面应用 SHALL 提供重新设计后的 Gallery asset board 和 Albums collection management workspace. Gallery SHALL 是 image-first all-assets browser, 支持 compact filter/search/sort command surface, selection action bar 和 asset Context Drawer. Albums SHALL 拥有 collection management workflow, 支持 manual album 和 smart album 管理, album list ordering, album detail, manual item ordering, batch add, remove asset, rename/delete 和 smart rule preview.

#### Scenario: 打开 Gallery Asset Board
- **WHEN** 用户打开 Gallery
- **THEN** Workflow Surface 展示 image-first asset board
- **AND** asset item 展示 image preview, title, current version, review state, provider/model, rating, tags 和 album context 的紧凑摘要

#### Scenario: Gallery Filter 不遮挡主任务
- **WHEN** 用户使用 Gallery filters, search 或 sort
- **THEN** filter command surface 保持紧凑, 可换行或折叠
- **AND** 不覆盖 asset board 主内容或 selection action

#### Scenario: 打开 Asset Context Drawer
- **WHEN** 用户选择 Gallery asset
- **THEN** Context Drawer 展示 asset detail, prompt, file/version information, album memberships, review state 和可用 actions
- **AND** compact / narrow viewport 下 drawer 可关闭以恢复主工作区

#### Scenario: 打开 Albums 管理视图
- **WHEN** 用户打开 Albums
- **THEN** Workflow Surface 展示 album manager, selected album surface 和 add-to-album drawer capability
- **AND** Albums add-to-album workflow 不依赖 Gallery 的隐式选择状态

### Requirement: 提供 Review Inbox

桌面应用 SHALL 提供重新设计后的 staged metadata Review workspace, 用于处理 pending metadata suggestions, local review draft, generated field results, suggestion history 和 canonical metadata accept. Review workspace MUST 明确区分 pending suggestion, local draft, generated result 和 canonical metadata, 并在 compact / narrow viewport 下保持 accept, reject, restore, regenerate 和 recovery actions 可达.

#### Scenario: 打开 Review Workspace
- **WHEN** 用户打开 Review
- **THEN** Workflow Surface 展示 pending suggestion inbox 和 selected suggestion draft editor
- **AND** confidence, history, related task 和 generated result detail 可通过 inline panel 或 Context Drawer 查看

#### Scenario: Review 不混淆 Canonical Metadata
- **WHEN** Review 展示 pending suggestion, generated field result 或 local draft
- **THEN** UI 明确其为 staged, generated 或 draft 状态
- **AND** 不将其展示为已确认 canonical metadata

#### Scenario: Review Compact Layout
- **WHEN** Review 在 compact 或 narrow viewport 下展示
- **THEN** inbox, editor 和 context detail 可以 stacked 或 segmented 呈现
- **AND** 当前 draft 内容和 accept/reject/recovery actions 不得互相覆盖

### Requirement: 提供 Generation Composer 和 Queue

桌面应用 SHALL 将 Queue 作为重新设计后的 operations console, 包含 batch composer, task queue 和 task detail. Queue MUST 展示 running, queued, retry waiting, completed, failed 和 canceled tasks, 并根据 task status 提供合适操作. Task detail MAY 使用 Context Drawer, split panel 或 segmented panel 呈现, 但在 compact / narrow viewport 下不得遮挡 queue recovery actions.

#### Scenario: 打开 Queue Operations Console
- **WHEN** 用户打开 Queue
- **THEN** Workflow Surface 展示 batch compose, task queue 和 task detail capability
- **AND** compact / narrow viewport 下提供等价 panel switching 或 drawer behavior

#### Scenario: 查看 Task Detail Cross Links
- **WHEN** 用户选择一个 task
- **THEN** Task Detail 展示 attempts, timeline, log tail, outputs, Open asset, Open version 和 Open review suggestion links, 如果这些 links 存在

### Requirement: Prompt Workspace Responsive Layout

桌面应用 SHALL 将 Prompts 作为一等 workspace, 支持 prompt library, prompt draft editor, version list, run controls, render preview 和 output history. Prompts workspace MUST 使用重新设计后的 interface system, 在 wide viewport 下可以使用多列 split layout, 在 compact / narrow viewport 下使用 stacked sections, segmented panels 或 drawer, 且不得让 name input, save actions, editor textareas 或 run actions 重叠.

#### Scenario: Wide Prompt Workspace
- **WHEN** 用户在 wide viewport 打开 Prompts
- **THEN** Prompt Library, Editor 和 Run/Preview 区域可以同时展示
- **AND** 关键 actions 保持紧凑可达

#### Scenario: Compact Prompt Workspace
- **WHEN** 用户在 compact 或 narrow viewport 打开 Prompts
- **THEN** Prompts workspace 降级为 stacked 或 segmented layout
- **AND** prompt name, save draft, save version, render 和 run controls 不发生重叠

## ADDED Requirements

### Requirement: Desktop Interface Visual System

桌面应用 SHALL 使用 tokenized visual system 支持 light 和 dark theme. UI MUST 使用 semantic tokens 表达 background, surface, text, muted text, border, focus, overlay, action, status, shadow, radius, spacing, typography, control dimensions 和 z-index. 页面组件 MUST NOT 依赖旧 palette 或页面局部硬编码颜色作为主要视觉系统. 风格 SHALL 更现代, 更 app-native, 有明确设计感但不复杂臃肿. UI MUST 屏弃传统 website / web admin dashboard 风格, 避免 hero, landing-page section, oversized cards, marketing-style panels, decorative bento, decorative gradient orbs, heavy shadows 或过度复杂的按钮样式.

#### Scenario: Light Theme
- **WHEN** 用户切换到 light theme
- **THEN** 应用使用 light semantic tokens
- **AND** text, border, focus, action 和 status colors 保持可读对比度

#### Scenario: Dark Theme
- **WHEN** 用户切换到 dark theme
- **THEN** 应用使用 dark semantic tokens
- **AND** background, surface, text, border, focus, action 和 status colors 保持可读对比度

#### Scenario: Theme Preference
- **WHEN** 用户切换 theme
- **THEN** 桌面应用保存 theme preference
- **AND** 该 preference 不写入 resource library metadata 或 asset metadata

#### Scenario: Compact Controls
- **WHEN** 页面展示高频操作
- **THEN** UI 使用紧凑 icon button, row action, segmented control 或 small text button
- **AND** 不使用 oversized button 或装饰性 card 让主工作区失去信息密度

#### Scenario: App-Native Modern Style
- **WHEN** 开发者完成 desktop visual redesign
- **THEN** UI 呈现为 app-native productivity tool, 而不是传统 website, landing page 或 web admin template
- **AND** 视觉重点来自 command surfaces, refined spacing, subtle depth, stable interaction states 和真实工作内容

#### Scenario: Avoid Bloated Visual Complexity
- **WHEN** 页面展示 command surface, workflow content 或 drawer detail
- **THEN** UI 不使用多层嵌套 card, 过重阴影, 大面积装饰背景或复杂按钮结构
- **AND** 主工作内容保持清晰, 紧凑, 可扫描

### Requirement: Desktop Locale Boundary

桌面应用 SHALL 支持多语言 UI boundary, 初始至少支持 `en` 和 `zh-CN`. Shell, navigation, workflow labels, user-facing actions, empty states, loading states, error recovery labels 和 settings labels MUST 通过 locale dictionary 或等价 translator boundary 提供. UI components MUST NOT 继续散落主要用户可见文案. Locale preference MUST NOT 修改 resource library content, prompt content, metadata suggestion content, tags, category, task logs 或 provider output.

#### Scenario: 切换到 English
- **WHEN** 用户选择 `en`
- **THEN** shell, navigation, workflow labels, actions, empty/error/loading states 和 settings labels 使用 English 文案

#### Scenario: 切换到 Simplified Chinese
- **WHEN** 用户选择 `zh-CN`
- **THEN** shell, navigation, workflow labels, actions, empty/error/loading states 和 settings labels 使用简体中文文案

#### Scenario: Locale 不修改业务数据
- **WHEN** 用户切换 locale
- **THEN** resource library, prompt body, metadata suggestion, tags, categories, task logs 和 provider output 不被翻译或修改

#### Scenario: Dynamic Formatting
- **WHEN** UI 展示 count, date, bytes, status 或 action label
- **THEN** 应用通过 formatter helper 或 locale-aware boundary 生成用户可见文案

### Requirement: Context Drawer Behavior

桌面应用 SHALL 使用 Context Drawer 承载 asset detail, task detail, review context, prompt lineage, album add 和 diagnostics detail. Drawer MUST 支持 docked, overlay 或 bottom sheet presentation, 并在 compact / narrow viewport 下避免永久遮挡主操作. Drawer MUST 提供明确 close action, focus recovery 和 accessible label.

#### Scenario: Wide Docked Drawer
- **WHEN** viewport 足够宽且 drawer 打开
- **THEN** Context Drawer MAY dock 在 Workflow Surface 旁边
- **AND** Workflow Surface 保持可用且不发生内容重叠

#### Scenario: Compact Overlay Drawer
- **WHEN** compact 或 narrow viewport 下 drawer 打开
- **THEN** drawer 以 overlay 或 bottom sheet 呈现
- **AND** 用户可以关闭 drawer 返回主 workflow 操作

#### Scenario: Drawer Focus Recovery
- **WHEN** 用户关闭 drawer
- **THEN** focus 返回触发 drawer 的合理元素或当前 workflow 的稳定 fallback

### Requirement: Responsive Overlap Prevention

桌面应用 SHALL 在 wide, compact 和 narrow viewport 下防止主要组件重叠, 互相覆盖或产生不可恢复横向溢出. 所有 fixed, sticky, absolute 和 overlay 元素 MUST 使用统一 z-index scale. 长 library path, prompt, schema prompt JSON, checksum, log path, task id 和 provider error MUST 使用 truncation, wrapping 或 bounded internal scroll.

#### Scenario: Wide Viewport Check
- **WHEN** viewport width 为 `1440px` 或更宽
- **THEN** Gallery, Albums, Prompts, Review, Queue, Settings 和 Context Drawer 均无主要控件重叠

#### Scenario: Compact Viewport Check
- **WHEN** viewport width 为 `960px`
- **THEN** Gallery, Albums, Prompts, Review, Queue, Settings 和 Context Drawer 均保持主操作, detail 和 recovery action 可达

#### Scenario: Narrow Viewport Check
- **WHEN** viewport width 小于 `960px`
- **THEN** 应用不出现不可恢复横向溢出
- **AND** 主要导航, 当前 workflow 主操作和 drawer close action 可达

#### Scenario: Long Text Check
- **WHEN** 页面展示长 path, prompt, schema prompt JSON, checksum, log path, task id 或 provider error
- **THEN** 文本通过截断, 换行或内部滚动避免撑破整体布局

### Requirement: Desktop Redesign Verification

Desktop redesign implementation MUST include automated and manual verification evidence. Automated verification MUST cover theme switching, locale switching, drawer state, responsive shell state 和 workflow state helpers. Manual 或 browser-based verification MUST cover at least Gallery, Albums, Prompts, Review, Queue, Settings 和 Context Drawer across wide, compact 和 narrow representative viewports.

#### Scenario: Automated Frontend Verification
- **WHEN** 开发者验证该 redesign
- **THEN** `npm test --prefix apps/desktop` 覆盖 theme, locale, drawer state, responsive shell state 和关键 workflow state helpers
- **AND** `npm run build --prefix apps/desktop` 成功

#### Scenario: Browser Layout Verification
- **WHEN** 开发者验证该 redesign
- **THEN** 使用 dev server 或等价 browser QA 检查 `1440px`, `1280px`, `960px`, `768px` 和 `390px` viewport
- **AND** 检查 Gallery, Albums, Prompts, Review, Queue, Settings 和 drawer states 无组件重叠, 无关键操作不可达, 无页面横向溢出

#### Scenario: Visual System Audit
- **WHEN** 开发者完成 redesign
- **THEN** 检查主要 UI 颜色来自 semantic tokens
- **AND** icon-only controls 具备 accessible labels 和 visible focus
- **AND** 检查界面未退回传统 website / web admin dashboard 风格
