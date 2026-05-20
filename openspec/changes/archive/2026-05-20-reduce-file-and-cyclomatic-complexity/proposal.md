## Why

当前上一轮重构已经把一批业务边界从入口文件中移出, 但 desktop 前端和部分 Rust 边界仍存在单文件过长, 状态编排集中, 分支逻辑重复的问题. 这些问题会继续抬高后续 Gallery, Albums, Review, Queue, Settings 和 daemon 迭代的修改成本, 也会让局部行为变更难以验证.

本次 change 的目标是继续做行为稳定的结构性重构, 优先降低单文件复杂度和圈复杂度, 让主要 workflow, controller, filtering pipeline 和 backend module boundary 都能按职责维护.

## What Changes

- 拆分 `apps/desktop/src/app/screens/workflows.tsx`, 让 Gallery/Albums/Review/Queue/Settings/Inspector 等屏幕组件进入按 workflow 分组的文件.
- 从 `apps/desktop/src/app/App.tsx` 抽取局部 controller hooks, 将 library, gallery/selection, generation composer, task queue, review, logs/update 等状态编排从根组件中下沉.
- 将 gallery filtering/sorting 的重复 pipeline 合并为共享 query specification 和 predicate helper, 避免 `query_gallery` 与 smart album preview 各自维护相似逻辑.
- 将 `apps/desktop/src-tauri/src/lib.rs` 和 `crates/imglab-daemon/src/lib.rs` 中的 `include!` 物理拆分升级为真实 Rust module boundary, 保留现有 public command/API shape.
- 拆分过大的测试文件, 让测试跟随被测模块或 workflow ownership.
- 不引入可见产品行为变化, 不修改 resource library 持久化格式, 不改变 desktop/CLI/daemon 对外协议.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `desktop-workbench`: 明确 desktop root component 和 workflow screen 文件的职责边界, 防止新的 workflow 继续堆回单一巨型文件.
- `performance-code-health`: 增加可维护性 guardrail, 约束单文件复杂度, controller hook 分层, 共享 filtering pipeline 和测试 ownership.
- `task-manager-daemon`: 明确 daemon 真实 module boundary, 避免 `include!` 维持伪拆分.

## Impact

- Affected frontend files:
  - `apps/desktop/src/app/App.tsx`
  - `apps/desktop/src/app/screens/workflows.tsx`
  - `apps/desktop/src/app/screens/**`
  - `apps/desktop/src/app/hooks/**`
- Affected Rust files:
  - `crates/imglab-core/src/library/gallery.rs`
  - `apps/desktop/src-tauri/src/lib.rs`
  - `apps/desktop/src-tauri/src/**`
  - `crates/imglab-daemon/src/lib.rs`
  - `crates/imglab-daemon/src/**`
- Verification:
  - OpenSpec validation for the active change and main specs.
  - Desktop TypeScript build/tests.
  - Rust fmt/check/tests for affected crates.
