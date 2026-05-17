## Context

当前项目已经完成 MVP 主干: Rust core 提供 resource library, asset/version, generation event, albums, search 和 metadata review; Tauri command 暴露核心操作; desktop 使用三栏 workbench 展示 Gallery 和 Inspector.

当前缺口集中在交付体验而非底层模型. Albums view 仍是基于 provider 的占位分组, 没有真实 album list 和 album detail. Review Inbox 目前缺少选择 suggestion, 查看上下文, 本地编辑, 恢复, 重新生成字段后接受的工作区形态. 这会导致生成和导入后的组织流程无法闭环.

已确认的产品方向是两个独立工作区:

- Albums 负责长期组织和 album membership.
- Review 负责 metadata suggestion 的信任门.

Generate Image 完成后不强制跳转 Review Inbox. 生成结果立即进入 Gallery, factual generation metadata 立即可见; title, description, tags 和 category 等 organization metadata 通过 pending suggestion 进入 Review 队列.

## Goals / Non-Goals

**Goals:**

- 将 Albums 视图升级为真实 album 工作区, 支持列出 albums, 创建 manual album, 打开 album 和展示内容.
- 在 Inspector 中补齐当前 asset 的 album memberships 和 `Add to album` 交互.
- 将 Review Inbox 升级为 selectable suggestion list + editable detail form.
- 保持 review-first 语义: suggestion 被接受前不得写入 canonical metadata.
- 写操作后刷新 pending suggestions, Gallery query, selected Inspector detail 和 Sidebar badge.
- 保持 Rust core 为业务语义来源, React 只管理 UI interaction state.

**Non-Goals:**

- 不实现完整 smart album builder.
- 不实现 bulk review.
- 不实现 album delete, rename, remove asset, drag ordering.
- 不合并 Albums 与 Review 为统一 curation inbox.
- 不实现 native OpenAI 或 Grok provider.

## Decisions

### 1. Albums 和 Review 保持独立工作区

Albums 和 Review 的生命周期不同. Albums 是长期组织结构, Review 是短期待处理队列. 合并成统一 curation inbox 会引入跨模块状态和交互复杂度, 但当前真实缺口是两条基础流程都尚未闭环.

替代方案是做统一整理台, 在一个队列里同时完成 review, album assignment, rating 和 tag. 该方案效率高, 但会把本次范围扩大为 curation workstation. 当前选择先让两个模块独立可用, 后续再决定是否合并。

### 2. Album detail 复用 Gallery query 和 Gallery card

打开 album 后不新增第二套 asset 列表语义, 而是设置 `GalleryQuery.album_id` 并复用 `query_gallery`. 这样 album detail 与 Gallery 的 search, card read model, selection 和 Inspector 数据保持一致.

需要补齐的是 album list read model, 至少包含 id, name, kind 和 item count. Smart album count 不是本次硬要求; 如果现有 query helper 能低成本计算可以提供, 否则 UI 显示 unavailable.

### 3. Review 编辑是本地 form state, 不是修改 suggestion record

Review Inbox 选择 suggestion 后, UI 以 suggestion 初始值填充表单. 用户修改 title, description, JSON schema prompt, tags 和 category 后点击 accept, 一次性提交最终确认值到现有 accept command. Core 仍保留原 suggestion 作为历史, canonical metadata 保存人工确认后的结果.

替代方案是新增 edit suggestion API. 该方案会让 pending suggestion 本身变成可变草稿, 但当前没有必要, 且会弱化原始 AI 建议与人工确认结果之间的边界.

### 4. JSON Schema Prompt 是独立元数据, 不复用 Description

Description 保留为面向人类的普通说明. JSON Schema Prompt 作为独立字段保存, suggestion 中使用 `suggested_schema_prompt`, accepted 后写入 canonical asset metadata 的 `schema_prompt`. 这样后续可以同时支持自然语言说明和可复用 prompt spec, 不会把两种语义混入同一个字段.

当前 generation 可以基于原始 prompt 和参数生成 schema prompt draft. 后续接入 vision-capable metadata suggestion provider 后, 再用最终图片内容补全材料, lighting, composition, output mood 和 avoid 约束.

### 5. Category 是受控单选, Tags 是可扩展多选

Category 用作粗粒度主归类, 只能从当前 library 已存在 category 中选择或留空. AI 可以推荐已有 category, 但不能自动创建新 category. 如果没有合适 category, 保持为空, 后续由用户通过显式管理动作创建.

Tags 用作细粒度、多值标签. Review UI 使用 chip input, 输入已有 tag 时自动补全, 输入不存在 tag 时创建新 chip. Accept 时 core 继续通过 confirmed tag 写入路径创建或复用 tag.

### 6. Generate Image 不自动导航到 Review Inbox

生成完成后的主要用户任务可能是继续调 prompt, 查看 Gallery, 或检查 lineage. 强制跳转 Review Inbox 会打断生成流. 因此生成完成只产生 pending suggestion, 增加 Review badge, 并在 Gallery/Inspector 显示 pending state.

Review Inbox 是生成流程产生的待处理队列, 不是生成流程的下一屏.

### 7. Library switch 清理跨库状态

Albums 和 Review 都有 selection state. 切换 library 时必须清空 selected album, selected suggestion, review form state 和 stale Inspector selection, 避免旧 library 的 id 被带入新 library command.

## Risks / Trade-offs

- Album list read model 可能需要新增 core trait 方法和 Tauri command. → 将 read model 控制在 id, name, kind, item count, 不引入通用 repository abstraction.
- Smart album count 可能牵涉 query evaluation. → 本次允许 smart count unavailable, 不阻塞 manual album 闭环.
- Review accept 后需要刷新多个视图, 容易出现 stale UI. → 将 refresh 顺序固定为 pending suggestions, Gallery query, selected asset detail, 并补 frontend state tests.
- `add_asset_to_album` 对重复 membership 是 no-op. → UI 将重复添加视为成功, 并通过 refreshed detail 展示真实 membership.
- Review form tags parsing 可能引入输入歧义. → 本次使用 tag chip input, Enter 确认, accept 前 trim 空值并去重.
