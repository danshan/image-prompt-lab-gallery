## 1. Core 数据关联

- [x] 1.1 为文生图新建 asset/version 的流程补齐 generation event 绑定, 确保 `generation_events.asset_id`, `generation_events.output_version_id` 和 `asset_versions.generation_event_id` 可互相追溯.
- [x] 1.2 保持图生图 child version 流程继续绑定 generation event, 并补充 output version id 回填或等价追溯能力.
- [x] 1.3 添加 core regression test, 覆盖 fake provider 文生图后 gallery/detail 能返回 provider, model, prompt 和 parameters.
- [x] 1.4 添加 core regression test, 覆盖图生图 child version 的 lineage/detail 能返回当前 version 对应 generation event.

## 2. Core Search 和 Read Model

- [x] 2.1 扩展 core gallery text search, 覆盖 title, category/status, provider/model, generation prompt 和 tags.
- [x] 2.2 扩展 core search service 的文本匹配, 至少覆盖 title, generation prompt 和 tags.
- [x] 2.3 确认 generated asset 的 gallery card read model 使用当前 version 或最新 event 返回 provider/model/image path/tags.
- [x] 2.4 添加 search/gallery tests, 覆盖按 prompt 文本搜索, 按 tag 文本搜索, tag filter 与其他 filter 的 AND 语义.

## 3. Tauri 和 Desktop UI

- [x] 3.1 确认 `query_gallery`, `get_asset_detail`, `add_tag_to_asset` command 的输入输出字段与 desktop TypeScript 类型一致.
- [x] 3.2 在 Inspector tags section 增加手动 tag 输入和提交行为, 空白 tag 不触发 command.
- [x] 3.3 Tag 添加成功后刷新当前 gallery query 和当前 asset detail, 并清理输入状态.
- [x] 3.4 同步 preview mode 的 `applyGalleryQuery` 字段, 避免搜索 prompt/tag 的预览行为与 Tauri 模式明显分叉.

## 4. 验证

- [x] 4.1 运行 Rust core 和 CLI tests, 至少包括 `cargo test -p imglab-core` 与 `cargo test -p imglab-cli`.
- [x] 4.2 运行 desktop state tests, 至少包括 `npm test --prefix apps/desktop`.
- [x] 4.3 手工验证真实 Tauri 模式: 生成图片后图墙显示 provider, Inspector 显示 prompt, 添加 tag 后可通过 search 和 tag filter 命中.
- [x] 4.4 确认 imported-only asset 仍允许 title/provider/prompt 缺失并显示占位, 不被当作 generated metadata regression.

## 5. Title 和 Rating 交互

- [x] 5.1 为 generation flow 创建基于 prompt 的默认 title, 并保持 imported-only asset 仍可无 title.
- [x] 5.2 支持 Inspector 双击 title 后编辑 canonical title, 成功后刷新 Gallery 和 Inspector.
- [x] 5.3 将 Gallery 和 Inspector 的 rating 显示改为星级样式.
- [x] 5.4 将 Inspector rating 输入改为星级控件.
- [x] 5.5 调整 tag 添加交互, 仅在点击 `+` 后显示 tag 输入框.
