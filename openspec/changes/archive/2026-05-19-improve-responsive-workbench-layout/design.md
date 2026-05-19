## Context

桌面端当前主要布局集中在 `apps/desktop/src/main.tsx` 和 `apps/desktop/src/styles.css`. 顶层 `.workbench` 使用固定三栏, 页面内部又有多处固定 grid track, 例如 Gallery toolbar, Albums/Review 双栏, Queue 三栏, Settings library table 和 Logs browser. 这些固定宽度在宽屏下清晰, 但在 compact desktop 窗口下会放大错位, 覆盖和不可达问题.

本变更已经通过 `docs/superpowers/specs/2026-05-18-responsive-workbench-layout-design.md` 对齐设计方向: 以 `960px` 为一等 compact desktop 目标, 采用 shell-first collapse system, 全页面同等处理.

## Goals / Non-Goals

**Goals:**

- 建立统一的 Workbench responsive shell, 让 Workspace 在宽度受限时成为主工作区.
- 允许 Sidebar 和 Inspector 在 compact desktop 下折叠, 并保持导航与详情可达.
- 为 Gallery, Albums, Review, Queue 和 Settings 定义共享响应式布局模式.
- 收敛 CSS breakpoint 和 layout token, 降低后续页面继续堆 ad hoc media query 的概率.
- 保持现有 Rust core, Tauri command, DTO 和业务数据流不变.

**Non-Goals:**

- 不做 phone-first 移动端重设计.
- 不重做视觉品牌和配色系统.
- 不修改 SQLite schema, provider, generation task 语义或 metadata review 业务逻辑.
- 不借此机会大规模拆分所有 data hooks 或 IPC orchestration.

## Decisions

### 0. 采用 Studio Workbench 视觉模板

产品定位是本地优先的 AI image prompt, asset, metadata 和 generation queue 工作台, 不是 marketing website 或重视觉创意站点. UI 应采用 `Minimal Swiss + Data-Dense Dashboard + Image-First Workbench` 的组合: neutral light surface, restrained teal/blue accent, Inter typography, compact controls, 明确的工作区边界, 图片内容作为主要视觉资产.

本轮不引入 glassmorphism, bento landing, cyberpunk, oversized hero 或高装饰背景. 这些风格会削弱 metadata review, queue monitoring 和 library management 的长期可维护性.

用户已明确允许重新设计页面, 不需要拘泥当前布局. 因此实施可以调整 toolbar, card hierarchy, panel composition 和 responsive behavior, 但仍不得改变 Rust core, Tauri commands, persistence schema 或 provider semantics. 信息架构可以继续保留 `Library / Workspace / Inspector`, 因为它匹配当前产品工作流, 但视觉表现不再受旧 CSS 约束.

### 1. 采用 shell-first collapse system

在 `1280px` 及以上保留完整三栏. 在 `960px` 到 `1279px` 进入 compact desktop: Sidebar 退化为可展开的窄 rail, Inspector 退化为可打开的 detail drawer 或 rail, Workspace 保持唯一稳定主列.

替代方案是逐页面修补 breakpoint. 该方案短期局部改动更小, 但会让 Gallery, Albums, Review, Queue 和 Settings 的响应式行为继续分叉. 本轮选择 shell-first, 因为问题是系统性布局约束失效, 不是单个页面的 CSS bug.

### 2. Inspector 在 compact desktop 下作为 contextual detail surface

Gallery 选择 asset 后, detail 必须仍然可达, 但不再要求右栏常驻. Inspector drawer 由选中内容驱动, 可关闭, 也可在选择 asset 后打开或更新.

替代方案是保留右栏并压缩 Sidebar. 这会让 960px 下 Workspace 仍然过窄, 与用户确认的 "Workspace 优先" 冲突.

### 3. Albums 和 Review 复用 split-workspace 模式

Albums 的 `album list | album detail` 与 Review 的 `suggestion list | detail form` 都是 list-detail 工作流. 实现应提取 shared class 或轻量 layout component, 宽屏双栏, compact desktop 下 detail 优先, list 作为上方 selector/list panel 或可折叠 panel.

替代方案是给两个页面分别写 media query. 这会更贴近单页局部 DOM, 但长期会重复同类布局规则.

### 4. Queue compact desktop 使用本地 panel 切换

Queue 同时包含 `Batch Composer`, `Tasks Queue`, `Task Detail` 三块重内容. 在 compact desktop 下直接纵向堆叠会形成过长页面, 操作焦点不清晰. 因此 Queue 使用本地 segmented control 或 tabs 切换 `Compose`, `Queue`, `Detail`.

