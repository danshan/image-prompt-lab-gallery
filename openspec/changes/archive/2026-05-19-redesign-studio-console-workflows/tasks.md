## 1. Spec Artifacts

- [x] 1.1 更新 `desktop-workbench` delta, 覆盖 Studio Console shell, visual system, workflow behavior, read model usage, responsive 和 component boundaries.
- [x] 1.2 更新 `metadata-review` delta, 覆盖 staged review draft, generated result apply/ignore, stale result, history compare 和 canonical-write-only-on-accept.
- [x] 1.3 更新 `image-generation` delta, 覆盖 image task output links, asset version, pending suggestion 和 cross-workflow navigation.
- [x] 1.4 更新 `albums-search` delta, 覆盖 Albums collection management, smart rule preview, album-scoped asset board 和 album context.
- [x] 1.5 更新 `app-logs` delta, 覆盖 Settings diagnostics, task log deep links 和 Settings/Task Detail 职责边界.
- [x] 1.6 更新 `resource-library` delta, 覆盖 Studio Overview 所需 library/provider status read model.

## 2. Studio Read Models and Commands

- [x] 2.1 在 Rust core/service boundary 定义 `StudioOverview` read model, 包含 library summary, storage/integrity, review count, active task summary 和 provider health summary.
- [x] 2.2 扩展 Gallery read model 或新增 `AssetBoardItem`, 包含 current version, version count, review state, provider/model, task origin 和 album context.
- [x] 2.3 定义 `AssetInspectorDetail`, 显式区分 canonical metadata, pending suggestion summary, generated task origin, album memberships, file integrity 和 version lineage.
- [x] 2.4 定义 `ReviewDraftDetail` read model, 包含 suggestion, draft seed, confidence, history, generated field results, related tasks 和 asset context.
- [x] 2.5 扩展 `TaskDetail` read model, 包含 output asset/version/generation event/review links, attempt logs, timeline 和 related review/asset targets.
- [x] 2.6 定义 `DiagnosticsOverview` 或等价 diagnostics payload, 覆盖 provider health, daemon status, app-owned logs 和 library lifecycle status.
- [x] 2.7 为所有新增或修改 Tauri command payload 增加稳定 JSON serialization tests 或 core read model tests.

## 3. Frontend Architecture Split

- [x] 3.1 将 `main.tsx` 收敛为 bootstrap 和 top-level orchestration, 不再承载所有 workflow rendering.
- [x] 3.2 新增 shell components: `AppShell`, `StudioRail`, `LibraryContextPanel`, `WorkspaceFrame`, `InspectorFrame`, `ActivityStrip`.
- [x] 3.3 新增 workflow components: `GalleryWorkspace`, `AlbumsWorkspace`, `ReviewWorkspace`, `QueueWorkspace`, `SettingsWorkspace`.
- [x] 3.4 拆分 data hooks 或 orchestration helpers: library registry, studio overview, gallery query, review drafts, task queue 和 diagnostics.
- [x] 3.5 保留并扩展 pure state helper tests, 覆盖 workflow navigation, selected asset, review draft, queue selection 和 compact shell state.

## 4. Visual System and Responsive Shell

- [x] 4.1 引入 Studio Console design tokens: graphite chrome, warm ivory canvas, cobalt secondary action, vermilion primary generation action, lime/amber/red/green status colors.
- [x] 4.2 实现 `Studio Rail | Library Context | Workspace | Inspector | Activity Strip` shell, 保持 normal desktop 和 compact desktop 可用.
- [x] 4.3 实现统一 icon affordance, focus styles, status pills, panels, asset tiles, list-detail panels 和 command surfaces.
- [x] 4.4 在 `1440px` 和 `960px` 下验证 shell 不出现不可恢复覆盖, 主操作不可达或文本重叠.

## 5. Gallery and Inspector Workflow

