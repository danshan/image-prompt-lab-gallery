## ADDED Requirements

### Requirement: 提供 Sidebar Library Selector

桌面应用 SHALL 在 Sidebar 顶部提供 Library selector, 用于展示当前 Library 名称并切换到其他已注册且未隐藏的 Library. 该 selector MUST 只展示 Library 名称, 并使用向下箭头提示这是下拉控件.

#### Scenario: 切换当前 Library

- **WHEN** 用户从 Sidebar 顶部 Library selector 选择另一个 Library
- **THEN** 桌面应用将该 Library 设为当前上下文, 重新加载 Gallery, 清空当前 Inspector selection, 并展示新 Library 的状态

#### Scenario: 没有已注册 Library

- **WHEN** app-level registry 中没有可见 Library
- **THEN** Sidebar Library selector 展示 empty state, 并提供创建或打开 Library 的入口

### Requirement: 使用应用内标题栏替代系统默认标题栏

桌面应用 SHALL 去掉 Tauri 系统默认标题栏, 并使用应用内标题栏提供窗口标题, 拖拽区域和窗口控制.

#### Scenario: 打开桌面窗口

- **WHEN** 用户打开桌面应用
- **THEN** 窗口中只出现一组最小化, 最大化或关闭窗口控制, 不得同时显示系统标题栏和应用内标题栏两组控制

#### Scenario: 使用窗口控制

- **WHEN** 用户点击应用内标题栏的最小化, 最大化或关闭控制
- **THEN** 桌面应用调用 Tauri window API 执行对应窗口操作

### Requirement: Sidebar 不出现内部滚动

桌面应用 SHALL 保持 Sidebar 在支持的窗口尺寸内不出现内部滚动条, 并通过响应式密度, 区域收缩或低优先级信息隐藏适配高度.

#### Scenario: 窗口高度受限

- **WHEN** 桌面窗口高度接近最小支持高度
- **THEN** Sidebar 不出现内部滚动条, 核心导航和 Library Status 关键字段仍可访问

### Requirement: Library Status 只展示实际 Storage Size

桌面应用 SHALL 在 Sidebar 的 Library Status 中只展示当前 Library 的实际 storage size, 不得展示固定容量上限, meter 或百分比上限.

#### Scenario: 查看 Library Storage

- **WHEN** 用户查看 Sidebar Library Status
- **THEN** Storage 字段展示当前 Library 大小, 例如 `142.7 GB`, 且不展示 `142.7 GB / 500 GB`

### Requirement: Selector 控件视觉统一

桌面应用 SHALL 为所有 selector 控件提供与 app 设计风格一致的视觉样式, 包括 toolbar filter, sort selector, provider selector 和 Sidebar Library selector.

#### Scenario: 查看 Selector 控件

- **WHEN** 用户查看任意 selector 控件
- **THEN** 控件使用一致的高度, 边框, 背景, focus state 和下拉 affordance, 不显示浏览器默认风格

### Requirement: Sidebar 正确展示 Resolution

桌面应用 SHALL 在 Sidebar 或相关 asset summary 中只展示来自 core read model 的真实 resolution. 如果 width 或 height 缺失, UI MUST 展示 unavailable 状态.

#### Scenario: Resolution 可用

- **WHEN** 当前 asset read model 包含 width 和 height
- **THEN** Sidebar 使用 `$width x $height` 格式展示 resolution

#### Scenario: Resolution 缺失

- **WHEN** 当前 asset read model 缺少 width 或 height
- **THEN** Sidebar 展示 unavailable 状态, 不展示错误尺寸或伪造尺寸

### Requirement: Checksum 展示算法标签

桌面应用 SHALL 在 Inspector File section 中展示 checksum algorithm 和 checksum digest.

#### Scenario: 展示 MD5 Checksum

- **WHEN** 当前 file context 的 checksum algorithm 为 `MD5`
- **THEN** Inspector 展示 `Checksum    MD5: $hash`

## MODIFIED Requirements

### Requirement: 提供产品化 Gallery 工作台

桌面应用 SHALL 提供接近设计稿风格的 Gallery 工作台, 包含 app top bar, library sidebar, workspace toolbar, gallery card grid 和 asset Inspector, 并保持高信息密度与稳定布局. app top bar SHALL 提供应用内窗口标题栏和窗口控制, Library 切换 SHALL 位于 Sidebar 顶部, 且应用窗口 SHALL 只显示一组窗口控制.

#### Scenario: 打开 Gallery 主工作流

- **WHEN** 用户打开已注册资源库的 Gallery
- **THEN** 桌面应用展示 app top bar, library sidebar, Gallery 查询工具栏, asset card grid 和当前选中 asset 的 Inspector

#### Scenario: 选择 Gallery 图片

- **WHEN** 用户选择 Gallery 中的某个 asset card
- **THEN** card 显示选中状态且尺寸不发生变化, Inspector 加载该 asset 的详情

#### Scenario: 切换 Library

- **WHEN** 用户通过 Sidebar 顶部 Library selector 切换当前 Library
- **THEN** Workspace 和 Inspector 使用新 Library 的 core read model 重新渲染, 不保留旧 Library 的 selected asset detail
