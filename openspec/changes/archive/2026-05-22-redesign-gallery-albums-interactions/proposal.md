## Why

当前 Gallery 和 Albums 共享 `GalleryQueryState`, 导致 Gallery 的全库浏览语义会被 Albums selected album 污染. 同时 Albums 页面缺少完整的 add-to-album 主流程, Gallery provider filter 仍是 hard-coded toggle, Settings 子页导航也与 Sidebar 信息架构不一致.

本变更将 Gallery 固定为 all-assets browser, 将 Albums 固定为 collection management workspace, 并通过 active-view second-level sidebar 统一 Albums 和 Settings 的二级导航.

## What Changes

- Gallery 默认展示当前 library 下所有普通 image assets, 不再受 Albums selected album 影响.
- Gallery filter 支持 search, provider selector, rating, review, album 单选或多选, 以及 `Not in any album` 特殊筛选.
- Gallery album filter 多选使用 OR / union 语义, `Not in any album` 与具体 album ids 互斥.
- Albums 页面提供完整的 manual album add-to-album 主流程, 包括 add drawer, source query, drawer 内多选和 add selected.
- Albums 页面区分 manual album 与 smart album. Manual album 支持 add, remove, reorder. Smart album 只支持 rule-based matching 和 rule editing.
- Sidebar 改为稳定主 rail 加 active-view second-level context panel. Album items 只在 Albums active 时展示, Settings sections 迁移到 second-level panel.
- Desktop state ownership 拆分为 Gallery query state 和 Albums workspace state, 不再以单个全局 gallery query 同时承载两个页面语义.
- Core Gallery query 引入明确 album filter 语义, 支持 `Any`, `InAny(albumIds)` 和 `Unassigned`.
- `album_order` sort 仅在单个 manual album context 中有效, 其他 album filter context 必须返回 validation error.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: 修改 Sidebar second-level navigation, Gallery filter surface, Albums add drawer, Settings section navigation, 以及 Gallery / Albums state ownership 的可观察行为.
- `albums-search`: 修改 Gallery query album filter 语义, album multi-select union 查询, unassigned 查询, manual-only album mutations, album order validation, 以及 Albums add-to-album read/write workflow.

## Impact

- 影响 desktop React shell 和 workflow modules: `studio-navigation.tsx`, `GalleryWorkspace`, `AlbumsWorkspace`, `SettingsWorkspace`, workflow state helpers, controller hooks, data hooks 和 `StudioAppController`.
- 影响 Tauri query input translation: `query_gallery` 需要表达新的 album filter, 同时可保留 legacy single `albumId` adapter compatibility.
- 影响 Rust core DTO 和 read path: `GalleryQuery`, `library/gallery.rs`, album membership filtering, unassigned filtering 和 sort validation.
- 影响 Albums write path 的 UI guard 和 recoverable error handling, 但不改变 SQLite schema 或 resource library layout.
- 需要新增 core tests, desktop state tests, TypeScript build verification 和 focused UI smoke.
