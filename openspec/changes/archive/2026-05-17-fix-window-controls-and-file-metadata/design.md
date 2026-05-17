## Context

当前桌面应用已经使用应用内标题栏替代系统默认标题栏, 并在 `AppTopBar` 中渲染关闭, 最小化和最大化按钮. 但实际窗口无法拖动, 窗口控制按钮也不可用. 由于本应用是生产工具, 窗口行为应优先遵循 macOS 和宿主系统标准, 不继续维护自定义 chrome.

Gallery 与 Inspector 已通过 core read model 获得 `width` 和 `height`. 当前左侧 Library Status 使用选中 asset 的尺寸展示 resolution, 这让侧边栏承担了过细的文件详情. 右侧 Inspector 的 File section 已显示尺寸, 更适合继续承载长宽比等文件级 metadata. Rating section 当前位于 Inspector 较靠后位置, 需要移动到 Prompt section 之后, 让用户先看到生成语义, 再处理主观评分.

本变更只调整桌面工作台的交互与展示层. Rust core 的数据模型, 资源库写路径和 Tauri command DTO 不应变化.

## Goals / Non-Goals

**Goals:**

- 恢复系统原生标题栏和窗口控制.
- 移除应用内自定义标题栏, 避免重复窗口控制.
- 保持窗口中只出现一组窗口控制.
- 从左侧 Sidebar 或 Gallery summary 中移除 resolution 展示.
- 在 Inspector File section 中展示尺寸和长宽比, 且只基于真实 `width` 与 `height` 计算.
- 将 Inspector Rating card 移动到 Prompt card 之下.
- 保持现有 core read model 和 Tauri DTO 边界不变.

**Non-Goals:**

- 不重新设计整套 workbench 信息架构.
- 不新增文件 metadata 字段到 SQLite 或 Rust DTO.
- 不实现图片裁剪, 编辑或 graph lineage.
- 不引入新的前端依赖.
- 不改变 rating 的写入语义和 core service 边界.

## Decisions

### 1. 窗口 chrome 使用系统原生 decoration

Tauri window 配置恢复系统 `decorations`, React 不再渲染 `AppTopBar` 和自定义红黄绿按钮. 窗口拖动, 最小化, 最大化或还原, 关闭都交给宿主系统处理.

备选方案是继续维护 frameless custom chrome, 使用 `data-tauri-drag-region`, `app-region: drag` 和 Tauri window API. 该方案视觉上可控, 但需要额外权限, hit target 处理和平台差异验证. 对当前生产工具而言收益不足.

### 2. Workbench 布局从内容区首行开始

移除应用内 topbar 后, `workbench` grid 不再保留 48px 的标题栏行. Sidebar, Workspace 和 Inspector 直接占用应用内容区高度. 这比保留空 topbar 占位更清晰, 也避免内容区出现无意义留白.

### 3. 长宽比在前端展示层派生

长宽比由 Inspector 的 `file.width` 和 `file.height` 计算并格式化, 不新增 DTO 字段. 这是派生展示值, 不需要持久化. 计算规则应只在两个值都为正数时生效, 否则返回 unavailable.

格式建议使用约分后的整数比, 例如 `1024 x 1024` 展示 `1:1`, `1792 x 1024` 展示 `7:4`, `1024 x 1536` 展示 `2:3`. 约分比单纯小数更适合图像比例识别, 也避免引入精度和四舍五入争议.

### 4. 左侧移除 resolution, Inspector 保留文件详情

左侧 Sidebar 或 Gallery summary 不再展示选中 asset 的 resolution. 左侧只保留 Library 级状态, 导航和必要的上下文. 文件尺寸, checksum, MIME type, 相对路径和长宽比都放在 Inspector File section.

备选方案是在 Gallery card 中保留 resolution. 这会提高列表密度但降低扫视效率, 也与当前用户要求冲突. 因此本次移除左侧 resolution, 不扩大到更多文件字段.

### 5. Inspector section 顺序局部调整

Inspector 中 Rating card 移动到 Prompt card 之后. 其他 section 顺序保持稳定, 避免这次修复变成大范围信息架构重排. Rating 仍调用现有 rating update flow, 成功后刷新 Gallery 和 Inspector.

## Risks / Trade-offs

- [Risk] 恢复系统 decoration 后首屏可用垂直空间减少或与现有视觉稿略有差异. → Mitigation: 移除应用内 topbar 行, 让内容区从系统标题栏下方直接开始.
- [Risk] 不同平台的系统标题栏外观不完全一致. → Mitigation: 接受平台标准行为, 不用自定义 chrome 追求跨平台完全一致.
- [Risk] 长宽比约分对异常尺寸产生无意义结果. → Mitigation: 只有 `width > 0` 且 `height > 0` 时展示比例, 其他情况展示 unavailable.
- [Risk] 移除左侧 resolution 后用户少一个快速判断尺寸的位置. → Mitigation: Inspector File section 保留尺寸并新增长宽比, 文件级详情集中在选中后的详情区.
- [Risk] Rating section 移动可能影响现有 CSS 间距. → Mitigation: 只调整 JSX section 顺序, 复用现有 `InspectorSection` 样式.
