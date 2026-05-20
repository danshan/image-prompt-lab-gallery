## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台

桌面应用 SHALL 提供 Studio Console shell, 在 normal desktop 下表达为 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip`, 在 compact desktop 下允许 Library Context 和 Inspector 折叠或转换为 drawer/rail, 并让 Workspace 成为稳定主工作区. Workbench implementation MUST 按 shell, workflow, data hooks, controller hooks 和 pure state helpers 划分职责, 避免单个入口组件长期承载 Gallery, Albums, Review, Queue, Settings, Inspector 和 IPC orchestration 的全部逻辑. Desktop root component MUST primarily compose controllers and screens, while workflow screen components MUST live under focused workflow-owned modules rather than a single cross-workflow mega file. 桌面应用 MUST 将 `960px` 作为一等 compact desktop 最小宽度目标, 在该宽度下保证导航, 当前 workflow 主操作, 详情和错误恢复操作均可达.

#### Scenario: Studio Console Shell
- **WHEN** 用户在 normal desktop 打开桌面应用
- **THEN** 应用展示 Studio Rail, Library Context, Workspace, Inspector 和 Activity Strip, 且每个区域职责清晰

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** shell components, Gallery, Albums, Review, Queue, Settings, Inspector, data hooks, controller hooks 和 pure state helpers 位于职责明确的模块中

#### Scenario: Root Component 保持组合职责
- **WHEN** 开发者修改 `apps/desktop/src/app/App.tsx`
- **THEN** root component 主要负责组合 shell, workflow controllers 和 screen components
- **AND** library, gallery selection, generation composer, task queue, review, settings, logs 或 update 的 async orchestration 不应继续集中堆叠在 root component 中

#### Scenario: Workflow Screens 按 Ownership 拆分
- **WHEN** 开发者维护 Gallery, Albums, Review, Queue, Settings 或 Inspector screen
- **THEN** screen implementation 位于对应 workflow-owned module 中
- **AND** cross-workflow export file 不得成为多个大型 screen component 的主要实现位置

#### Scenario: Compact Desktop Shell
- **WHEN** 桌面窗口宽度处于 compact desktop 范围
- **THEN** Workspace 保持为主工作区, Library Context 和 Inspector 可折叠或以 drawer/rail 形式访问, Activity Strip 仍可达

#### Scenario: Below First-Class Minimum
- **WHEN** 桌面窗口宽度小于 `960px`
- **THEN** 桌面应用允许页面退化为单列或内部滚动, 但不得让主要导航, 当前页面主操作或错误恢复操作永久不可达
