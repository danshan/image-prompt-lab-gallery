## Context

上一轮 `refactor-maintainability-architecture` 已经完成并归档, 主要解决了 provider planning, task manager 边界, desktop shell 结构和一批代码健康问题. 但当前代码仍有几个明显的长期维护风险:

- `apps/desktop/src/app/App.tsx` 仍承担 root composition, IPC orchestration, workflow state, selection, task queue, review, settings, logs 和 update 状态.
- `apps/desktop/src/app/screens/workflows.tsx` 聚合多个大型 screen 组件, 物理上仍是 workflow mega file.
- `crates/imglab-core/src/library/gallery.rs` 中 gallery query 与 smart album preview 存在相似过滤 pipeline.
- `apps/desktop/src-tauri/src/lib.rs` 和 `crates/imglab-daemon/src/lib.rs` 已经物理拆分文件, 但仍通过 `include!` 注入同一 module scope, module ownership 不够真实.
- 部分测试文件仍按历史入口聚合, 不利于跟随模块重构.

本 change 必须保持用户可见行为稳定. 任何产品语义, daemon API, Tauri command shape, resource library schema 或 migration 都不属于本次范围.

## Goals / Non-Goals

**Goals:**

- 将 desktop screen components 按 workflow 分组, 让 `workflows.tsx` 只保留导出聚合或被删除.
- 将 `App.tsx` 降为 composition root, 通过 focused controller hooks 承载独立 workflow state 和 async orchestration.
- 合并 gallery filtering/sorting 共享逻辑, 降低 query path 与 smart album path 的重复和分支复杂度.
- 将 Tauri backend 和 daemon 的 `include!` 伪拆分升级为真实 `mod` 边界.
- 将测试迁移到对应 module ownership 下, 避免新的 mega test file.

**Non-Goals:**

- 不重新设计 UI 视觉或交互语义.
- 不改变 gallery/smart album 查询结果语义.
- 不改变 daemon endpoint, authentication, response JSON 或 scheduler behavior.
- 不改变 resource library manifest, SQLite schema 或历史 library 兼容策略.
- 不引入新的外部依赖或代码生成工具.

## Decisions

### 1. 先按 ownership 拆分, 不按行数机械切片

大型文件的根因不是行数本身, 而是多个 runtime responsibility 混在同一 review surface. 本次拆分以 ownership 为准:

- `screens/gallery`, `screens/albums`, `screens/review`, `screens/tasks`, `screens/settings`, `screens/inspector`, `screens/chrome`.
- `hooks/useLibraryController`, `hooks/useGalleryController`, `hooks/useTaskController`, `hooks/useReviewController`, `hooks/useSettingsController`.
- Rust backend 以 `commands`, `views`, `paths`, `services`, `runtime`, `routes`, `scheduler`, `executors`, `logs` 等真实 modules 表达.

备选方案是只把文件切成若干 `part-*.tsx` 或继续使用 `include!`. 这能快速降行数, 但不会降低认知复杂度, 也不能阻止后续变更继续堆回入口文件.

### 2. Frontend controller hooks 只抽取 orchestration, 不改变 data contract

Controller hooks 负责持有 workflow state, 调用已有 Tauri transport helpers, 维护 loading/error/action pending 状态, 对 UI 暴露稳定 props. Screen components 保持 presentational 或 workflow-local state. Root `App` 只组合 shell, controller results 和 cross-workflow navigation.

这样可以降低 `App.tsx` 的圈复杂度, 同时避免一次性引入全局 store 或新的状态管理依赖. 备选方案是引入 reducer/store, 但当前状态还未复杂到需要额外框架, 且会增加迁移风险.

### 3. Gallery filtering 使用共享 query spec

`query_gallery` 与 `apply_smart_album_query` 共享一套 predicate helper. 对外仍保留原 public functions 和返回 shape. Smart album query 先被转换为内部 filter spec, 然后复用 text/tags/provider/rating/review/category/time range/sort 规则.

关键约束是不能改变既有查询语义. 因此先覆盖现有 gallery 和 smart album 测试, 再做提取.

### 4. Rust `include!` 改为真实 modules 时优先保持 API surface

Tauri command names, daemon routes 和 public structs 不改名. 对需要跨 module 共享的 helper, 使用 `pub(crate)` 暴露到同 crate, 不扩大 public API. 迁移顺序先建立 `mod` 壳和 re-export, 再逐步收紧 imports.

### 5. 测试跟随模块迁移

测试拆分与代码拆分同批进行. 对纯 UI build-level 保障, 继续依赖 TypeScript build. 对 Rust 行为, 保留现有 unit/integration 覆盖, 只是按 ownership 重排.

## Risks / Trade-offs

- [Risk] 大规模文件移动容易造成 import churn 和 reviewer 负担. → Mitigation: 分阶段提交小块 ownership, 每阶段运行 build/test.
- [Risk] Controller hook 抽取时可能引入 stale closure 或 loading state 漏清理. → Mitigation: 保留原函数调用顺序, 先移动再简化, 用 TypeScript build 和现有 interaction tests 兜底.
- [Risk] Gallery filter 合并可能改变 smart album 边界条件. → Mitigation: 先确认现有 tests, 新增 shared predicate 覆盖 text, tags, provider, rating, review, category, time range 和 sort.
- [Risk] Rust module boundary 迁移可能暴露循环依赖. → Mitigation: 先用 `pub(crate)` helper 解耦, 如果发现真实循环, 优先抽 shared view/helper module, 不回退到 `include!`.
- [Risk] 本次重构不直接带来产品功能收益. → Mitigation: 验收标准以复杂度和 ownership 为准, 明确要求可见行为稳定.
