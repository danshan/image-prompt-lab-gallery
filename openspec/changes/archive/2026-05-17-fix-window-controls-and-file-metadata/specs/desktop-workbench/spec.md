## MODIFIED Requirements

### Requirement: 使用系统原生标题栏

桌面应用 SHALL 使用系统原生标题栏和窗口控制. 桌面应用 MUST NOT 同时渲染应用内自定义窗口控制和系统窗口控制.

#### Scenario: 打开桌面窗口

- **WHEN** 用户打开桌面应用
- **THEN** 窗口使用系统原生标题栏, 且只出现一组最小化, 最大化或关闭窗口控制

#### Scenario: 使用系统窗口行为

- **WHEN** 用户使用系统标题栏拖动窗口, 或点击系统最小化, 最大化或关闭控制
- **THEN** 宿主系统执行对应窗口操作

### Requirement: Sidebar 不展示 Asset Resolution

桌面应用 SHALL 避免在 Sidebar, Library Status 或 Gallery asset summary 中展示当前 asset 的文件 resolution. 文件级 resolution 信息 MUST 由 Inspector File section 承载.

#### Scenario: 查看左侧 Sidebar

- **WHEN** 用户在 Gallery 中选择包含 width 和 height 的 asset
- **THEN** Sidebar 和 Library Status 不展示该 asset 的 `$width x $height` resolution

#### Scenario: 查看 Gallery asset summary

- **WHEN** Gallery card 或左侧 summary 渲染 asset 摘要信息
- **THEN** 摘要信息不展示文件 resolution, 但仍可展示 title, provider, rating, tags 和 version summary

### Requirement: Inspector File Section 展示 Resolution 和 Aspect Ratio

桌面应用 SHALL 在 Inspector File section 中展示文件 resolution 和 aspect ratio. aspect ratio MUST 只在 width 和 height 都可用且大于 0 时由真实尺寸计算, 不得使用伪造或默认比例.

#### Scenario: Resolution 和 Aspect Ratio 可用

- **WHEN** 当前 file context 包含有效 width 和 height
- **THEN** Inspector File section 展示 `$width x $height` resolution 和约分后的 aspect ratio

#### Scenario: Aspect Ratio 不可用

- **WHEN** 当前 file context 缺少 width 或 height, 或 width 或 height 不是正数
- **THEN** Inspector File section 不展示伪造 aspect ratio, 并展示 unavailable 状态或隐藏该字段

### Requirement: Inspector 支持编辑 Title 和星级 Rating

桌面应用 SHALL 允许用户在 Inspector 中双击当前 title 后编辑 canonical title. 桌面应用 SHALL 使用星级控件展示和修改 rating, 不得使用 `*` 文本串作为主要 rating 表达. Rating card SHALL 显示在 Prompt card 之下.

#### Scenario: 编辑 Title

- **WHEN** 用户在 Inspector 中双击 title, 输入新标题并确认
- **THEN** 桌面应用通过 Rust core 更新 canonical title, 并刷新 Gallery card 和 Inspector title

#### Scenario: 修改 Rating

- **WHEN** 用户在 Inspector 中点击某个星级
- **THEN** 桌面应用通过 Rust core 更新 rating, 并使用星级显示更新后的评分

#### Scenario: 查看 Rating Card 位置

- **WHEN** 用户查看选中 asset 的 Inspector
- **THEN** Rating card 显示在 Prompt card 之下
