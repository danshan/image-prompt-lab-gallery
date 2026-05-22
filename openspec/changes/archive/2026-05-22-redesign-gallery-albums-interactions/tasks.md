## 1. Core Gallery Query

- [x] 1.1 扩展 core Gallery query DTO, 表达 explicit album filter: `Any`, `InAny(albumIds)`, `Unassigned`, 并保留 legacy single album id adapter compatibility.
- [x] 1.2 更新 Gallery query validation, 覆盖 empty album ids, `Unassigned` 互斥, unknown album id 和 `album_order` context 校验.
- [x] 1.3 更新 SQLite Gallery read path, 支持 multi-album union 去重和 unassigned assets 查询, 保持 batch preload / projection pipeline.
- [x] 1.4 增加 core tests, 覆盖 default query, single album, multi album union, unassigned, unknown album id 和 invalid album_order.

## 2. Desktop State and Query Adapters

- [x] 2.1 拆分 Gallery query state 与 Albums workspace state, 移除 Albums selected album 对 Gallery query 的直接写入.
- [x] 2.2 更新 desktop query input adapter, 将 Gallery filter 和 Albums add drawer source filter 转换为新的 core album filter.
- [x] 2.3 增加或更新 desktop pure state tests, 覆盖 Gallery album filter 不改变 Albums selection, Albums selection 不改变 Gallery query, add drawer query 独立于 Gallery query.

## 3. Sidebar and Settings Navigation

- [x] 3.1 更新 Sidebar, 保持主 rail 稳定, 并让 context panel 根据 active view 展示 Gallery library context, Albums album list, Settings sections.
- [x] 3.2 将 Settings sections 从 workspace tabs 迁移到 Sidebar second-level context panel 控制, 保持切换 section 不改变当前 library 或 Gallery query.
- [x] 3.3 调整 Sidebar / Settings styles, 保证 normal desktop 和 compact desktop 下二级导航可达且不挤压主操作.

## 4. Gallery Filter Surface

- [x] 4.1 将 Gallery provider filter 改为基于 available providers 的 selector / popover, 移除 hard-coded provider toggle.
- [x] 4.2 增加 Gallery album selector, 支持单选, 多选, `Not in any album`, clear 单项 filter 和 clear all.
- [x] 4.3 更新 Gallery toolbar / filter chips / sort 控制, 在非单 manual album context 下隐藏或禁用 `album_order`.

## 5. Albums Add-To-Album Workflow

- [x] 5.1 更新 Albums workspace, 使用 Albums-owned selected album 和 album contents query 渲染 manual / smart album detail.
- [x] 5.2 实现 manual album `Add images` drawer, 包含独立 source query, drawer 内多选, 默认排除当前 album 已有 assets.
- [x] 5.3 实现 drawer submit, 调用 batch add, 成功后刷新 album contents 和 item count, 失败时保留 selection 并展示 recoverable error.
- [x] 5.4 隐藏 smart album 的 add / remove / reorder affordances, 并保留 smart rule context.

## 6. Verification

- [x] 6.1 运行 OpenSpec change validation, 确认 proposal, design, specs 和 tasks 可用.
- [x] 6.2 运行 Rust focused tests: Gallery / album query 相关 core tests.
- [x] 6.3 运行 desktop focused tests 或 `npm run test`.
- [x] 6.4 运行 build / type verification: `npm run build`, `cargo check -p imglab-core`, `cargo check -p imglab-desktop`.
- [x] 6.5 执行 focused manual smoke 或记录未能执行的 UI smoke 风险.
