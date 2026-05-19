## Purpose

Define the desktop workbench, gallery, inspector, generation queue, and core read model integration.
## Requirements
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

### Requirement: Settings 提供 Libraries 和 Logs 子页

桌面应用 SHALL 在 Settings workspace 中提供 `Libraries` 和 `Logs` 两个子页. Settings 默认打开 `Libraries` 子页. 切换 Settings 子页 MUST NOT 改变当前 resource library.

#### Scenario: 打开 Settings 默认进入 Libraries

- **WHEN** 用户打开 Settings
- **THEN** 桌面应用展示 `Libraries` 子页, 并提供切换到 `Logs` 子页的控件

#### Scenario: 切换 Settings 子页不改变当前 Library

- **WHEN** 用户在 `Settings / Libraries` 和 `Settings / Logs` 之间切换
- **THEN** 当前 resource library, Gallery query 和 Inspector selection 不因子页切换而改变

### Requirement: Settings Libraries 维护多个资源库

桌面应用 SHALL 在 `Settings / Libraries` 中展示 registered libraries 管理表, 并提供创建资源库, 打开已有资源库文件夹, 导入备份 zip, 切换当前资源库, 重命名本机 alias, 取消注册, 导出备份 zip 和 Reveal in Finder 操作.

#### Scenario: 查看 Registered Libraries 管理表

- **WHEN** 用户打开 `Settings / Libraries`
- **THEN** 桌面应用展示 registered libraries 列表, 每行包含 name, path 和 actions, 并将 missing on disk 作为行状态标记

#### Scenario: 创建资源库后设为当前资源库

- **WHEN** 用户在 `Settings / Libraries` 中输入 library name 和 folder name, 选择父目录并创建新资源库成功
- **THEN** 桌面应用刷新 registered libraries, 将新资源库设为当前资源库, 并清空旧资源库相关的 Workspace 和 Inspector 上下文

#### Scenario: 打开已有资源库文件夹

- **WHEN** 用户选择 `Open Existing Library` 并选择有效资源库目录
- **THEN** 桌面应用调用 Rust core 校验并注册该资源库, 刷新 registered libraries, 并将其设为当前资源库

#### Scenario: 重命名当前资源库 Alias

- **WHEN** 用户在 `Settings / Libraries` 中重命名当前资源库 alias
- **THEN** 桌面应用刷新 registered libraries, 并在 Sidebar Library selector 和 Settings row 中展示新 alias

#### Scenario: 取消注册当前资源库

- **WHEN** 用户 Close 当前资源库并确认
- **THEN** 桌面应用调用 Rust core 取消注册该资源库, 清空当前 library, Gallery, Inspector detail, Albums, Review, Queue 和 selected ids, 并进入 no-library state

#### Scenario: 取消注册非当前资源库

- **WHEN** 用户 Close 一个非当前资源库并确认
- **THEN** 桌面应用刷新 registered libraries, 且当前 library 和 Workspace 上下文保持不变

#### Scenario: 导出资源库备份 Zip

- **WHEN** 用户对一个有效资源库执行 Export Zip 并选择保存路径
- **THEN** 桌面应用调用 Rust core 导出完整资源库备份 zip, 并在操作期间阻止重复提交

#### Scenario: 导入资源库备份 Zip

- **WHEN** 用户选择有效备份 zip 和目标目录
- **THEN** 桌面应用调用 Rust core 导入并注册该资源库, 刷新 registered libraries, 并默认切换到导入后的资源库

#### Scenario: Reveal in Finder

- **WHEN** 用户对一个路径存在的资源库执行 Reveal in Finder
- **THEN** 桌面应用通过操作系统文件管理器打开该资源库 root folder, 且不改变当前资源库状态

#### Scenario: Missing Path 行为

- **WHEN** registered library 的 root path 不存在
- **THEN** `Settings / Libraries` 将该行标记为 missing on disk, 保持 Close 可用, 并禁用或拒绝 Export Zip 和 Reveal in Finder

### Requirement: 提供 Gallery 和 Albums 视图

