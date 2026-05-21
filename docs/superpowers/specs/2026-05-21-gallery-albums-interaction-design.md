# Gallery and Albums Interaction Redesign

Date: 2026-05-21

## Summary

本设计重新划分 Gallery, Albums, 和 Sidebar 的交互职责.

核心结论:

- Gallery 是全库图片浏览器, 默认展示所有普通 assets. Album 只作为 filter 条件, 不作为 Gallery 的导航作用域.
- Albums 是 collection management 工作台, 负责 album 列表, album 内容维护, manual album 的 add / remove / reorder 操作, 以及 smart album 的 rule 管理.
- Sidebar 升级为稳定主 rail 加 active-view second-level context panel. 主 rail 不直接挂 album item, 避免 album 数量增长后主导航失控.
- Gallery 与 Albums 拆分 state ownership. Gallery query 不再被 Albums selected album 污染.

## Context

当前实现中 Gallery 和 Albums 共享同一个 `GalleryQueryState`.

这导致几个问题:

- Gallery 显示范围会受 Albums 页面 selected album 影响, 与 "Gallery 展示所有图片" 的用户预期冲突.
- Albums 页面内容直接复用 `displayedGallery`, 所以 album detail 与全局 Gallery filter 状态耦合.
- Gallery 的 provider filter 是 hard-coded toggle, 不是基于当前 library available providers 的 selector.
- Albums 页面虽然能展示 album contents, 但 "把图片放到 album" 的主路径依赖 Gallery selection, 不足以在 Albums 页面内闭环完成.
- Settings 的 `Libraries / Providers / Updates / Logs` 已经有二级 IA, 但仍放在页面内 tabs, 与 Sidebar 的整体导航模型不一致.

## Goals

- 明确 Gallery 与 Albums 的页面职责, 消除 state 互相污染.
- 支持 Gallery 通过 provider selector, rating, review, album 单选或多选筛选.
- 支持 `Not in any album` 作为 Gallery album filter 的特殊选项.
- 在 Albums 页面内实现完整的 add-to-album 主流程.
- 让 Sidebar 支持 active-view second-level navigation, 包括 Albums 下的 album items 和 Settings 下的 sections.
- 保持当前 local-first, single-user, core-as-source-of-truth 架构.

## Non-Goals

- 不引入云同步, 多用户协作, album sharing, 或权限模型.
- 不做 graph-style lineage visualization.
- 不把 album 设计成全局 route tree 或无限嵌套文件夹.
- 不改变 uploaded reference assets 默认不进入普通 Gallery 查询的既有语义.
- 不在本设计中实现 native provider 或新的 generation provider.

## UX Decisions

### Navigation

Sidebar 采用两层结构, 但不是全局 tree.

主 rail 保持稳定:

- Gallery
- Albums
- Review
- Queue
- Settings

右侧 context panel 根据 active view 切换:

- Gallery active: 显示 library context, status, integrity summary 等 library-level 信息.
- Albums active: 显示 album search, create album, `All albums`, album items.
- Settings active: 显示 `Libraries`, `Providers`, `Updates`, `Logs`.
- Review / Queue active: 保持现有 workflow-level context, 后续可按需要扩展.

Album items 只在 Albums active 时出现在 second-level panel. 点击 album item 会切换到 Albums workspace 并选中该 album, 不改变 Gallery query.

Settings 页面内现有 tabs 应迁移到 second-level panel, 避免同一层级出现重复导航.

### Gallery

Gallery 是 all-assets browser.

Gallery toolbar 提供:

- Search: 搜索 title, prompt, tags, category, provider 等当前 read model 可表达字段.
- Provider: selector / popover, 来源是当前 library 的 available providers.
- Rating: min rating filter.
- Review: `Any` / `Pending`.
- Albums: album 单选或多选 filter.
- Sort: 保留 current sort options, 并只在 album-scoped query 支持 album order.
- Active filter chips: 展示当前 filter, 支持移除单个 filter 和 clear all.

Album filter 语义:

- 默认是 `Any`.
- 选择一个 album 时, 显示属于该 album 的 assets.
- 选择多个 albums 时, 使用 OR / union 语义, 显示属于任意 selected album 的 assets.
- `Not in any album` 是特殊选项, 与具体 album ids 互斥.
- 清空 selected albums 后回到 `Any`.

Gallery 可以保留 selected assets 作为 secondary action 的输入, 例如从 Gallery 快速 add to album. 但这不是 add-to-album 的主路径.

### Albums

Albums 是 collection management workspace.

布局采用:

- Second-level album nav: album search, create album, `All albums`, album list.
- Main panel: selected album header, action bar, content grid.
- Add images drawer: manual album 下打开, 用 all-assets source query 添加图片.

Manual album 支持:

- Add images.
- Remove image from album.
- Reorder album items.
- Rename album.
- Delete album.

Smart album 支持:

- View matching assets.
- Edit smart rules.
- Rename album.
- Delete album.

Smart album 不支持 manual membership mutation, 因此不显示 Add images, Remove, Reorder.

Add images drawer:

