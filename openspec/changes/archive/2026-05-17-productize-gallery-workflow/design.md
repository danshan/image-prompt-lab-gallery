## Context

当前 MVP 已经包含 Tauri + React 桌面壳, Rust core service boundary, SQLite resource library, Gallery, Albums, Review Inbox, Generation Queue 和 Inspector 的基础 UI. 现状的问题是 Gallery 主工作流仍偏原型: 前端用较薄的 `gallery_items` 数据拼装卡片和 Inspector, 搜索/过滤/排序语义尚未形成统一 core contract, 页面视觉也没有达到设计稿中的信息密度和工具感.

本变更基于已确认的 `docs/superpowers/specs/2026-05-17-productized-gallery-workflow-design.md`, 将范围限定在 Gallery 主工作流: 查询资产, 选择资产, 查看 Inspector 详情, 处理 metadata/review/lineage/file context, 以及从当前 version 发起 variation 的入口.

约束:

- Rust core 仍是 GUI 与 CLI 写操作的唯一业务事实来源.
- 查询语义也应由 core 定义, 避免 GUI 和 CLI 长期分叉.
- 设计稿风格要尽可能贴近, 但允许为 loading, empty, error 和 responsive state 做产品化微调.
- 不引入 cloud sync, encryption, daemon, graph lineage 或完整 smart album builder.

## Goals / Non-Goals

**Goals:**

- 提供 core 定义的 Gallery 查询 DTO 和稳定 sort/filter 语义.
- 提供 Gallery card 和 Asset detail 读模型, 让 Tauri/React 不再从低级结果临时拼装主工作流.
- 重构桌面端三栏 workbench, 接近设计稿的信息架构和视觉密度.
- Inspector 展示 prompt, provider/model, tags, albums, versions/lineage 和 file context.
- 从 Inspector 暴露 `Generate variation` 入口, provider 不支持图生图时返回可恢复 capability error.
- 添加覆盖 query, detail aggregation, command mapping 和前端关键 state 的测试.

**Non-Goals:**

- 不实现 graph-style lineage.
- 不实现完整 metadata editor 或 raw payload workstation.
- 不实现完整 smart album builder.
- 不实现完整 queue history 管理.
- 不要求 Codex CLI adapter 真正支持稳定图生图.
- 不引入新 provider 或新外部服务.

## Decisions

### Decision 1: Gallery 查询语义下沉到 Rust core

采用 `GalleryQuery` 表达文本, providers, min rating, review status, tags, album 和 sort. Core 负责把 query 转成 SQLite 查询并返回排序后的结果.

替代方案是前端对已加载列表做内存过滤. 该方案实现快, 但会导致大库表现不稳定, 并让 CLI 与 GUI 查询语义分叉. 本项目已经明确 Rust core 是业务边界, 因此查询语义也放在 core 更符合长期维护目标.

### Decision 2: 增加主工作流读模型, 不泄漏 SQLite row

新增或扩展 `GalleryAssetView` 和 `AssetDetailView`. `GalleryAssetView` 服务 card 列表, `AssetDetailView` 服务 Inspector. 它们是桌面 workflow DTO, 不是数据库 row, 也不是 React component state.

替代方案是由 Tauri 或 React 多次调用现有低级 command 再聚合. 这会增加 round-trip, error state 和缓存复杂度, 也会让 UI 理解过多 domain join 细节.

### Decision 3: 展示字段允许为空, 但不伪造真实数据

当前 schema 和导入/生成流程可能无法稳定提供 file size, dimensions, generation duration, provider model label 等字段. Core 应返回 `None` 或空集合, UI 展示明确的 unavailable state.

替代方案是为了贴近设计稿在前端长期保留 mock 数据. 这会污染用户对真实资源库状态的理解, 不利于后续功能验证.

### Decision 4: Image-to-image 采用 capability-aware 边界

Inspector 提供 variation 入口, `start_generation` 支持 `input_version_id`. 如果 provider 不支持图生图, core 返回 `UnsupportedProviderCapability`, Tauri 映射为 recoverable command error, UI 在操作附近显示提示.

替代方案是本轮完整打通图生图 provider 行为. 当前 Codex CLI adapter 受限且行为不稳定, 强行完整实现会把主工作流风险集中到 provider adapter 上.

### Decision 5: UI 拆分为 query state, selection state 和 detail state

实现时应避免继续把所有逻辑堆在 `main.tsx`. 建议拆出 Gallery query state, card list, Inspector sections, sidebar/status panel 和 command boundary helpers. 组件拆分以现有项目规模为准, 不引入重型状态管理库.

替代方案是一次性引入全局 store. 当前 MVP 规模不需要新依赖, 过早引入会增加迁移成本.

## Risks / Trade-offs

- Scope creep 到完整 metadata workstation -> 将本轮验收限定为 Gallery 主工作流和 Inspector 展示/入口.
- Query join 变复杂影响性能 -> 先定义稳定语义和 focused tests, 必要时添加 index, 避免 prematurely 泛化.
- UI 与真实数据字段不匹配 -> Core nullable, UI explicit unavailable state, 不用 mock 冒充真实数据.
- Provider capability error 体验不佳 -> 错误必须 recoverable, 并显示在 variation/generation 相关区域, 不破坏整个 workbench.
- 前端重构范围过大 -> 按 core read model, Tauri command, frontend state, UI polish 分阶段提交.

## Migration Plan

本变更不要求破坏性迁移. 新增展示字段优先 nullable 或由现有表聚合得到. 如果实现过程中必须新增 SQLite 列或索引, 应提供向前兼容 migration, 并保持旧资源库可打开.

实施顺序:

1. 添加 OpenSpec delta 并提交.
2. 添加 core DTO, query/detail service 和测试.
3. 添加 Tauri command 与 error mapping.
4. 重构前端 state 和组件.
5. 调整视觉和 responsive behavior.
6. 运行 Rust, Tauri/desktop 和 frontend 测试, 再做浏览器视觉检查.

回滚策略是按 commit 边界回滚. 因为 schema 字段优先 nullable, 回滚不应要求删除用户数据.

## Open Questions

无阻塞性问题. 实现时可以在不改变 spec 的前提下决定具体字段命名和组件文件拆分. 如果发现必须引入破坏性 schema 迁移或 provider 行为假设, 需要先回到 Plan 重新评估.