桌面应用 SHALL 提供 Gallery asset board 和 Albums collection management workspace. Gallery SHALL 以 image-first asset board 展示 assets. Albums SHALL 支持 manual album 和 smart album 管理, 包含 album list ordering, album detail, manual item ordering, batch add, remove asset, rename/delete 和 smart rule preview.

#### Scenario: 打开 Gallery Asset Board
- **WHEN** 用户打开 Gallery
- **THEN** Workspace 展示 image-first asset board, 每个 asset item 包含 image preview, title, current version, version count, review state, provider/model, task origin 和 album context

#### Scenario: 打开 Albums 管理视图
- **WHEN** 用户打开 Albums 视图
- **THEN** Workspace 展示 collection list, album detail, manual/smart album 状态和 album-scoped asset board

### Requirement: Albums Workspace 使用真实 Album 数据

桌面应用 SHALL 提供 Albums workspace, 展示真实 albums 列表, 支持创建 manual album, 打开 album detail, 并复用 Gallery card 展示 album 内容. Albums workspace SHALL 支持 album list drag reorder, rename, delete, manual album item drag reorder, remove asset, batch add selected assets, and Smart Album builder.

#### Scenario: 查看 Albums Workspace

- **WHEN** 用户打开 Albums workspace
- **THEN** 桌面应用展示当前 library 的 album list, 每项包含 name, kind 和 item count

#### Scenario: 创建 Manual Album

- **WHEN** 用户在 Albums workspace 输入 name 并创建 manual album
- **THEN** 桌面应用通过 Rust core 创建 album, 刷新 album list, 并展示新 album

#### Scenario: 打开 Album Detail

- **WHEN** 用户点击一个 manual album
- **THEN** Workspace 展示该 album 的 header 和 album-scoped Gallery cards

#### Scenario: 拖拽排序 Album List

- **WHEN** 用户在 Albums workspace 拖拽调整 album list 顺序
- **THEN** 桌面应用调用 Rust core 保存 album sort order, 并按保存后的顺序渲染 album list

#### Scenario: 重命名 Album

- **WHEN** 用户在 Albums workspace 重命名 album
- **THEN** 桌面应用调用 Rust core 更新 album name, 刷新 album list 和当前 album detail header

#### Scenario: 删除 Album

- **WHEN** 用户在 Albums workspace 删除 album
- **THEN** 桌面应用调用 Rust core 删除 album, 清理当前 selected album, 并刷新 album list 和 Gallery query

#### Scenario: 拖拽排序 Manual Album Assets

- **WHEN** 用户在 manual album detail 中拖拽 asset cards
- **THEN** 桌面应用调用 Rust core 保存 album item sort order, 并按 album order 重新渲染该 album

#### Scenario: 从 Manual Album 移除 Asset

- **WHEN** 用户在 manual album detail 中移除某个 asset
- **THEN** 桌面应用调用 Rust core 删除 membership, 刷新 album detail 和 album list item count

#### Scenario: 批量添加 Gallery Assets 到 Manual Album

- **WHEN** 用户选择多个 Gallery assets 并添加到 manual album
- **THEN** 桌面应用调用 Rust core 批量写入 memberships, 刷新 album list, Gallery 和受影响 asset detail

#### Scenario: 构建 Smart Album

- **WHEN** 用户在 Smart Album builder 中设置 text, tags, providers, min rating, review status, category, status, created date range 或 sort
- **THEN** 桌面应用将 typed query 提交给 Rust core validation, 并展示满足 query 的 live preview

### Requirement: Inspector 支持 Album Membership 和 Add To Album

桌面应用 SHALL 在 Inspector 中展示当前 asset 的 album memberships, 并允许用户将当前 asset 添加到已有 manual album.

#### Scenario: 查看 Asset Album Membership

- **WHEN** 用户选择一个 asset
- **THEN** Inspector 展示该 asset 当前所属 albums

#### Scenario: 添加 Asset 到 Album

- **WHEN** 用户在 Inspector 中选择一个 manual album 并确认添加
- **THEN** 桌面应用调用 Rust core 写入 membership, 并刷新 album list, 当前 Gallery query 和当前 asset detail

