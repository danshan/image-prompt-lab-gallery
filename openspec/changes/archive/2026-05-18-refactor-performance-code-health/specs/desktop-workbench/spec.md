## ADDED Requirements

### Requirement: Desktop Gallery 图片加载必须惰性且异步解码
桌面应用 SHALL 在 Gallery grid 中避免一次性解码所有 full-resolution 图片, 并为图片元素提供 lazy loading 和 async decoding.

#### Scenario: 渲染 Gallery 图片
- **WHEN** Gallery card 渲染图片元素
- **THEN** 图片元素使用 lazy loading 和 async decoding, 且 card 尺寸在加载前后保持稳定

### Requirement: Desktop Query Refresh 必须防抖
桌面应用 SHALL 对 IPC-backed Gallery query refresh 使用 debounce 或等价机制, 避免用户每输入一个字符就触发完整 Tauri IPC 和数据库查询.

#### Scenario: 输入搜索文本
- **WHEN** 用户连续输入 Gallery 搜索文本
- **THEN** 桌面应用只在输入稳定后刷新 Gallery query, 不得按每个 key stroke 触发后端查询

### Requirement: Desktop Derived State 必须稳定
桌面应用 SHALL 对会传入大型子组件的 derived data 使用 memoized derivation 或等价稳定引用, 包括 provider list, queue count, filtered gallery 和 smart album preview.

#### Scenario: 非相关状态变化
- **WHEN** 用户修改与 Gallery 过滤无关的局部状态
- **THEN** provider list, queue count 和 filtered gallery 等 derived values 不应产生不必要的新引用导致大型子组件重渲染

### Requirement: Desktop Refresh Actions 避免无必要 Waterfall
桌面应用 SHALL 将语义独立的 refresh 操作并发执行, 不得无原因串行调用多个 IPC refresh.

#### Scenario: 接受 Metadata Suggestion
- **WHEN** 用户接受一条 metadata suggestion
- **THEN** Gallery refresh 和 suggestions refresh 可并发执行; 只有依赖当前 selection 的 detail refresh 需要条件化执行

### Requirement: Desktop Polling 必须可清理
桌面应用 SHALL 跟踪 polling 或 delayed wait 的 timeout handle, 并在组件 unmount, library switch 或任务结束时清理.

#### Scenario: 切换 Library 时存在 Polling
- **WHEN** 用户切换 library 且旧 library 存在任务轮询 timeout
- **THEN** 桌面应用清理旧 timeout, 不得在旧请求完成后更新新 library state

## MODIFIED Requirements

### Requirement: 提供三栏桌面工作台
桌面应用 SHALL 提供 `Library Sidebar | Workspace | Inspector` 三栏工作台, 并允许 Inspector 在窄窗口中折叠. Workbench implementation MUST 按 workflow 组件和 data hooks 划分职责, 避免单个入口组件长期承载 Gallery, Albums, Review, Queue, Settings, Inspector 和 IPC orchestration 的全部逻辑.

#### Scenario: 选择 Gallery 图片
- **WHEN** 用户在 Gallery 中选择一个 asset
- **THEN** Workspace 保持图片网格上下文, Inspector 展示该 asset 的 metadata, prompt, tags, albums 和 versions

#### Scenario: Workflow 边界清晰
- **WHEN** 开发者维护 desktop workbench
- **THEN** Gallery, Albums, Review, Task, Settings 和 Inspector 的主要 rendering 与 data orchestration 位于职责明确的组件或 hooks 中
