## 1. Core Albums Read Model

- [x] 1.1 增加 album list read model, 覆盖 id, name, kind 和 item count.
- [x] 1.2 在 core album service 中实现 list albums 查询, manual album count 基于 `album_items` 统计.
- [x] 1.3 确认 album-scoped `GalleryQuery.album_id` 能返回 manual album 内容, 并补齐空 album 行为.
- [x] 1.4 为 list albums, manual album item count, add asset 后 album query 命中添加 core tests.

## 2. Tauri Commands And DTO Mapping

- [x] 2.1 增加 `list_albums` Tauri command 和 serializable `AlbumListItem` DTO.
- [x] 2.2 确认 `create_manual_album` 和 `add_asset_to_album` command 返回或刷新所需字段一致.
- [x] 2.3 确认 Review 相关 command 支持 editable accept flow 的字段映射.
- [x] 2.4 为 Albums 和 Review command mapping 增加可验证覆盖或针对性测试.

## 3. Desktop Albums Workspace

- [x] 3.1 增加 desktop album state: album list, selected album, album loading/error state.
- [x] 3.2 将 Albums 占位 provider grouping 替换为真实 album list, empty state 和 create manual album 表单.
- [x] 3.3 实现打开 album 后的 album detail header 和 album-scoped Gallery card grid.
- [x] 3.4 将 Inspector `Add to album` 接入已有 manual albums, 写入后刷新 album list, Gallery 和 selected asset detail.
- [x] 3.5 在 library switch 时清理 selected album 和 album stale state.

## 4. Desktop Review Inbox Workspace

- [x] 4.1 增加 selected suggestion 和 editable review form state.
- [x] 4.2 将 Review Inbox 列表升级为 pending suggestion list + selected suggestion detail layout.
- [x] 4.3 实现本地编辑 title, description, tags 和 category 后 accept, 并保留失败时的编辑内容.
- [x] 4.4 实现 restore 后恢复本地表单, 且不修改 canonical metadata.
- [x] 4.5 在 accept 后刷新 Gallery query, 并在当前 Inspector asset 受影响时刷新 detail.
- [x] 4.6 在 library switch 时清理 selected suggestion 和 review form stale state.

## 5. Generation And Review State Integration

- [x] 5.1 确认 generation 成功后保持当前 workflow, 不强制切换到 Review Inbox.
- [x] 5.2 确认生成产生 pending suggestion 时 Review badge 和 asset detail pending state 可刷新.
- [x] 5.3 确认 factual generation metadata 立即展示, pending suggestion metadata 不作为 canonical metadata 展示.
- [x] 5.4 修复 generation 成功后未自动创建 pending metadata suggestion 的 core regression.
- [x] 5.5 为 generation -> Review Inbox 队列补充 core regression test.

## 6. Frontend Tests And Visual QA

- [x] 6.1 添加 frontend state tests: 创建 album 后刷新列表, 打开 album 后 query 携带 album id.
- [x] 6.2 添加 frontend state tests: Add to album 后 Inspector membership 更新.
- [x] 6.3 添加 frontend state tests: accept/remove suggestion 后 pending list 和 badge 更新.
- [x] 6.4 添加 frontend state tests: library switch 清理 album 和 review stale state.
- [x] 6.5 检查 Albums 和 Review workspace 在 desktop, medium 和 narrow viewport 下无文本溢出或 incoherent overlap.

## 7. Verification

- [x] 7.1 运行 `cargo fmt --all --check`.
- [x] 7.2 运行 `cargo test --offline -p imglab-core -p imglab-cli`.
- [x] 7.3 运行 `cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok`.
- [x] 7.4 运行 `cd apps/desktop && npm run test && npm run build`.
- [x] 7.5 如果本地 Tauri 依赖可用, 运行 `cargo check --offline -p imglab-desktop`.
- [x] 7.6 运行 `openspec validate harden-albums-review-workflows`.

## 8. Review Metadata Refinement

- [x] 8.1 增加 canonical `schema_prompt` 和 suggestion `suggested_schema_prompt` 持久化字段.
- [x] 8.2 生成 pending suggestion 时创建 JSON schema prompt draft.
- [x] 8.3 Review accept 支持写入 description, schema prompt, tags 和 category.
- [x] 8.4 Review UI 将 tags 改为 chip input + autocomplete, 支持新 tag chip 和去重.
- [x] 8.5 Review UI 将 category 改为已有 category 单选, 不自动创建新 category.
- [x] 8.6 更新 core, CLI 和 frontend tests.
- [x] 8.7 重新运行格式化, 测试, build 和 OpenSpec validate.

## 9. Review Interaction Refinement

- [x] 9.1 移除 Review UI reject 入口, 改为恢复本地表单初始值.
- [x] 9.2 为 title, description 和 JSON schema prompt 增加重新生成按钮.
- [x] 9.3 统一 Review category select 与其他 select 控件样式.
- [x] 9.4 Gallery asset 支持重新进入 Review Inbox, 并刷新 Review pending 状态.
- [x] 9.5 更新 OpenSpec, frontend state tests 和验证命令.