### Requirement: 提供 Generation Composer 和 Queue

桌面应用 SHALL 将 Queue 作为 operations console, 包含 batch composer, tasks queue 和 task detail. Queue MUST 展示 task attempts, timeline, log tail, outputs, asset/version/review links, 并根据 task status 提供合适操作.

#### Scenario: 打开 Queue Operations Console
- **WHEN** 用户打开 Queue
- **THEN** Workspace 展示 batch composer, task queue 和 task detail, 或在 compact desktop 下提供等价 panel switching

#### Scenario: 查看 Task Detail Cross Links
- **WHEN** 用户选择一个 task
- **THEN** Task Detail 展示 attempts, timeline, log tail, outputs, Open asset, Open version 和 Open review suggestion links, 如果这些 links 存在

### Requirement: 提供 Review Inbox

桌面应用 SHALL 提供 staged metadata Review workspace, 用于处理 pending metadata suggestions, local review draft, generated field results, suggestion history 和 canonical metadata accept. Review workspace MUST 明确区分 pending suggestion, local draft, generated result 和 canonical metadata.

#### Scenario: 打开 Review Workspace
- **WHEN** 用户打开 Review
- **THEN** Workspace 展示 pending suggestion list, selected suggestion draft detail, confidence, history, field regeneration 状态和 accept/reject actions

#### Scenario: Review 不混淆 Canonical Metadata
- **WHEN** Review 展示 pending suggestion 或 generated field result
- **THEN** UI 明确其为 staged 或 generated 状态, 不将其展示为已确认 canonical metadata

### Requirement: Review Inbox Workspace 支持 Editable Detail

桌面应用 SHALL 提供 Review Inbox workspace, 包含 pending suggestion list 和 selected suggestion editable detail form. Review 表单 SHALL 支持 title, description 和 JSON schema prompt 的字段级 Codex CLI 重新生成, 并在字段生成期间展示 loading 状态. Review detail SHALL 展示 suggestion history, 支持从 history pick 字段值到本地 draft, 支持 full suggestion regeneration, 并展示 confidence score.

#### Scenario: 选择 Pending Suggestion

- **WHEN** 用户在 Review Inbox 选择一条 pending suggestion
- **THEN** Workspace 展示可编辑的 title, description, JSON schema prompt, tag chips 和单选 category 表单

#### Scenario: 接受 Edited Suggestion

- **WHEN** 用户编辑 suggestion 表单后点击接受
- **THEN** 桌面应用调用 Rust core 写入 canonical metadata, 并刷新 pending list, Review badge, Gallery 和受影响的 Inspector detail

#### Scenario: 快速编辑 Tags

- **WHEN** 用户在 Review tag input 中输入 tag
- **THEN** 桌面应用展示已有 tag 自动补全, Enter 添加 chip, 新 tag 也可作为 chip 添加, 且重复 tag 不会重复显示

#### Scenario: 选择 Category

- **WHEN** 用户在 Review category 控件中选择 category
- **THEN** 桌面应用只允许选择当前 library 已存在 category 或空值, 不提供自动新建 category 行为

#### Scenario: 恢复 Edited Suggestion

- **WHEN** 用户点击恢复 selected suggestion
- **THEN** 桌面应用不调用 Rust core, 仅将当前本地 form state 恢复为 selected suggestion 初始值

#### Scenario: 重新生成 Review 字段

- **WHEN** 用户点击 title, description 或 JSON schema prompt 的重新生成按钮
- **THEN** 桌面应用调用后端 Codex CLI metadata generation command, 展示对应字段 loading 状态, 成功后只更新对应本地 form 字段, 不修改 pending suggestion 状态

#### Scenario: 重新生成期间保留其他字段可编辑

- **WHEN** 用户正在重新生成某一个 Review 字段
- **THEN** 桌面应用只禁用该字段和对应按钮, 其他 Review 字段仍可编辑

#### Scenario: 切换 Suggestion 后忽略旧响应

- **WHEN** 用户触发字段重新生成后切换到另一条 suggestion
- **THEN** 原请求完成时不得覆盖当前 selected suggestion 的 Review 表单内容

