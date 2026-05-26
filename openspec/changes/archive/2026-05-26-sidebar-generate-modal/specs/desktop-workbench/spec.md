## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供重新设计后的 desktop interface system, 不再以旧 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` 或旧 sidebar / 右侧常驻 inspector 为目标布局. 新 shell SHALL 表达为 `Command Bar | Responsive Sidebar | Workflow Surface | Context Drawer`, 其中 Workflow Surface 是稳定主工作区, Responsive Sidebar 是一级导航和 Generate 主入口, Context Drawer 在大屏 MAY dock, 在 compact / narrow viewport 下 MUST 可转换为 overlay drawer 或 bottom sheet. Workbench implementation MUST 按 shell, workflow, design system, locale boundary, controller hooks 和 pure state helpers 划分职责, 避免单个入口组件承载 Gallery, Albums, Prompts, Review, Queue, Settings, drawer 和 IPC orchestration 的全部逻辑. 桌面应用 MUST 将 `960px` 作为一等 compact desktop 最小宽度目标, 并支持低于 `960px` 的 narrow fallback. 当前 production redesign MUST 对齐 Studio Console 的紧凑生产工具形态: Command Bar 顶部固定, Responsive Sidebar 在左侧承载一级导航, Gallery filter stack 仅作为 workflow-local 工具存在, 不得恢复旧 Library Context sidebar 或旧右侧常驻 inspector.

#### Scenario: 新 Desktop Interface Shell
- **WHEN** 用户在 normal desktop 打开桌面应用
- **THEN** 应用展示 Command Bar, Responsive Sidebar, Workflow Surface 和 Context Drawer capability
- **AND** 不依赖旧 Library Context panel 或旧右侧常驻 inspector 承载主要导航

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** shell components, Gallery, Albums, Prompts, Review, Queue, Settings, drawer, design tokens, locale dictionaries, controller hooks 和 pure state helpers 位于职责明确的模块中

#### Scenario: Command Bar 承载全局工作台状态
- **WHEN** 用户查看任意 workspace
- **THEN** Command Bar 提供当前 library, global search 或等价搜索入口, theme toggle, language switch 和 app-level status
- **AND** Command Bar MAY 展示 compact assets/status summary
- **AND** Command Bar 不再作为一级 workspace navigation 的主要来源

#### Scenario: Responsive Sidebar 承载一级导航
- **WHEN** 用户在 Gallery, Albums, Prompts, Schedules, Review, Queue 和 Settings 之间切换
- **THEN** Responsive Sidebar 保持为一级导航来源
- **AND** wide viewport 下默认展开并展示 icon, label 和 count label
- **AND** compact desktop viewport 下默认折叠并展示 icon-only buttons 和必要 count label
- **AND** Responsive Sidebar 提供全局 theme 切换和 language 切换入口
- **AND** Responsive Sidebar 的颜色和对比度跟随当前 theme
- **AND** 切换 workspace 不会隐式修改 Gallery query, Review draft 或 Prompt draft

#### Scenario: Sidebar Generate 入口打开轻量 Modal
- **WHEN** 用户点击 Responsive Sidebar 或 shell 中的 Generate 主入口
- **THEN** 桌面应用打开轻量 Generate modal
- **AND** modal 支持 provider, prompt 和 reference images 输入
- **AND** 页面主内容流不展示 inline Generate composer

#### Scenario: Compact Desktop Shell
- **WHEN** 桌面窗口宽度处于 `960px - 1279px`
- **THEN** Workflow Surface 保持为主工作区
- **AND** Responsive Sidebar 默认折叠为 icon-only rail
- **AND** Context Drawer 可折叠, overlay 或临时 dock
- **AND** 主要导航, 当前 workflow 主操作, detail 和 recovery action 均可达

#### Scenario: Narrow Viewport Fallback
- **WHEN** 桌面窗口宽度小于 `960px`
- **THEN** 应用允许 workflow 退化为单列, stacked panels, segmented panels, compact rail 或 bottom sheet
- **AND** 页面不得出现永久横向溢出, 主要导航不可达, 主操作被固定元素遮挡或 recovery action 不可达

