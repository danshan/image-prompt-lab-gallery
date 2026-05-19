## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供 Studio Console shell, 在 normal desktop 下表达为 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip`, 在 compact desktop 下允许 Library Context 和 Inspector 折叠或转换为 drawer/rail, 并让 Workspace 成为稳定主工作区. Workbench implementation MUST 按 shell, workflow, data hooks 和 pure state helpers 划分职责, 避免单个入口组件长期承载 Gallery, Albums, Review, Queue, Settings, Inspector 和 IPC orchestration 的全部逻辑. 桌面应用 MUST 将 `960px` 作为一等 compact desktop 最小宽度目标, 在该宽度下保证导航, 当前 workflow 主操作, 详情和错误恢复操作均可达.

#### Scenario: Studio Console Shell
- **WHEN** 用户在 normal desktop 打开桌面应用
- **THEN** 应用展示 Studio Rail, Library Context, Workspace, Inspector 和 Activity Strip, 且每个区域职责清晰

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** shell components, Gallery, Albums, Review, Queue, Settings, Inspector, data hooks 和 pure state helpers 位于职责明确的模块中

#### Scenario: Compact Desktop Shell
- **WHEN** 桌面窗口宽度处于 compact desktop 范围
- **THEN** Workspace 保持为主工作区, Library Context 和 Inspector 可折叠或以 drawer/rail 形式访问, Activity Strip 仍可达

#### Scenario: Below First-Class Minimum
- **WHEN** 桌面窗口宽度小于 `960px`
- **THEN** 桌面应用允许页面退化为单列或内部滚动, 但不得让主要导航, 当前页面主操作或错误恢复操作永久不可达

### Requirement: 提供 Gallery 和 Albums 视图

桌面应用 SHALL 提供 Gallery asset board 和 Albums collection management workspace. Gallery SHALL 以 image-first asset board 展示 assets. Albums SHALL 支持 manual album 和 smart album 管理, 包含 album list ordering, album detail, manual item ordering, batch add, remove asset, rename/delete 和 smart rule preview.

#### Scenario: 打开 Gallery Asset Board
- **WHEN** 用户打开 Gallery
- **THEN** Workspace 展示 image-first asset board, 每个 asset item 包含 image preview, title, current version, version count, review state, provider/model, task origin 和 album context

#### Scenario: 打开 Albums 管理视图
- **WHEN** 用户打开 Albums 视图
- **THEN** Workspace 展示 collection list, album detail, manual/smart album 状态和 album-scoped asset board

### Requirement: 提供 Review Inbox

桌面应用 SHALL 提供 staged metadata Review workspace, 用于处理 pending metadata suggestions, local review draft, generated field results, suggestion history 和 canonical metadata accept. Review workspace MUST 明确区分 pending suggestion, local draft, generated result 和 canonical metadata.

#### Scenario: 打开 Review Workspace
- **WHEN** 用户打开 Review
- **THEN** Workspace 展示 pending suggestion list, selected suggestion draft detail, confidence, history, field regeneration 状态和 accept/reject actions

#### Scenario: Review 不混淆 Canonical Metadata
- **WHEN** Review 展示 pending suggestion 或 generated field result
- **THEN** UI 明确其为 staged 或 generated 状态, 不将其展示为已确认 canonical metadata

### Requirement: 提供 Generation Composer 和 Queue

桌面应用 SHALL 将 Queue 作为 operations console, 包含 batch composer, tasks queue 和 task detail. Queue MUST 展示 task attempts, timeline, log tail, outputs, asset/version/review links, 并根据 task status 提供合适操作.

#### Scenario: 打开 Queue Operations Console
- **WHEN** 用户打开 Queue
- **THEN** Workspace 展示 batch composer, task queue 和 task detail, 或在 compact desktop 下提供等价 panel switching

#### Scenario: 查看 Task Detail Cross Links
- **WHEN** 用户选择一个 task
- **THEN** Task Detail 展示 attempts, timeline, log tail, outputs, Open asset, Open version 和 Open review suggestion links, 如果这些 links 存在

## ADDED Requirements

### Requirement: Studio Console Visual System

桌面 Workbench SHALL 使用 Studio Console visual system. UI MUST 使用 tokenized colors, spacing, radius, status colors, focus styles 和 compact control dimensions. 主要 palette SHALL 包含 graphite chrome, warm ivory canvas, cobalt secondary action, vermilion primary generation action, lime/amber/red/green status colors 或等价语义 tokens. UI MUST 使用一致 SVG icon affordance 和 accessible labels, 不得用临时文本符号替代常见图标.