#### Scenario: Gallery 发起 Re-review

- **WHEN** 用户在 Gallery asset card 中点击重新 review
- **THEN** 桌面应用调用 Rust core 创建 pending suggestion, 刷新 Review badge, Gallery 和受影响的 Inspector detail

#### Scenario: 展示 Suggestion History

- **WHEN** 用户打开 selected suggestion detail
- **THEN** 桌面应用展示同一 asset 的 suggestion history, 并标识每条 suggestion 的 status, created time 和 reviewed time

#### Scenario: 从 History Pick 字段

- **WHEN** 用户从 suggestion history 中选择某个字段值
- **THEN** 桌面应用只更新当前本地 Review draft, 不调用 Rust core 写入 canonical metadata

#### Scenario: 重新生成完整 Suggestion

- **WHEN** 用户点击 full suggestion regeneration
- **THEN** 桌面应用调用后端生成新的 pending suggestion record, 刷新 history 和 pending list, 并保留当前 draft 的可恢复状态

#### Scenario: 展示 Confidence Score

- **WHEN** selected suggestion 包含可解析 confidence_json
- **THEN** 桌面应用展示 normalized overall score 和字段级 score chips

#### Scenario: Review 中加入 Album

- **WHEN** 用户在 Review 中选择一个 manual album 并执行 Add to Album
- **THEN** 桌面应用将 selected suggestions 对应 assets 或当前 suggestion asset 批量添加到该 album, 并保持 suggestion status 不变

### Requirement: Settings 展示 App Logs

桌面应用 SHALL 在 Settings 中提供 Logs 模块, 展示最近 app 生成日志, 支持刷新日志列表, 并允许用户查看选中日志内容.

#### Scenario: 查看最近日志

- **WHEN** 用户打开 Settings Logs 模块
- **THEN** 桌面应用展示最近 app-owned Codex image generation 和 metadata generation 日志列表, 包含 kind, path, modified time, size 和 preview

#### Scenario: 刷新日志列表

- **WHEN** 用户点击 Logs 模块的刷新按钮
- **THEN** 桌面应用重新加载最近日志列表, 并在刷新期间展示 loading 状态

#### Scenario: 查看日志内容

- **WHEN** 用户选择某条日志
- **THEN** 桌面应用读取并展示该日志内容预览

#### Scenario: 无日志 Empty State

- **WHEN** 当前没有 app-owned 生成日志
- **THEN** Settings Logs 模块展示明确 empty state

### Requirement: Library Switch 清理 Albums 和 Review State

桌面应用 MUST 在切换 library 时清理 selected album, selected suggestion, editable review form 和 stale Inspector selection.

#### Scenario: 切换 Library

- **WHEN** 用户从 Sidebar 切换到另一个 library
- **THEN** 桌面应用清空旧 library 的 selected album, selected suggestion, review form 和 Inspector selection, 并使用新 library 重新加载数据

### Requirement: 所有写操作通过 Rust Core

桌面应用 MUST 通过 Tauri command 调用 Rust core 完成资源库写操作, 不得在前端直接写 SQLite 或 managed files.

#### Scenario: 更新评分

- **WHEN** 用户在 Inspector 中修改 asset rating
- **THEN** 桌面应用调用 core service 更新 rating, 并根据返回结果刷新 UI state

### Requirement: 提供产品化 Gallery 工作台

桌面应用 SHALL 提供接近设计稿风格的 Gallery 工作台, 包含 library sidebar, workspace toolbar, gallery card grid 和 asset Inspector, 并保持高信息密度与稳定布局. 桌面窗口 SHALL 使用系统原生标题栏和窗口控制, Library 切换 SHALL 位于 Sidebar 顶部, 且应用窗口 SHALL 只显示一组窗口控制.

#### Scenario: 打开 Gallery 主工作流

- **WHEN** 用户打开已注册资源库的 Gallery
- **THEN** 桌面应用展示系统原生标题栏, library sidebar, Gallery 查询工具栏, asset card grid 和当前选中 asset 的 Inspector

