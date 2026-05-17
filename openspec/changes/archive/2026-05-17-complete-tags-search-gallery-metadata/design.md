## Context

当前系统已经有 SQLite tags 表, asset_tags 表, gallery read model, asset detail read model, Tauri commands 和桌面三栏工作台. 问题集中在三条链路没有闭合:

- tags 只有 core 和 Tauri command, Inspector 中 `+` 入口尚未调用写操作.
- gallery 搜索由 core 和前端 helper 各自实现一部分, 真实 Tauri 模式下应以 core `GalleryReadService` 为准.
- 文生图创建新 asset 时先写入 generation event, 再通过普通 import 创建 asset/version, 导致 event 没有绑定新 asset/version. 这会让 provider, model, prompt 和 parameters 在图墙与详情页中缺失. 纯导入 asset 没有 generation metadata 是合理数据状态, 不应被误判为显示 bug.

## Goals / Non-Goals

**Goals:**

- 完成 Inspector 手动添加 tag 的闭环: 输入, 调用 Tauri command, core 写入, 刷新 gallery/detail.
- 让真实 gallery query 搜索覆盖 title, prompt, provider/model 和 tags, 并保持 provider, rating, review status, tags 的 AND 过滤语义.
- 修复 generation flow 的数据完整性, 使新生成 asset 的 current version 能关联 generation event.
- 让 UI 明确区分真实缺失 metadata 与生成资产关联缺失: imported-only 可以显示占位, generated asset 必须显示 provider/prompt.
- 增加 core, CLI 或 Tauri boundary, desktop state/UI 的回归测试.

**Non-Goals:**

- 不引入 schema migration 或新的持久化表.
- 不实现 tag rename, tag delete, tag color, tag autocomplete 管理页.
- 不实现全文索引或复杂 query language.
- 不改变 imported-only asset 的默认 title 行为.

## Decisions

1. 以 core gallery read model 作为搜索事实来源.

   Desktop Tauri 模式继续调用 `query_gallery`, 搜索语义在 Rust core 中实现. 前端 `applyGalleryQuery` 只保留 preview/test fallback, 并同步覆盖 prompt 字段以免 preview 与真实模式行为分叉. 相比在前端拼装搜索, 该方案能保持 CLI, GUI 和未来 IPC/API 的一致行为.

2. 修复 generation event 绑定, 不升级 schema.

   文生图创建新 asset 时应先把生成文件纳入 managed library, 再把 generation event 与 asset/version 建立关联, 或提供一个 core 内部 helper 在同一业务流程中创建 generated asset/version 并写入 event id. 实现必须保证 `asset_versions.generation_event_id`, `generation_events.asset_id` 和 `generation_events.output_version_id` 至少对新生成资产互相可追溯. 现有 schema 已有这些字段, 不需要 migration.

3. 手动 tag 使用已有 `add_tag_to_asset` command.

   Inspector 中新增轻量 tag input, 写入成功后重新加载 gallery 和当前 detail. 不在前端直接修改 SQLite, 也不把 UI 乐观更新作为唯一事实来源. 该选择牺牲一点交互即时性, 但更符合当前 "Rust core 是唯一写入事实来源" 的边界.

4. 搜索 prompt 来自 generation event, title 来自 asset canonical metadata.

   Gallery text search 应匹配 canonical title/category/status, generation provider/model/prompt 和 tags. `Untitled` 只是 UI fallback, 不参与 core 搜索. 如果用户搜索某个 prompt 片段, 生成资产应被命中.

## Risks / Trade-offs

- 既有旧数据中 generation event 未绑定新 asset/version -> 新实现无法自动恢复所有历史记录. 缓解: 本 change 只保证新写入正确, 并在测试中覆盖新 generation flow; 如需修复历史库, 后续单独设计 backfill.
- 在内存中过滤 gallery 资产随着库变大可能变慢 -> 当前 MVP 库规模可接受. 缓解: 保持查询语义集中在 core, 后续可在同一边界内替换为 SQL join 或 FTS.
- Tag 输入如果只做简单字符串校验会留下大小写或空白差异 -> 本次至少 trim 空输入并复用现有 unique name 约束; 大小写归一化策略后续作为 tag 管理能力处理.
- 生成资产的 title 仍可能为空 -> 这是 metadata review/canonical title 的独立流程. 缓解: 图墙仍可用 provider/prompt/tags 搜索, title 占位不再影响详情 metadata 正确性.
