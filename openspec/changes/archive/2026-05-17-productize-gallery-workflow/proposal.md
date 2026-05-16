## Why

当前桌面端已经具备 Gallery, Inspector, Review Inbox, Generation Queue 等 MVP 骨架, 但 Gallery 主工作流仍偏原型: 搜索和过滤语义不完整, Inspector 依赖前端拼装和占位字段, 页面信息密度与设计稿差距较大. 这会让后续相册, metadata review, version lineage 和 provider 能力继续分叉, 增加维护成本.

本变更将 Gallery 主工作流产品化: 以设计稿为视觉基准, 同时把搜索, 过滤, 排序和资产详情聚合下沉到 Rust core, 让 GUI 继续只负责 UI 状态和交互表达.

## What Changes

- 增加 core 定义的 Gallery 查询语义, 覆盖文本, provider, rating, review status, tags, album 和 sort.
- 增加面向桌面端的 Gallery card 读模型和 Asset detail 读模型, 由 core 聚合 asset, version, generation event, tags, albums, lineage 和 file 信息.
- 调整 Tauri command 边界, 提供 Gallery 查询和资产详情读取 command, 并保持错误映射为 `{ code, message, recoverable }`.
- 按设计稿重构桌面端三栏 workbench, 包括 sidebar, toolbar, filter chips, gallery cards, Inspector sections 和 responsive collapse.
- 在 Inspector 中提供从当前 version 发起 variation 的入口, 并通过 provider capability 返回可恢复错误.
- Albums, Queue, Settings 保持可用并与新视觉一致, 但本轮不扩展为完整子系统.
- 不引入 breaking change. 现有资源库应继续打开, 新增展示字段允许为空.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: Gallery 主工作流, Inspector 信息架构, responsive behavior 和 Tauri read command 行为发生变化.
- `albums-search`: 搜索, 过滤和排序语义扩展为 core 定义的 Gallery query.
- `image-generation`: generation input 支持从当前 version 发起 variation, provider 不支持时返回可恢复 capability error.
- `asset-versioning`: Inspector 需要展示当前 version, parent/current lineage 和 version count.
- `resource-library`: Gallery 和 Inspector 需要展示文件位置, checksum, integrity status 等 resource library file context.

## Impact

- Rust core: 新增或扩展 Gallery query DTO, Gallery card/detail 读模型, 查询 service 和必要索引.
- Tauri app: 新增 `query_gallery` 和 `get_asset_detail` command, 扩展 `start_generation` 的 `input_version_id` capability handling.
- React desktop UI: 重构 Gallery workspace, sidebar, Inspector, query state, selected asset detail state, loading/error state 和 responsive styles.
- Tests: 增加 core query/detail 聚合测试, Tauri DTO/error mapping 测试, frontend state 和关键交互测试.
- Docs/specs: 更新 OpenSpec delta, 后续归档时同步到现有 specs.