宽屏仍可保留三栏并行. 这避免为了 compact desktop 牺牲宽屏效率.

在 `1280px` 到 `1439px` 的中屏桌面下, Queue 不应过早降级为单 panel. 该宽度仍有足够空间保留 `Queue | Detail` 并行, 同时将 Composer 作为可切换 panel. 只有在 compact desktop 下才强制单 panel 切换.

### 5. Settings table-to-card, Logs bounded preview

Settings Libraries 在 compact desktop 下不继续保持 fixed grid table, 改为 row cards: name, path, status, actions 分区垂直组织. Logs browser 在 compact desktop 下 list 和 preview 堆叠, preview 使用 bounded height 和内部滚动.

这个选择优先保证长 path, log path 和 log content 不撑破整体页面.

### 6. CSS token 化和长文本策略先行

实现应集中定义 compact breakpoint, wide breakpoint, sidebar widths, inspector widths, panel minimums 和 toolbar control minimums. 所有可变长内容都必须明确 `min-width: 0`, wrapping, truncation 或 internal scroll.

替代方案是只改出问题的 selector. 这可以更快修补当前截图, 但无法防止后续新增字段再次破坏布局.

### 7. Gallery card 改为 image-first asset tile

Gallery 的主要任务是浏览和选择图片资产, 不是在每张卡片里完成全部 review 和 metadata 操作. Card 默认应优先展示 thumbnail, title 和一行 compact metadata. Review pending, tags, version 和 selection 可以保留, 但必须降低视觉权重并避免把 tile 拉成长表单.

Review 动作保留为低噪声 secondary action, 选中后完整上下文进入 Inspector. 这样可以提高主网格密度, 也符合图片资产管理工具的使用预期.

### 8. Toolbar 从全局过滤条收敛为 view-aware command surface

当前 `WorkspaceToolbar` 服务所有 view, 但 Gallery, Albums, Review, Queue 和 Settings 的主操作不同. 本轮不拆分完整 toolbar 数据流, 但要在视觉层面提供 view-aware title/status, 将全局 search/filter 的层级降低, 并让 Generate 等主动作保持清晰.

后续如果继续重构, 可以把每个 workspace 的 command surface 下沉到对应 view 组件中.

### 9. 统一 icon affordance

所有纯图形按钮使用一致的 inline SVG icon component, 避免 `#`, `=`, `X`, `DB`, `+` 这类临时文本符号出现在 UI 中. 图标只承担常见 affordance: grid, list, close, plus, database, panel, menu. 不引入新的 icon library 依赖, 以保持当前 React/Vite footprint 不变.

## Risks / Trade-offs

- [Risk] Shell state 增加后可能引入交互状态复杂度. → Mitigation: 只新增 layout-only state, 不接管业务 query, selected asset, review form 或 task state.
- [Risk] Compact Inspector drawer 可能改变用户查看详情的肌肉记忆. → Mitigation: 宽屏仍保留常驻 Inspector, compact 下提供明确的打开/关闭入口和选中后自动更新.
- [Risk] Queue panel 切换可能隐藏部分上下文. → Mitigation: 宽屏保持三栏, compact 下 panel selector 始终可见, 选中 task 后 Detail panel 可达.
- [Risk] Shared split-workspace 抽象过早泛化. → Mitigation: 先以 class pattern 或小型 layout-only component 实现, 仅覆盖 Albums 和 Review 当前共同结构.
- [Risk] 视觉验证依赖人工判断. → Mitigation: 至少覆盖 `1440px`, `1180px`, `960px`, `900px` 四档截图, 并明确检查长文本和关键操作可达性.

## Migration Plan

1. 先实现 shell responsive token 和 compact class behavior.
2. 再按页面处理 Gallery, Albums, Review, Queue, Settings 的局部布局.
3. 保持现有业务 command 和 state shape 不变.
4. 运行 desktop build/test.
5. 启动桌面前端, 用浏览器或 Tauri preview 在多视口下做视觉验证.

回滚策略是回退本变更的前端组件和 CSS 修改. 因为本变更不改持久化数据和后端 schema, 回滚不需要迁移数据.

## Open Questions

无阻塞问题. 实施时如果发现某个页面 DOM 结构无法承载设计, 应先更新本 change 的 artifacts, 再继续扩大实现.