- [x] 5.1 将 Gallery 改为 image-first asset board, 展示 version, review state, provider/model, task origin 和 album context.
- [x] 5.2 实现 Gallery query/filter/sort command surface, 不遮蔽 asset board 主任务.
- [x] 5.3 将 Inspector 改为 selected asset command surface, 展示 prompt, file integrity, album memberships, version lineage, pending review state, task origin 和 variation entry.
- [x] 5.4 实现 Inspector 到 Generate variation, Add to album, Open pending review 和 Open source task 的 deep links.
- [x] 5.5 覆盖 Gallery 和 Inspector 的 loading, empty, error 和 recovery states.

## 6. Albums Workflow

- [x] 6.1 将 Albums 改为 collection management workspace, 区分 manual albums 和 smart albums.
- [x] 6.2 实现 manual album list ordering, album-scoped asset board, item ordering, remove asset, batch add selected assets, rename 和 delete.
- [x] 6.3 实现 smart album rule builder 和 core-backed live preview, 不在前端猜测 smart query 语义.
- [x] 6.4 在 album-scoped asset board 中展示 album context, review state 和 version summary.
- [x] 6.5 覆盖 Albums 的 loading, empty, error 和 recovery states.

## 7. Review Workflow

- [x] 7.1 将 Review 改为 staged metadata workbench, 使用 `ReviewDraftDetail` read model seed 本地 draft.
- [x] 7.2 显示 confidence, history compare, generated result available, field apply/ignore 和 full suggestion regeneration.
- [x] 7.3 保证 accept 前 pending suggestion, generated result 和 local draft 不写入 canonical metadata.
- [x] 7.4 实现 stale generated result guard, 用户后续编辑不得被异步 field result 静默覆盖.
- [x] 7.5 实现 Review 到 source task, asset, suggestion history 和 affected album 的 deep links.
- [x] 7.6 覆盖 Review 的 loading, empty, error, recovery, accept failure 和 batch transaction failure states.

## 8. Queue Workflow

- [x] 8.1 将 Queue 改为 operations console, 包含 batch composer, task queue 和 task detail.
- [x] 8.2 展示 running, queued, retry waiting, completed, failed, canceled tasks, 并只展示符合状态的操作.
- [x] 8.3 Task Detail 展示 attempts, timeline, log tail, outputs, asset/version/review links 和 error classification.
- [x] 8.4 实现 Task Detail 到 asset, output version, review suggestion 和 attempt log 的 deep links.
- [x] 8.5 覆盖 Queue 的 loading, empty, error, recovery 和 daemon offline states.

## 9. Settings and Diagnostics

- [x] 9.1 将 Settings 聚焦为 Libraries, Providers/Diagnostics 和 Logs 子区.
- [x] 9.2 Libraries 保留 create, open existing, import zip, switch, rename, close, export zip 和 reveal 行为.
- [x] 9.3 Provider diagnostics 展示 provider health, credential/capability status 和可恢复配置错误.
- [x] 9.4 Logs 保留 global app-owned log browser, 但 task debugging deep-link 到 Task Detail.
- [x] 9.5 覆盖 Settings 的 loading, empty, error, recovery 和 missing library path states.

## 10. Verification

- [x] 10.1 运行 Rust core tests, 覆盖新增 read models, review draft invariants, task output links 和 smart album preview.
- [x] 10.2 运行 desktop frontend state tests, 覆盖 workflow transitions, review draft apply/ignore, queue selection 和 shell state.
- [x] 10.3 运行 `npm test` 和 `npm run build` 验证 TypeScript/Vite.
- [x] 10.4 运行 `cargo test --workspace` 或当前项目等价 Rust test set.
- [x] 10.5 使用 desktop dev server 或 Tauri dev 在 `1440px` 和 `960px` 下完成 Gallery, Albums, Review, Queue, Settings, Inspector 视觉 QA.
- [x] 10.6 手动 smoke: image generation task completed -> asset version created -> pending review -> review accepted -> canonical metadata updated.
- [x] 10.7 检查 visual demo 是否保留为 reference 或移动到 docs, 不让 demo-only HTML 进入 production bundle.