- 从 all assets 查询, 不依赖 Gallery 当前 query.
- 支持 search, provider selector, rating, review, album multi-select filter.
- 默认排除已在当前 manual album 中的 assets.
- 支持 drawer 内多选.
- `Add selected to album` 成功后刷新 album contents 和 album count, 清空 drawer selection.
- 当没有 eligible assets 时显示明确 empty state.

## State Ownership

Gallery state 与 Albums state 分离.

Gallery owns:

- `galleryQuery`
- `selectedGalleryAssetId`
- `selectedGalleryAssetIds`
- Gallery grid result
- Gallery filter popover state

Albums owns:

- `selectedAlbumId`
- `albumSearch`
- `albumCreateState`
- `albumContentsQuery`
- `addDrawerOpen`
- `addSourceQuery`
- `addSourceSelection`
- album reorder state

Shared derived data:

- available providers
- available tags
- available categories
- album summaries

Shared derived data 可以从 library read models 派生, 但不应把 page-specific query state 合并回一个全局 query.

## Core Query Model

当前 `GalleryQuery` 只有 `album_id: Option<AlbumId>`. 新设计需要扩展为更明确的 album filter.

建议领域语义:

- `Any`: 不按 album membership 过滤.
- `InAny(Vec<AlbumId>)`: union 语义, asset 属于任意指定 album 即匹配.
- `Unassigned`: asset 不属于任何 album.

实现时可在 DTO / Tauri boundary 保持向后兼容字段, 但内部 service 语义应避免继续把 album filter 表达成单个 optional id.

排序规则:

- `AlbumOrder` 仅在 album filter 能确定一个 manual album 时有效.
- 多 album 或 unassigned filter 不应使用 album order. UI 应隐藏或禁用该 sort, core 必须校验并返回明确 validation error.

## Validation and Error Handling

Gallery filters are read-only. Gallery filter change 不应触发 album membership mutation.

Album mutations 由 Albums workflow 发起:

- add assets to album
- remove asset from album
- reorder album items
- rename album
- delete album

Manual-only guard:

- add / remove / reorder 仅允许 manual album.
- smart album 调用这些 mutation 应返回 domain error.

Album filter validation:

- empty album ids 等价于 `Any`.
- `Unassigned` 与具体 album ids 互斥.
- unknown album id 应返回明确 domain error. UI 应尽量只提交当前 album summaries 中存在的 album ids.

Async feedback:

- Gallery query loading 应显示 lightweight loading state.
- Add drawer mutation pending 时禁用 submit, 保留 selected rows.
- 成功后显示 short status.
- 失败时使用 recoverable error surface, 不清空用户 selection.

## Accessibility and Interaction Quality

设计遵循 dense operational dashboard 风格:

- 默认页面保持高信息密度, 减少不必要 card nesting.
- Icons 用于明确命令, 文字用于需要可读性的 filter / action.
- Clickable rows 和 image cards 需要 hover, focus-visible, keyboard activation.
- Drawer 需要 focus management, Escape close, and return focus.
- Error messages 使用 announced error surface, 不只用颜色表达.
- Images 保持 meaningful alt text.

## Implementation Notes

预期影响范围:

- `apps/desktop/src/studio-navigation.tsx`
- `apps/desktop/src/app/screens/workflows/chrome.tsx`
- `apps/desktop/src/app/screens/gallery/GalleryWorkspace.tsx`
- `apps/desktop/src/app/screens/workflows/albums.tsx`
- `apps/desktop/src/app/workflows/gallery/*`
- `apps/desktop/src/app/workflows/albums/*`
- `apps/desktop/src/app/StudioAppController.tsx`
- `apps/desktop/src/studio-data-hooks.ts`
- `apps/desktop/src/studio-orchestration.ts`
- `crates/imglab-core/src/dto.rs`
- `crates/imglab-core/src/library/gallery.rs`
- `crates/imglab-core/src/library/albums.rs`
- Tauri command input translation for `query_gallery`

Implementation should avoid broad rewrites. The first implementation slice should introduce explicit state/query boundaries, then update UI surfaces, then extend core album filtering.

## Testing Strategy

Core tests:

- Gallery default query returns all non-reference assets.
- `InAny([album_a])` returns assets in album A.
- `InAny([album_a, album_b])` returns union without duplicates.
- `Unassigned` returns assets in no albums.
- `AlbumOrder` is accepted only for one manual album context.
- Manual-only mutation guards reject smart albums.

Desktop state tests:

- Opening an album does not mutate Gallery query.
- Gallery album filter changes do not mutate Albums selected album.
- Gallery provider selector uses available providers.
- Add drawer source query is independent from Gallery query.
- Add drawer excludes already-member assets by default.

Manual smoke:

- Navigate Gallery, apply provider / rating / review / album filters.
- Switch to Albums, select album, open Add images drawer, add assets.
- Return to Gallery and confirm it still uses its own query.
- Switch Settings sections through second-level panel.
- Confirm responsive layout does not overflow at laptop width.

## Deferred Planning Details

No blocking open questions remain from brainstorming.

Deferred details for implementation planning:

- Exact component split for filter popovers.
- Core DTO may accept legacy `album_id` only as adapter compatibility input. Internal service semantics should use explicit album filter.
- Exact visual density of album drawer rows.