#### Scenario: 选择 Gallery 图片

- **WHEN** 用户选择 Gallery 中的某个 asset card
- **THEN** card 显示选中状态且尺寸不发生变化, Inspector 加载该 asset 的详情

#### Scenario: 切换 Library

- **WHEN** 用户通过 Sidebar 顶部 Library selector 切换当前 Library
- **THEN** Workspace 和 Inspector 使用新 Library 的 core read model 重新渲染, 不保留旧 Library 的 selected asset detail

### Requirement: 提供 Sidebar Library Selector

桌面应用 SHALL 在 Sidebar 顶部提供 Library selector, 用于展示当前 Library 名称并切换到其他已注册且未隐藏的 Library. 该 selector MUST 只展示 Library 名称, 并使用向下箭头提示这是下拉控件.

#### Scenario: 切换当前 Library

- **WHEN** 用户从 Sidebar 顶部 Library selector 选择另一个 Library
- **THEN** 桌面应用将该 Library 设为当前上下文, 重新加载 Gallery, 清空当前 Inspector selection, 并展示新 Library 的状态

#### Scenario: 没有已注册 Library

- **WHEN** app-level registry 中没有可见 Library
- **THEN** Sidebar Library selector 展示 empty state, 并提供创建或打开 Library 的入口

### Requirement: 使用系统原生标题栏

桌面应用 SHALL 使用系统原生标题栏和窗口控制. 桌面应用 MUST NOT 同时渲染应用内自定义窗口控制和系统窗口控制.

#### Scenario: 打开桌面窗口

- **WHEN** 用户打开桌面应用
- **THEN** 窗口使用系统原生标题栏, 且只出现一组最小化, 最大化或关闭窗口控制

#### Scenario: 使用系统窗口行为

- **WHEN** 用户使用系统标题栏拖动窗口, 或点击系统最小化, 最大化或关闭控制
- **THEN** 宿主系统执行对应窗口操作

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

### Requirement: Sidebar 不展示 Asset Resolution

桌面应用 SHALL 避免在 Sidebar, Library Status 或 Gallery asset summary 中展示当前 asset 的文件 resolution. 文件级 resolution 信息 MUST 由 Inspector File section 承载.

#### Scenario: 查看左侧 Sidebar

- **WHEN** 当前 asset read model 包含 width 和 height
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

### Requirement: Checksum 展示算法标签

桌面应用 SHALL 在 Inspector File section 中展示 checksum algorithm 和 checksum digest.

#### Scenario: 展示 SHA-256 Checksum

- **WHEN** 当前 file context 的 checksum algorithm 为 `SHA-256`
- **THEN** Inspector 展示 `Checksum    SHA-256: $hash`

### Requirement: 通过 Core Read Model 加载 Gallery 和 Inspector

桌面应用 MUST 通过 Tauri command 调用 Rust core 获取 Gallery card 列表和 Inspector asset detail, 不得在前端或 Tauri command 层用多个低级结果重建业务查询语义. Gallery 和 Inspector MUST 展示 core read model 返回的 provider, model, prompt, tags, albums, lineage 和 file sections. 对通过 generation flow 创建的 asset, 桌面端 MUST 展示其 provider, model 和 prompt, 不得因前端字段映射缺失而回退为占位文案. Tauri command 层 MUST NOT 保留用于 Gallery 的 direct SQLite read command.

#### Scenario: 查询 Gallery 列表

- **WHEN** 用户修改搜索, filter 或 sort 条件
- **THEN** 桌面应用调用 core 定义的 Gallery query command 并使用返回的 card read model 渲染列表

#### Scenario: 加载 Inspector 详情

- **WHEN** 用户选择某个 asset
- **THEN** 桌面应用调用 asset detail command 并展示 prompt, provider/model, tags, albums, lineage 和 file sections

#### Scenario: 展示生成资产 Metadata

- **WHEN** 用户选择一个通过 generation flow 创建的 asset
- **THEN** Gallery card 和 Inspector 展示该 asset 的 provider, model 和 prompt

#### Scenario: 禁止 Gallery Direct SQL Command