#### Scenario: Tokenized Visual System
- **WHEN** 开发者维护 desktop visual style
- **THEN** 主要颜色, status, action, surface, border, focus 和 spacing 通过共享 tokens 表达

#### Scenario: Consistent Icon Buttons
- **WHEN** 页面展示 icon-only 操作
- **THEN** 操作使用一致 SVG icon affordance 和 accessible label, 不使用 `DB`, `#`, `=`, `+`, `X` 等临时文本符号作为图标

### Requirement: Studio Console Read Models

桌面应用 SHALL 通过稳定屏幕级 read models 加载 Studio Console 数据. React MUST NOT 通过多个无关低级 command 临时拼装 cross-workflow semantics. Read models MAY 由 Rust core 或 Tauri service boundary 组合, 但 payload MUST 有稳定字段和测试覆盖.

#### Scenario: 加载 Studio Overview
- **WHEN** 桌面应用加载 Studio Console shell
- **THEN** 应用通过 read model 获取当前 library summary, storage/integrity, review pending count, active task summary 和 provider health summary

#### Scenario: 加载 Asset Board Item
- **WHEN** Gallery 或 album-scoped board 请求 asset items
- **THEN** read model 返回 image path, title, current version, version count, review state, provider/model, task origin, album context, rating 和 tags

#### Scenario: 加载 Review Draft Detail
- **WHEN** 用户打开 Review suggestion detail
- **THEN** read model 返回 suggestion, draft seed, confidence, history, generated field results, related tasks 和 asset context

#### Scenario: 加载 Task Detail Links
- **WHEN** 用户打开 Task Detail
- **THEN** read model 返回 attempts, timeline, log tail, outputs, asset links, version links 和 review suggestion links

### Requirement: Cross-Workflow Deep Links

桌面应用 SHALL 支持 Gallery, Inspector, Review, Queue, Albums 和 Settings 之间的 cross-workflow deep links. Links MUST 使用稳定 target payload, 不依赖 UI 猜测 id 类型.

#### Scenario: 从 Task 打开生成结果
- **WHEN** Task Detail 包含 asset 或 version output link
- **THEN** 用户可以打开对应 asset, Gallery/Inspector 展示该 asset 或 version context

#### Scenario: 从 Task 打开 Review Suggestion
- **WHEN** Task Detail 包含 review suggestion output link
- **THEN** 用户可以跳转到 Review workspace 并打开对应 suggestion

#### Scenario: 从 Review 打开 Source Task
- **WHEN** Review suggestion 关联 source task
- **THEN** 用户可以从 Review 打开该 task detail

#### Scenario: 从 Inspector 打开 Pending Review
- **WHEN** 当前 asset 存在 pending metadata suggestion
- **THEN** Inspector 提供打开对应 Review suggestion 的入口

### Requirement: Workflow State Coverage

Gallery, Albums, Review, Queue, Settings 和 Inspector SHALL 分别定义 normal, loading, empty, error 和 recovery states. 可恢复错误 MUST 保留用户可继续工作的上下文, 除非当前 library 已关闭或切换.

#### Scenario: Workflow Loading State
- **WHEN** 任一 workflow 正在加载必要 read model
- **THEN** UI 展示明确 loading state, 且不误显示旧数据为当前数据

#### Scenario: Workflow Empty State
- **WHEN** 任一 workflow 没有可展示实体
- **THEN** UI 展示与当前 workflow 相关的 empty state 和可用下一步

#### Scenario: Workflow Recoverable Error
- **WHEN** 任一 workflow 发生可恢复错误
- **THEN** UI 展示错误和 recovery action, 并尽量保留当前 selection, draft 或 query context

### Requirement: Studio Console Visual Verification

Studio Console 变更 MUST 覆盖多视口视觉验证. 验证 SHALL 至少检查 Gallery, Albums, Review, Queue, Settings 和 Inspector 在 normal desktop 和 `960px` compact desktop 下无关键控件覆盖, 无不可达主操作, 无文本重叠, 无不可读对比度, 且长文本不会撑破主布局.

#### Scenario: Normal Desktop Visual Check
- **WHEN** 开发者验证 Studio Console 变更
- **THEN** 开发者检查 normal desktop 下所有 top-level workflows 的布局和关键交互可达性

#### Scenario: Compact Desktop Visual Check
- **WHEN** 开发者验证 Studio Console 变更
- **THEN** 开发者检查 `960px` compact desktop 下所有 top-level workflows 的布局和关键交互可达性

#### Scenario: Long Text Layout Check
- **WHEN** 页面展示长 library path, prompt, schema prompt JSON, checksum, log path 或 task id
- **THEN** 桌面应用通过截断, 换行或内部滚动避免长文本破坏整体布局
