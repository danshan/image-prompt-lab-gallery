## Context

当前项目已经具备 local-first resource library, Gallery query, manual/smart album 基础能力, Review Inbox, 单条 suggestion accept/reject, 以及字段级 Codex CLI metadata regeneration. 现有 Album 能创建和添加 asset, 但缺少重命名, 删除, 移除, 排序和批量添加. 现有 Smart Album 只做 JSON key allowlist, 还不是可被 UI builder 稳定消费的 typed contract. 现有 Review 以单条 suggestion 为核心, 缺少批量动作, suggestion history 对比, full suggestion regeneration 和 confidence 评分展示模型.

关键约束:

- Rust core 是 GUI 与 CLI 写操作的唯一业务事实来源.
- Review-first 规则不变: suggestion 被接受前不得写入 canonical asset metadata.
- metadata suggestions 需要保留 history, 重新生成不得覆盖旧记录.
- 本轮不引入通用规则引擎, 多用户协作, cloud sync 或 native OpenAI/Grok metadata clients.

## Goals / Non-Goals

**Goals:**

- 补齐 Album 管理主路径: album list 排序, rename, delete, manual album 内 asset 排序, remove asset, batch add assets.
- 将 Smart Album builder 固化为 typed query contract, 支持 text, tags, providers, min rating, review status, category, status, created date range 和 sort.
- 将 Review 从单条处理扩展为可批量处理的审核工作台, 支持 history compare, field pick merge, batch accept/reject, full suggestion regeneration, confidence visualization 和 add selected assets to album.
- 保持批量写入原子性, 避免局部成功导致 UI 与 core 状态难以解释.

**Non-Goals:**

- 不实现 nested query groups 或通用 smart query AST.
- 不改变 Review-first 的持久化规则.
- 不新增 cloud sync, 多用户协作或权限模型.
- 不实现 graph-style lineage visualization.
- 不把 Codex metadata generation 替换为 native provider client.

## Decisions

### 1. 一个 change, 三个边界

本次使用一个 OpenSpec change, 但实现拆成 Album domain, Smart Album domain 和 Review domain.

原因: Review 中直接加入 album 是跨域工作流, 拆成两个 change 会让这条路径被迫延后. 但如果实现时混合所有逻辑, Album 排序, Smart query 和 Review batch 写入会互相污染. 因此只在显式 command 层连接跨域动作, 例如 batch add assets to album.

备选方案:

- Album-first: 风险更低, 但无法一次解决 Review 到 Album 的工作流.
- UI-first: 可见进展快, 但会把 query 和 batch 语义推到前端, 违反 core 边界.

### 2. Album list 排序持久化到 `albums.sort_order`

`albums` 表新增 `sort_order INTEGER NOT NULL DEFAULT 0`. migration 对既有 albums 按当前稳定顺序赋值. `list_albums` 按 `sort_order, name` 返回.

manual album 内 asset 排序继续使用既有 `album_items.sort_order`. 新增 `album_order` sort 语义, 让 album-scoped gallery query 能按拖拽顺序返回.

备选方案:

- 只排序 album items: 无法满足 album list 拖拽.
- 在前端保存 album order: 会让排序事实来源脱离 core, library 切换和 CLI 也无法复用.

### 3. Smart Album 使用 typed JSON contract

Smart Album 仍以 JSON 存储, 但接受的 JSON shape 改为 typed contract, 而不是任意 key allowlist. 支持字段为 `text`, `tags`, `providers`, `minRating`, `reviewStatus`, `category`, `status`, `createdAtFrom`, `createdAtTo`, `sort`. created date 使用 `assets.created_at`.

原因: 这样可以让桌面 builder, core validation 和未来 CLI 自动化共享同一契约, 同时避免过早实现嵌套条件树.

### 4. Batch review 使用 per-suggestion final payload

`batch_accept` 接收多条 final payload. 当前打开的 suggestion 可以使用本地 draft, 其他选中 suggestion 使用 persisted suggestion 值. core 先验证全部 suggestion 仍为 pending, category 合法, tags 可写入, 再在单个 transaction 中写 canonical metadata, tags 和 statuses.

原因: 该模型兼顾批量效率和 review-first. 当前 draft 不会丢失, 其他未打开项也不需要额外编辑 UI.

### 5. Suggestion history append-only, field merge 只更新 draft

Full suggestion regeneration 创建新的 pending suggestion record, 不覆盖历史. 字段级 regeneration 继续只更新当前本地 draft. History compare UI 允许从任意 history row 将字段值 pick 到 draft, 但不会修改 suggestion record.

原因: history 是审计和对比基础. merge 操作应是用户审核草稿行为, 不应伪造 AI suggestion history.

### 6. Confidence 只规范化展示, 不改写 raw JSON

`confidence_json` 展示 contract 为 `overall` 和 `fields.title/description/schemaPrompt/tags/category`. 分数支持 `0..1` 或 `0..100`, UI normalize 到 `0..100`. malformed 或 missing 值显示 unknown, 不阻塞 accept/reject.

原因: confidence 是辅助判断, 不应成为审核写入的硬依赖. 保留 raw JSON 可以兼容未来更丰富字段.

## Risks / Trade-offs

- [Risk] Album-scoped gallery order 与已有 sort 语义冲突. → Mitigation: 显式增加 `album_order` sort, 仅在 album context 下有效, 非 album query 使用该 sort 返回 `InvalidGalleryQuery`.
- [Risk] Batch accept 部分成功会破坏 review-first 的可解释性. → Mitigation: core 先全量验证再 transaction 写入, 不提供 partial success.
- [Risk] Smart query typed contract 与 Gallery query 逐渐漂移. → Mitigation: 在 core 内转换并复用 Gallery query validation, specs 同步约束字段.
- [Risk] Suggestion history compare UI 过重. → Mitigation: 第一版只做字段级 pick 到 draft, 不做 nested diff 或自动 merge policy.
- [Risk] Confidence 结构不稳定. → Mitigation: 只定义最小评分 contract, unknown keys 保留但不展示为业务语义.

## Migration Plan

1. 将 schema version 提升一版, 为 `albums` 增加 `sort_order`.
2. 对既有 albums 按当前 `ORDER BY name` 或创建时间的稳定顺序回填 `sort_order`.
3. 保持既有 `smart_query_json` 列不变, 但新写入和更新必须通过 typed contract validation.
4. 既有 metadata suggestions 不迁移. 缺少 created/reviewed 字段的 read model 按现有列返回.
5. 回滚策略是保留新增列不使用; 不删除用户数据. 已创建的新 smart query 若使用新增字段, 旧版本可能无法正确解释, 因此实施前不建议跨版本混用同一 library.

## Open Questions

无. 本轮已明确选择:

- Album 与 Review 合并一个 change.
- Smart Album builder 使用包含 created date range 的 typed contract.
- Album list 与 manual album items 都支持拖拽排序.
- Batch accept 使用当前打开 draft 覆盖当前 suggestion, 其他选中项使用 persisted suggestion.
- Suggestion history 支持字段级 pick/merge.
- 字段级 regeneration 和 full suggestion regeneration 同时保留.
- Review add to album 作用于选中 suggestions 对应 assets.
- Confidence 使用最小规范化评分模型.
