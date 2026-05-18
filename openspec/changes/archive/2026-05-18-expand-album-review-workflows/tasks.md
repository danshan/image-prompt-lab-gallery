## 1. Core Schema 和 DTO

- [x] 1.1 提升 library schema version, 为 `albums` 增加 `sort_order`, 并为既有 albums 回填稳定顺序.
- [x] 1.2 扩展 Album DTO 和 service trait, 覆盖 rename, delete, remove asset, batch add assets, reorder albums 和 reorder album items.
- [x] 1.3 扩展 Gallery sort DTO, 增加 `album_order`, 并约束其只能用于 album-scoped query.
- [x] 1.4 定义 typed Smart Album query DTO 和 validation helper, 覆盖 text, tags, providers, min rating, review status, category, status, created date range 和 sort.
- [x] 1.5 扩展 Metadata Review DTO 和 service trait, 覆盖 suggestion history, batch accept, batch reject, full suggestion regeneration 输入输出和 confidence normalization.

## 2. Core Album 和 Smart Album 实现

- [x] 2.1 实现 album list 按 `sort_order, name` 返回, 并在创建 album 时分配下一个 sort order.
- [x] 2.2 实现 rename album 和 delete album, 删除 album 时清理该 album 的 `album_items` 且不删除 assets.
- [x] 2.3 实现 remove asset from manual album, 并拒绝 smart album 的 manual-only 操作.
- [x] 2.4 实现 batch add assets to manual album, 保持重复 membership 幂等且 invalid asset 或 invalid album kind 原子失败.
- [x] 2.5 实现 reorder albums, 验证提交 ids 属于当前 library 后事务更新 sort order.
- [x] 2.6 实现 reorder manual album items, 验证 ordered asset ids 完整属于目标 album 后事务更新 `album_items.sort_order`.
- [x] 2.7 将 typed Smart Album query validation 接入 create/update smart album 路径, 并使用 `assets.created_at` 实现 created date range.
- [x] 2.8 更新 album-scoped Gallery query, 支持 `album_order` 并在非 album query 使用时返回 `InvalidGalleryQuery`.

## 3. Core Review 实现

- [x] 3.1 实现按 asset 读取 suggestion history, 返回 pending, accepted 和 rejected records 及 created/reviewed time.
- [x] 3.2 实现 batch accept, 使用 per-suggestion final payload, 先全量验证 pending status/category/tags, 再事务写 canonical metadata 和 suggestion status.
- [x] 3.3 实现 batch reject, 全量验证 pending status 后事务标记 rejected.
- [x] 3.4 实现 full suggestion regeneration 的 core/Tauri 边界, 成功时创建新的 pending suggestion record, 失败时不改变现有 draft 或 history.
- [x] 3.5 实现 confidence normalization helper, 支持 `0..1`, `0..100`, missing 和 malformed JSON.

## 4. Tauri Commands

- [x] 4.1 增加 album rename/delete/remove/batch add/reorder commands, 并保持 camelCase input/output contract.
- [x] 4.2 增加 typed smart album create/update 或 validation command, 供 Smart Album builder 和 live preview 使用.
- [x] 4.3 增加 review history, batch accept, batch reject 和 full suggestion regeneration commands.
- [x] 4.4 确保所有新增 command 错误都映射到既有 `CommandError` shape.

## 5. Desktop State 和 UI

- [x] 5.1 扩展 `workbench-state.ts`, 支持 album reorder state, selected gallery assets, review multi-select 和 batch action helpers.
- [x] 5.2 更新 Albums workspace, 支持 album list drag reorder, rename, delete 和 selected album 清理.
- [x] 5.3 更新 manual album detail, 支持 asset card drag reorder, remove asset 和 batch add selected gallery assets.
- [x] 5.4 实现 Smart Album builder controls, 覆盖 typed query 字段并展示 live preview.
- [x] 5.5 更新 Review Inbox, 支持 suggestion multi-select, batch accept, batch reject 和 Add selected assets to album.
- [x] 5.6 更新 Review detail, 展示 suggestion history, 支持 field pick 到 draft, full suggestion regeneration 和 confidence score/chips.
- [x] 5.7 确保 Album 和 Review 写操作完成后刷新 Gallery, album list, Review badge 和受影响 Inspector detail.

## 6. 测试和验证

- [x] 6.1 增加 core tests: album list reorder, manual album item reorder, rename/delete/remove/batch add 和 invalid smart album manual operation.
- [x] 6.2 增加 core tests: typed smart album query validation, created date range 和 `album_order` sort 约束.
- [x] 6.3 增加 core tests: suggestion history, batch accept rollback, batch reject rollback 和 confidence normalization.
- [x] 6.4 增加 desktop state tests: review multi-select, batch accept draft merge, history field pick 和 library switch cleanup.
- [x] 6.5 运行 Rust 和 desktop 测试, 修复新增测试暴露的问题.
- [ ] 6.6 手工验证 album drag order 持久化, manual album item order 持久化, batch review 刷新和 full suggestion history 追加.