### Requirement: Sidebar 支持 Active View 二级导航

桌面应用 SHALL 用 Responsive Sidebar 承载一级 workspace navigation, 并用 workflow-local navigation 承载二级上下文. Responsive Sidebar MUST NOT 恢复旧 active-view second-level context panel. Album items MUST 只在 Albums workflow 的局部 album manager 中展示. Settings sections MUST 在 Settings workflow 内作为 page-local sections 展示. Gallery MUST 继续作为 all-assets browser, 不因 Albums 的 selected album 隐式改变 scope.

#### Scenario: Gallery 不展示 Album 二级树

- **WHEN** 用户打开 Gallery
- **THEN** Gallery 展示 all-assets browser 和显式 filters
- **AND** 不展示 Albums selected item tree 作为 Gallery scope

#### Scenario: Albums 拥有 Album 导航

- **WHEN** 用户打开 Albums
- **THEN** Albums workflow 展示 album search, create album, album list 和 selected album surface
- **AND** 点击 album item 保持 active workspace 为 Albums
- **AND** Albums workspace 选中该 album
- **AND** Gallery query 不因该点击而改变

#### Scenario: Settings 拥有 Page-Local Sections

- **WHEN** 用户打开 Settings
- **THEN** Settings workflow 展示 `Libraries`, `Archived`, `Automation`, `Task Queue`, `Providers`, `Updates` 和 `Logs` sections
- **AND** sections 不依赖旧 Sidebar second-level context panel

## ADDED Requirements

### Requirement: Generate Modal 使用紧凑 Reference Images Strip

桌面应用 SHALL 在轻量 Generate modal 中支持 provider, prompt 和 reference images 输入. Reference images MUST 使用固定高度 compact strip, thumbnail MUST 使用固定 square 尺寸, 且 reference image 数量增加时 MUST NOT 增加 modal 的纵向高度.

#### Scenario: 打开 Generate Modal
- **WHEN** 用户从 shell Generate 入口打开 Generate modal
- **THEN** modal 展示 provider 选择, prompt 输入, reference images strip 和 Run 操作
- **AND** modal 不展示 task history, provider health, queue detail 或 metadata review controls

#### Scenario: Reference Thumbnails 不撑高 Modal
- **WHEN** modal 中存在一个或多个 reference images
- **THEN** reference strip 高度保持固定
- **AND** 每个 reference thumbnail 使用固定 square 尺寸
- **AND** 超出可见数量的 reference images 使用横向 overflow 或 overflow count 表达
- **AND** modal 高度不因 reference images 数量增加而增长

#### Scenario: Inspector Generation 入口复用 Modal
- **WHEN** 用户从 Inspector 点击 Generate variation 或 Generate from reference
- **THEN** 桌面应用打开同一个轻量 Generate modal
- **AND** modal 展示对应 source context 或 reference image
- **AND** 提交后继续使用既有 generation task path

### Requirement: Overview Counters 不常驻主内容流

桌面应用 SHALL 移除每个 workspace 主内容流中的常驻 overview counters. Gallery, Albums, Prompts, Schedules, Review 和 Queue 计数 MUST 迁移到 sidebar count label 或等价 shell indicator. Assets/status 信息 MUST 以 compact shell text 或 page-local context 表达.

#### Scenario: Workspace 顶部不展示 Overview Band
- **WHEN** 用户打开 Gallery, Albums, Review, Queue 或 Settings workspace
- **THEN** 主内容流不展示常驻 overview counters band
- **AND** Gallery, Albums, Prompts, Schedules, Review 和 Queue 数量仍可从 sidebar count label 或等价 shell indicator 获得

#### Scenario: Integrity Issues 不作为每页 KPI
- **WHEN** 用户查看非 Settings workspace
- **THEN** Integrity issues 不作为每页常驻 KPI 展示
- **AND** library integrity 状态仍可在 Settings 或 library status context 中查看
