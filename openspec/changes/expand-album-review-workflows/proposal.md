## Why

当前 Albums 和 Review 已有基础能力, 但还不足以支撑真实的图片整理和 metadata 审核工作流. 用户需要在 Album 中完成排序, 删除, 批量归档和 Smart Album 构建, 也需要在 Review 中批量处理 suggestions, 比较历史版本并把审核中的 asset 直接加入相册.

本次变更把 Album 与 Review 作为一个产品闭环推进, 同时保持 Rust core 作为业务事实来源, 避免排序, batch 写入, smart query 和 suggestion history 语义散落在前端.

## What Changes

- 为 Album 增加列表拖拽排序, 重命名, 删除, 手动相册内 asset 拖拽排序, 从相册移除 asset, 以及批量添加多个 assets.
- 将 Smart Album 从简单 JSON allowlist 升级为 typed builder contract, 支持 text, tags, providers, min rating, review status, category, status, created date range 和 sort.
- 为 Review 增加 suggestion 多选, 批量 accept/reject, suggestion history 对比, 字段级 pick/merge 到当前 draft, full suggestion regeneration, 以及从 Review 直接把选中 assets 加入 manual album.
- 为 Review confidence 增加稳定展示模型, 支持 overall 和字段级分数的规范化显示.
- 新增或扩展 core service, Tauri command, desktop state 和 UI 交互, 并保持 batch 写入事务语义.

## Capabilities

### New Capabilities

无. 本次变更扩展现有 Album/Search, Metadata Review 和 Desktop Workbench 能力.

### Modified Capabilities

- `albums-search`: 扩展 manual album 管理, album list ordering, album item ordering, batch add assets, 以及 typed Smart Album query.
- `metadata-review`: 扩展 batch review, suggestion history, full suggestion regeneration, field-level history merge 和 confidence visualization contract.
- `desktop-workbench`: 扩展 Albums 与 Review 视图的交互模型, 包括 drag ordering, Smart Album builder, Review multi-select, batch actions 和 Review 中加入 album.

## Impact

- Rust core:
  - `crates/imglab-core/src/library/schema.rs`
  - `crates/imglab-core/src/library/albums.rs`
  - `crates/imglab-core/src/library/metadata.rs`
  - `crates/imglab-core/src/library/gallery.rs`
  - `crates/imglab-core/src/dto.rs`
  - `crates/imglab-core/src/services.rs`
- Tauri desktop command layer:
  - `apps/desktop/src-tauri/src/lib.rs`
  - existing Codex metadata generation boundary for full suggestion regeneration.
- React desktop app:
  - `apps/desktop/src/main.tsx`
  - `apps/desktop/src/workbench-state.ts`
  - `apps/desktop/src/styles.css`
  - `apps/desktop/tests/workbench-state.test.mjs`
- OpenSpec specs:
  - `openspec/specs/albums-search/spec.md`
  - `openspec/specs/metadata-review/spec.md`
  - `openspec/specs/desktop-workbench/spec.md`
