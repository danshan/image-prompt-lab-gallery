## Context

当前桌面端已经有 Gallery, Albums, Settings 和 Sidebar, 但 Gallery 与 Albums 共享一个 `GalleryQueryState`. Albums 打开 album 时会写入 query 的 album context, 而 Gallery 又复用同一个 query 展示 grid. 这让 Gallery 的 all-assets 浏览语义和 Albums 的 collection management 语义混在一起.

当前 Albums 页面也没有完整的添加图片主路径. 用户主要依赖 Gallery selected assets, 再回到 Albums 批量添加. 这对 collection management 来说是反向流程. Settings 已经有 `Libraries / Providers / Updates / Logs` 子区, 但这些子区仍在 workspace 内部 tabs, 与 Sidebar 作为应用导航入口的职责不一致.

本设计以已确认的 `docs/superpowers/specs/2026-05-21-gallery-albums-interaction-design.md` 为产品输入, 将其落到 OpenSpec change 中.

## Goals / Non-Goals

**Goals:**

- Gallery 固定为 all-assets browser, album 只作为 filter 条件.
- Albums 固定为 collection management workspace, 在页面内完成 manual album 添加图片主流程.
- Sidebar 支持稳定主 rail 和 active-view second-level context panel.
- Gallery query 支持 provider selector, rating, review, album 单选或多选, 以及 unassigned assets.
- Gallery 与 Albums 拥有独立 page state, 避免 selected album 污染 Gallery query.
- Core Gallery query 使用明确 album filter 语义, 支持 `Any`, `InAny(albumIds)` 和 `Unassigned`.

**Non-Goals:**

- 不引入全局 album tree route.
- 不引入 album folder nesting, cloud sync, sharing 或 permissions.
- 不修改 SQLite schema 或 resource library layout.
- 不改变 reference assets 默认不进入普通 Gallery 查询的语义.
- 不重新设计 Review, Queue 或 Inspector 的主工作流.

## Decisions

### 1. Sidebar 使用 active-view second-level context panel

主 rail 保持 `Gallery / Albums / Review / Queue / Settings`. 右侧 context panel 按 active view 切换:

- Gallery active 时显示 library context 和 status.
- Albums active 时显示 album search, create album, `All albums` 和 album items.
- Settings active 时显示 `Libraries / Providers / Updates / Logs`.

替代方案是把 album items 和 settings sections 直接作为全局 tree 子节点. 该方案可达性强, 但 album 数量会让主导航持续增长, 也会让 Gallery / Albums 的职责再次混淆. 因此选择 active-view second level.

### 2. Gallery query 与 Albums workspace state 分离

Gallery owns `galleryQuery`, Gallery selected ids 和 Gallery result. Albums owns `selectedAlbumId`, album search, album contents query, add drawer state 和 add drawer selection.

替代方案是继续复用一个 query, 只修补 album filter. 这能减少改动, 但无法保证 Gallery 回到页面后仍保持 all-assets 语义. 因此拆分 state ownership.

### 3. Gallery album filter 使用显式 union 语义

Core 使用明确 album filter:

- `Any`: 不按 album membership 过滤.
- `InAny(albumIds)`: 返回属于任意 selected album 的 assets, 去重.
- `Unassigned`: 返回不属于任何 album 的 assets.

`Unassigned` 与具体 album ids 互斥. 空 album ids 等价于 `Any`.

替代方案是把 multi-select 解释为 intersection. 这更像高级筛选, 但对 album curation 的常见目标不直观, 且会让 UI 解释成本更高. 因此选择 union.

### 4. Albums add-to-album 主流程在 Add images drawer 中完成

Manual album header 提供 `Add images`. Drawer 内使用 all-assets source query, 支持 search, provider, rating, review 和 album filters, 默认排除已在当前 manual album 中的 assets. Drawer 内多选后执行 `Add selected to album`.

替代方案是常驻第三列 source pane. 这会压缩 album contents grid, 尤其在 laptop width 下风险更高. Drawer 只在添加流程中出现, 默认页面更稳定.

### 5. Smart album 不暴露 manual membership 操作

Smart album 显示 matching assets 和 rule editing, 不显示 add, remove 或 reorder. Core 继续作为 manual-only guard 的事实来源, UI 只做 affordance guard.

### 6. `album_order` 只允许单个 manual album context

当 query 是 `Any`, `Unassigned` 或 multi album union 时, `album_order` 没有唯一排序来源. UI 必须隐藏或禁用该 sort, core 必须返回 validation error, 不做 fallback sort.

## Risks / Trade-offs

- [Risk] State 拆分会触碰 `StudioAppController` 及多个 workflow props. → Mitigation: 先新增 pure state helpers 和 query adapter, 再逐步迁移 screen props, 保持每一步可测试.
- [Risk] Core query DTO 变化可能影响现有 Tauri 和 CLI caller. → Mitigation: adapter 层可接受 legacy `albumId` 作为单 album compatibility input, 但 internal service 使用 explicit album filter.
- [Risk] Add drawer 与 Gallery filter 有重复 UI. → Mitigation: 抽取可复用 filter helper / presentational control, 但保持 Gallery 和 Add drawer 各自拥有 query state.
- [Risk] Multi-album union 查询可能引入 duplicate rows 或 SQL N+1. → Mitigation: core tests 覆盖 union 去重, read path 继续复用 batch preload / projection pipeline.
- [Risk] Second-level context panel 可能挤压 workspace. → Mitigation: 复用现有 `Library Context` 区域, compact desktop 下继续允许折叠或 drawer 化.

## Migration Plan

1. 扩展 core DTO 和 gallery read path, 加入 explicit album filter, union, unassigned 和 sort validation.
2. 拆分 desktop Gallery query state 和 Albums workspace state, 更新 Tauri query input adapter.
3. 更新 Sidebar context panel, 让 Albums 和 Settings 使用 second-level navigation.
4. 更新 Gallery toolbar filters, 将 provider 改为 selector, album 改为 multi-select filter.
5. 更新 Albums workspace 和 Add images drawer, 将 add-to-album 主流程放入 Albums 页面.
6. 运行 core tests, frontend state tests, `npm run test`, `npm run build`, `cargo check -p imglab-core`, `cargo check -p imglab-desktop`, 以及 focused manual smoke.

Rollback strategy: 该变更不修改持久化 schema. 若 UI 回归, 可回退 desktop state/UI changes. 若 core query adapter 有问题, legacy `albumId` compatibility input 可继续用于单 album 查询路径.

## Open Questions

无 blocking open questions. 实现时仅需确定 filter popover 的组件拆分粒度和 drawer row 的最终视觉密度.