- **WHEN** 桌面端需要加载 Gallery 列表
- **THEN** Tauri invoke handler 不提供绕过 core read model 的 `gallery_items` direct SQL command

### Requirement: Inspector 支持添加 Tag

桌面应用 SHALL 允许用户从 Inspector 给当前 asset 添加手动 tag. 写入 MUST 通过 Tauri command 调用 Rust core, 成功后刷新当前 Gallery query 和 Inspector detail.

#### Scenario: 添加 Tag 成功

- **WHEN** 用户在 Inspector 输入非空 tag 并确认添加
- **THEN** 桌面应用调用 core 写入 tag, 并在 Gallery card tag 列表与 Inspector tag 列表中展示该 tag

#### Scenario: 添加空 Tag

- **WHEN** 用户提交空白 tag
- **THEN** 桌面应用不调用写入 command, 并保持当前 Gallery 和 Inspector state 不变

### Requirement: Search 输入驱动 Core Gallery Query

桌面应用 SHALL 使用 Workspace toolbar 的 search 输入驱动 core Gallery query text 字段. 在真实 Tauri 模式下, 搜索结果 MUST 来自 `query_gallery` command, 并匹配 title, prompt 和 tags.

#### Scenario: 搜索 Prompt

- **WHEN** 用户在搜索框输入某个生成 prompt 的文本片段
- **THEN** Gallery 只展示 core 返回的匹配 asset

#### Scenario: 搜索 Tag

- **WHEN** 用户在搜索框输入某个 tag 名称
- **THEN** Gallery 展示 tags 中包含该名称片段的 asset

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

### Requirement: 提供可恢复错误和空状态

桌面应用 SHALL 对 Gallery 和 Inspector 的 loading, empty 和 recoverable error 状态提供明确 UI 反馈.

#### Scenario: Provider 不支持 Variation

- **WHEN** 用户从 Inspector 发起 variation 且当前 provider 不支持图生图
- **THEN** 桌面应用在 generation 或 variation 区域展示可恢复错误, 且 workbench 仍保持可操作

### Requirement: 支持 Inspector 响应式折叠

桌面应用 SHALL 在窄窗口中折叠 Inspector, 并保持 Gallery 查询和选择流程可用.

#### Scenario: 窄窗口查看 Asset 详情

- **WHEN** 窗口宽度不足以展示三栏布局且用户选择 asset
- **THEN** 桌面应用以 drawer, overlay 或详情面板展示 Inspector 内容, 且不遮挡主要导航和查询入口

### Requirement: Inspector 支持 App 内原图预览

桌面应用 SHALL 在 Inspector 中允许用户点击当前 asset 的图片缩略图, 并在 app 内 lightbox 中查看完整原图. Lightbox MUST 使用完整图片比例展示图片, 不得裁剪图片内容.

#### Scenario: 打开 Inspector 原图预览

- **WHEN** 用户选择一个带有图片路径的 asset 并点击 Inspector 图片缩略图
- **THEN** 桌面应用打开 app 内 lightbox, 并完整展示该 asset 图片

#### Scenario: 关闭 Inspector 原图预览

- **WHEN** lightbox 已打开且用户点击关闭按钮, 点击背景, 或按下 `Escape`
- **THEN** 桌面应用关闭 lightbox 并返回当前 Gallery 和 Inspector 上下文

#### Scenario: Gallery Card 点击语义保持不变

- **WHEN** 用户点击 Gallery 中的 asset card
- **THEN** 桌面应用选择该 asset 并在 Inspector 中展示详情, 不直接打开原图预览

### Requirement: Gallery 和 Inspector 缩略图不显示内部正方形描边

桌面应用 SHALL 在 Gallery 图墙和 Inspector 图片缩略图中展示图片内容, 不得在图片内部叠加额外正方形描边.

#### Scenario: 查看 Gallery 图墙图片

- **WHEN** 用户查看 Gallery asset card 图片
- **THEN** 图片内部不显示额外正方形描边

#### Scenario: 查看 Inspector 图片

