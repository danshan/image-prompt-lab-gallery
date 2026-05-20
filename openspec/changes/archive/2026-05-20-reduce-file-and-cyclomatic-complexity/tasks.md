## 1. Frontend Screen Ownership

- [x] 1.1 拆分 `apps/desktop/src/app/screens/workflows.tsx`, 将 chrome, composer, albums, review, tasks, settings 和 inspector screen 移入 workflow-owned modules.
- [x] 1.2 保留或新增轻量 export barrel, 确保现有 imports 可审阅且不再承载大型实现.
- [x] 1.3 运行 desktop TypeScript build, 修复拆分后的 import/type regressions.

## 2. Frontend Controller Hooks

- [x] 2.1 从 `App.tsx` 抽取 library/settings controller, 覆盖 registered libraries, current library, missing paths 和 library actions.
- [x] 2.2 从 `App.tsx` 抽取 gallery/selection controller, 覆盖 gallery query, selected asset, detail loading, lightbox 和 inspector drawer state.
- [x] 2.3 从 `App.tsx` 抽取 generation/task controller, 覆盖 composer draft, task drafts, queue operations, daemon health, task detail 和 retry/cancel/reorder actions.
- [x] 2.4 从 `App.tsx` 抽取 review/logs/update controller 或等价 focused controllers, 降低 root component state 和 async branch 数量.
- [x] 2.5 运行 desktop tests/build, 确认 root component 仍能组合 Gallery, Albums, Review, Queue, Settings 和 Inspector.

## 3. Core Gallery Filtering

- [x] 3.1 在 `crates/imglab-core/src/library/gallery.rs` 中提取共享 gallery filter spec, predicate helpers 和 sort helper.
- [x] 3.2 将 `query_gallery` 和 smart album preview path 迁移到共享 filtering pipeline, 保持现有结果语义.
- [x] 3.3 补充或迁移 Rust tests, 覆盖 text, tags, provider, rating, review pending, category, time range 和 sort 行为.

## 4. Rust Module Boundaries

- [x] 4.1 将 `apps/desktop/src-tauri/src/lib.rs` 的 root-level `include!` 迁移为真实 Rust modules, 保持 Tauri command names 和 view shapes.
- [x] 4.2 将 `crates/imglab-daemon/src/lib.rs` 的 root-level `include!` 迁移为真实 Rust modules, 保持 daemon endpoint, auth 和 response JSON 兼容.
- [x] 4.3 拆分或迁移相关大型 test files, 让 tests 跟随 module ownership.

## 5. Verification And OpenSpec Closeout

- [x] 5.1 运行 `openspec validate reduce-file-and-cyclomatic-complexity --strict`.
- [x] 5.2 运行 `cd apps/desktop && npm run test && npm run build`.
- [x] 5.3 运行 `cargo fmt --all --check` 和 affected Rust crate tests/checks.
- [x] 5.4 运行 large-file/ownership scan, 记录剩余 mega files 或明确 deferred reason.
- [x] 5.5 更新 `tasks.md` checkbox 状态, 并汇总验证结果和剩余风险.

### Notes

- `apps/desktop/src-tauri/src/lib.rs` 已移除 root-level `include!`, 通过 `commands`, `errors`, `paths`, `services`, `view_mappers` 和 `views` 真实 modules 组织 Tauri backend. Test-only include 也已改为 `mod tests`.
- `crates/imglab-daemon/src/lib.rs` 已移除 root-level `include!`, 保留原 crate-level public exports, 并通过 module-local imports 与 `pub(crate)` helper/DTO 边界保持 endpoint, auth 和 JSON response 兼容.
- `crates/imglab-daemon/src/tests.rs` 已拆为 shared helper shell 和 `tests/api.rs`, `tests/scheduler.rs`, `tests/recovery.rs`, 让 daemon API, scheduler 和 recovery tests 跟随 module ownership.
- Large-file scan 仍显示历史 core files 较大: `crates/imglab-core/src/library/tests.rs` 2692 行, `apps/desktop/src/app/App.tsx` 1289 行, `crates/imglab-core/src/library/gallery.rs` 1226 行, `crates/imglab-core/src/library/tasks.rs` 1026 行. 本 change 已降低 workflow screen, controller, daemon test 和 include boundary 的主要复杂度, 后续若继续压缩应另开针对 core library ownership 的 change, 避免在同一轮里重排持久化/任务核心测试造成 review 风险.
- Verification completed: `npm run test`, `npm run build`, `cargo fmt --all --check`, `cargo check -p imglab-core`, `cargo check -p imglab-daemon`, `cargo check -p imglab-desktop`, `cargo test -p imglab-core gallery -- --nocapture`, `cargo test -p imglab-daemon --lib`, `cargo test -p imglab-desktop --lib`.
