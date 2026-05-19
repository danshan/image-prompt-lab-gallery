## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台
桌面应用 SHALL 提供 `Library Sidebar | Workspace | Inspector` 三栏工作台, 并允许 Sidebar 和 Inspector 在 compact desktop 窗口中折叠. Workbench implementation MUST 按 workflow 组件和 data hooks 划分职责, 避免单个入口组件长期承载 Gallery, Albums, Review, Queue, Settings, Inspector 和 IPC orchestration 的全部逻辑. Settings workspace SHALL 按 `Libraries` 和 `Logs` 子页拆分 Library lifecycle management 与 diagnostics. 桌面应用 MUST 将 `960px` 作为一等 compact desktop 最小宽度目标, 在该宽度下保证 Workspace 为优先主区域, 导航, 详情和关键操作均可达, 且不得出现组件错位或不可恢复覆盖.

#### Scenario: 选择 Gallery 图片
- **WHEN** 用户在 Gallery 中选择一个 asset
- **THEN** Workspace 保持图片网格上下文, Inspector 或 compact detail drawer 展示该 asset 的 metadata, prompt, tags, albums 和 versions

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** Gallery, Albums, Review, Task, Settings 和 Inspector 的主要 rendering 与 data orchestration 位于职责明确的组件或 hooks 中

#### Scenario: Settings 边界清晰
- **WHEN** 开发者维护 Settings workspace
- **THEN** Library lifecycle management 和 Logs diagnostics 位于职责明确的 Settings 子组件或 hooks 中

#### Scenario: Compact Desktop Shell
- **WHEN** 桌面窗口宽度处于 `960px` 到 `1279px`
- **THEN** 桌面应用将 Sidebar 和 Inspector 作为可折叠 shell 区域处理, 并让 Workspace 成为稳定主列

#### Scenario: Below First-Class Minimum
- **WHEN** 桌面窗口宽度小于 `960px`
- **THEN** 桌面应用允许页面退化为单列或内部滚动, 但不得让主要导航, 当前页面主操作或错误恢复操作永久不可达

## ADDED Requirements

### Requirement: Compact Gallery Layout
桌面应用 SHALL 在 compact desktop 下保持 Gallery 查询, 筛选, 图片网格和 asset detail 工作流可用. Gallery toolbar MUST 支持换行或堆叠, Gallery cards MUST 在主列宽度受限时保持内容边界, 选中 asset 的详情 MUST 通过 Inspector drawer, detail rail 或等价 compact detail surface 可达.

#### Scenario: Compact Gallery Toolbar
- **WHEN** 用户在 `960px` 到 `1279px` 宽度查看 Gallery
- **THEN** 搜索, filter, sort, status 和主要操作控件换行或堆叠显示, 且不互相覆盖

#### Scenario: Compact Gallery Detail
- **WHEN** 用户在 compact desktop Gallery 中选择 asset
- **THEN** 桌面应用提供可打开的 detail surface 展示该 asset 的 Inspector 信息, 而不要求常驻右栏占用 Workspace 宽度

#### Scenario: Gallery Card Text Boundaries
- **WHEN** Gallery card 包含长 title, provider, prompt 或 tags
- **THEN** card 内文本通过截断, 换行或内部约束保持在 card 边界内

### Requirement: Compact Albums Layout
桌面应用 SHALL 在 compact desktop 下保持 Albums list-detail workflow 可用. Album detail MUST 作为主操作区域优先展示, album list MAY 折叠为 selector/list panel 或堆叠区域. Create album UI MUST 不覆盖关键 list, header 或 detail 操作.

#### Scenario: Compact Albums Detail Priority
- **WHEN** 用户在 compact desktop Albums workspace 打开 album
- **THEN** album detail 和 album-scoped assets 作为主区域展示, album list 仍可通过 compact list 或 selector 访问

#### Scenario: Compact Album Creation
- **WHEN** 用户在 compact desktop Albums workspace 打开 create album UI
- **THEN** create album UI 以内联 panel, drawer-like panel 或其他 viewport-constrained 形式展示, 不遮挡关键操作

### Requirement: Compact Review Layout
桌面应用 SHALL 在 compact desktop 下保持 Review suggestion list, editable detail form, batch actions 和 field regeneration controls 可用. Review detail MUST 作为主操作区域优先展示, suggestion list MAY 折叠为 selector/list panel 或堆叠区域. Review form MUST 在宽度受限时切换为单列或自适应 form grid.

#### Scenario: Compact Review Detail Priority
- **WHEN** 用户在 compact desktop Review workspace 选择 suggestion
- **THEN** editable detail form 作为主区域展示, suggestion list 仍可通过 compact list 或 selector 访问