- **WHEN** 用户查看 Inspector 中的当前 asset 图片
- **THEN** 图片内部不显示额外正方形描边

### Requirement: Generate Workspace 使用 Queue-Centric Task Workflow

桌面应用 SHALL 将 Generate workspace 设计为 Batch Composer, Tasks Queue 和 Task Detail 三栏工作流, 并通过 daemon task API 展示多任务状态.

#### Scenario: 打开 Generate Workspace

- **WHEN** 用户打开 Generate workspace
- **THEN** 桌面应用展示 batch task drafts, task queue list 和 selected task detail, 并显示 daemon 连接状态

#### Scenario: 创建 Batch Draft

- **WHEN** 用户点击 Add task
- **THEN** 桌面应用创建一个独立 task draft card, card 内 prompt editor 支持多行内容且不会按 newline 拆分 task

#### Scenario: Batch Enqueue

- **WHEN** 用户点击 Enqueue all
- **THEN** 桌面应用将每个 draft 的 prompt, provider, operation, params 和 source version snapshot 作为独立 task input 提交到 daemon

#### Scenario: Structured Import

- **WHEN** 用户导入结构化 JSON task list
- **THEN** 桌面应用将 JSON 中的每个 task object 转为 draft card, 并在提交前允许用户检查和编辑

### Requirement: Tasks Queue 支持多任务管理

桌面应用 SHALL 在 Tasks Queue 中展示 running, queued, retry waiting, completed, failed 和 canceled tasks, 并提供符合 task state 的操作.

#### Scenario: 展示 Task Row

- **WHEN** daemon 返回 task list
- **THEN** 每条 task row 展示 task type, prompt summary, provider, status, wait reason, attempt count, next retry time 和 quick actions

#### Scenario: 人工排序 Queued Tasks

- **WHEN** 用户拖动或点击 move up / move down 调整 queued task 顺序
- **THEN** 桌面应用调用 daemon reorder API, 并只提交 queued tasks 的新顺序

#### Scenario: 禁止排序 Non-Queued Tasks

- **WHEN** task 状态为 running, retry waiting, completed, failed 或 canceled
- **THEN** 桌面应用不展示 drag handle 或 move up / move down 控件

#### Scenario: Task Quick Actions

- **WHEN** task 支持 cancel, retry 或 duplicate
- **THEN** 桌面应用展示对应 action, 并通过 daemon API 执行后刷新 queue 和 detail

### Requirement: Task Detail 展示 Timeline, Logs 和 Outputs

桌面应用 SHALL 为 selected task 展示 input snapshot, attempts, structured timeline, live log tail, raw log preview, output links 和错误详情.

#### Scenario: 查看 Running Task Detail

- **WHEN** 用户选择 running task
- **THEN** Task Detail 展示当前 attempt, structured timeline, live log tail, cancel action 和 input snapshot

#### Scenario: 查看 Failed Task Detail

- **WHEN** 用户选择 failed task
- **THEN** Task Detail 展示 last error, error classification, attempt history, raw log preview, retry 或 duplicate action

#### Scenario: 查看 Completed Image Task

- **WHEN** 用户选择 completed image generation task
- **THEN** Task Detail 展示 asset, version, generation event 和 review suggestion output links

### Requirement: Review Inbox 显示 Task State Mirror

桌面应用 SHALL 在 Review Inbox 中将 metadata generation 的局部 loading state 关联到 daemon task, 并允许从 Review 表单打开 task detail.

#### Scenario: Field Generation Running

- **WHEN** 用户触发 Review field generation 且 task 正在运行
- **THEN** 对应字段显示 generating 状态和 Open task detail 入口, 其他字段仍可编辑

#### Scenario: Field Generation Retry Waiting

- **WHEN** Review field generation task 处于 retry waiting
- **THEN** Review Inbox 显示 retry waiting 和 next retry time, 且不覆盖当前 field draft

#### Scenario: Field Generation Result Stale

- **WHEN** metadata field generation 完成但用户已切换 suggestion 或修改 base revision
- **THEN** Review Inbox 显示 generated result available, 不得静默覆盖当前 draft

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