#### Scenario: Compact Review Field Actions
- **WHEN** Review 字段 label 包含 regenerate action 且可用宽度受限
- **THEN** regenerate action 保持可点击, 且不得将字段输入挤出容器

### Requirement: Compact Queue Panel Switching
桌面应用 SHALL 在 compact desktop 下为 Queue workspace 提供 `Compose`, `Queue`, `Detail` 本地 panel 切换或等价导航. 宽屏 MAY 同时展示 Batch Composer, Tasks Queue 和 Task Detail. Compact desktop MUST 保证三个 panel 都可达, 且 JSON, log tail 和 output preview 使用内部滚动或 wrapping.

#### Scenario: Compact Queue Panel Access
- **WHEN** 用户在 compact desktop 打开 Queue workspace
- **THEN** 用户可以在 `Compose`, `Queue` 和 `Detail` panel 之间切换, 且当前 panel 占据主工作区

#### Scenario: Compact Queue Task Detail
- **WHEN** 用户在 compact desktop Queue workspace 选择 task
- **THEN** Task Detail panel 可达并展示该 task 的 input, attempts, timeline, outputs 和 log tail

### Requirement: Compact Settings Layout
桌面应用 SHALL 在 compact desktop 下保持 Settings Libraries 和 Logs 子页可用. Libraries registered table MUST 在宽度受限时退化为 row cards 或等价 stacked layout. Logs browser MUST 在宽度受限时堆叠 list 和 preview, preview 使用 bounded height 和内部滚动.

#### Scenario: Compact Libraries Row Cards
- **WHEN** 用户在 compact desktop 打开 `Settings / Libraries`
- **THEN** 每个 registered library 以不溢出的 stacked row card 或等价布局展示 name, path, status 和 actions

#### Scenario: Compact Logs Preview
- **WHEN** 用户在 compact desktop 打开 `Settings / Logs` 并选择 log
- **THEN** log list 和 preview 均可达, preview 内容在 bounded scroll region 内展示

### Requirement: Responsive Visual Verification
桌面 Workbench 响应式变更 MUST 覆盖多视口验证. 验证 SHALL 至少检查 Gallery, Albums, Review, Queue 和 Settings 在 `1440px`, `1180px`, `960px`, `900px` 宽度下无关键控件覆盖, 无不可达主操作, 且长文本不会撑破主布局.

#### Scenario: Multi-Viewport Visual Check
- **WHEN** 开发者验证 responsive Workbench 变更
- **THEN** 开发者检查所有 top-level workspace 在 `1440px`, `1180px`, `960px`, `900px` 宽度下的布局和关键交互可达性

#### Scenario: Long Text Layout Check
- **WHEN** 页面展示长 library path, prompt, schema prompt JSON, checksum, log path 或 task id
- **THEN** 桌面应用通过截断, 换行或内部滚动避免长文本破坏整体布局

### Requirement: Studio Workbench Visual Template
桌面 Workbench SHALL 使用适合本地 AI image asset 工具的 Studio Workbench 视觉模板. UI MAY 重新组织 toolbar, card hierarchy, panel composition 和 responsive behavior, 不必拘泥旧布局. UI MUST 采用中性专业浅色 surface, restrained accent, compact controls, image-first asset browsing, 以及清晰的 workspace/inspector 信息层级. UI MUST NOT 依赖 landing page, glassmorphism, bento marketing, cyberpunk 或高装饰背景作为主要产品风格.

#### Scenario: Gallery Image-First Asset Browsing
- **WHEN** 用户打开 Gallery
- **THEN** Gallery cards 优先展示图片, title 和 compact metadata, 并将 review, selection, tags 和 version 作为 secondary 信息展示

#### Scenario: View-Aware Workspace Command Surface
- **WHEN** 用户切换 Gallery, Albums, Review, Queue 或 Settings
- **THEN** Workspace 顶部展示当前 view 的标题, 状态和主操作层级, 且搜索和过滤控件不遮蔽当前 view 的核心任务

#### Scenario: Consistent Icon Buttons
- **WHEN** 页面展示 icon-only 操作
- **THEN** 操作使用一致的 SVG icon affordance 和 accessible label, 不使用临时文本符号替代图标

#### Scenario: Queue Mid-Width Productivity
- **WHEN** 桌面窗口宽度处于 `1280px` 到 `1439px`
- **THEN** Queue workspace 尽量保留至少 `Queue` 和 `Detail` 两个 panel 并行, 不过早退化为单 panel tabs
