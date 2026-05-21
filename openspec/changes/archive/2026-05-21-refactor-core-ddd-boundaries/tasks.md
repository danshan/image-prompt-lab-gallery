## 1. Baseline 与边界准备

- [x] 1.1 记录当前 `imglab-core`, CLI, daemon, Tauri backend 和 desktop frontend 的主要 public contracts, 包括 CLI JSON, daemon endpoint, desktop command view, provider behavior 和 resource library compatibility.
- [x] 1.2 运行或记录 baseline 验证命令结果: `cargo fmt --all --check`, `cargo test -p imglab-core`, `cargo test -p imglab-daemon`, `cargo test -p imglab-cli`, `npm test --prefix apps/desktop`, `npm run build --prefix apps/desktop`.
- [x] 1.3 建立 `imglab-core` DDD module skeleton: `domain`, `application`, `application/ports`, `application/use_cases`, `application/read_models`, `infrastructure`, `interface_contracts`.
- [x] 1.4 添加短期 compatibility exports, 保证迁移期间 downstream crates 可以逐步改 import, 且新增代码优先使用新 boundary.
- [x] 1.5 建立 architecture check 初版, 至少能报告 domain modules 是否导入 `rusqlite`, `std::fs`, daemon, Tauri, CLI parser 或 desktop view 类型.

## 2. Domain Model 迁移

- [x] 2.1 将 shared identity 和 value objects 从 shared DTO namespace 迁移到 `domain/shared`, 包括 library, asset, version, generation event, metadata suggestion, album 和 task identifiers.
- [x] 2.2 建立 `domain/library`, 表达 resource library identity, manifest summary, schema compatibility 和 registry alias 语义.
- [x] 2.3 建立 `domain/asset`, 表达 `Asset`, `AssetVersion`, version number, version name, parent chain, checksum metadata, MIME metadata 和 reference source rule.
- [x] 2.4 将 asset version number 分配, same-asset parent validation, cross-asset reference source 判断抽为可复用 policy 或 domain service.
- [x] 2.5 建立 `domain/generation`, 表达 generation operation, provider capability, generation request policy, generation event status 和 input/output relation.
- [x] 2.6 建立 `domain/metadata_review`, 表达 suggestion status, accept/reject/batch review invariant 和 confidence normalization rule.
- [x] 2.7 建立 `domain/album`, 表达 manual album, smart album, album order, smart query validation 和 album-scoped query rule.
- [x] 2.8 建立 `domain/task`, 表达 task status machine, attempt lifecycle, output link, retry policy 和 scheduler selection policy.
- [x] 2.9 为每个 domain 添加 focused unit tests, 覆盖 invariant, transition 和 reusable policy, 避免依赖 SQLite 或真实 filesystem.
- [x] 2.10 对每个 domain 做 context-local complexity review, 拆分高复杂度函数, 合并重复 rule, 记录仍需保留的复杂点和原因.

## 3. Application Ports 与 Use Cases

- [x] 3.1 定义 repository ports: `LibraryRepository`, `AssetRepository`, `GenerationEventRepository`, `MetadataSuggestionRepository`, `AlbumRepository`, `TaskRepository`.
- [x] 3.2 定义 infrastructure-facing ports: `ManagedFileStore`, `LibraryRegistry`, `ImageProvider`, `Clock`, `IdGenerator`, `TransactionManager`.
- [x] 3.3 实现 `ImportAssetUseCase`, 让 import orchestration 依赖 domain policy 和 ports, 不直接执行 SQL 或 filesystem copy.
- [x] 3.4 实现 `CreateChildVersionUseCase`, 让 child version 创建复用 asset aggregate rule 和 transaction boundary.
- [x] 3.5 实现 `GenerateImageUseCase`, 覆盖 text-to-image, same-asset image-to-image, uploaded reference asset, generation event, metadata suggestion 和 provider failure recording.
- [x] 3.6 实现 metadata review use cases, 覆盖 create suggestion, accept, reject, batch accept/reject, history 和 review draft detail.
- [x] 3.7 实现 album/search/gallery use cases, 保持 gallery query, smart album preview 和 search predicate 复用, 避免重复过滤逻辑.
- [x] 3.8 实现 task use cases, 覆盖 create/list/detail/status update, attempt/event/output append, retry, duplicate 和 reorder.
- [x] 3.9 建立 `ImgLabApplication` 或等价 application facade, 将 library, assets, generation, metadata review, albums, gallery 和 tasks 暴露为稳定应用入口.
- [x] 3.10 添加 fake/in-memory ports 用于 application tests, 覆盖主要 write flows 和 domain rule 组合行为.

## 4. Infrastructure Adapter 迁移

- [x] 4.1 将 SQLite schema source 按 context 拆分到 `infrastructure/sqlite`, 保留统一 migration entrypoint 和当前 schema version 行为.
- [x] 4.2 将 asset/version SQLite queries 和 persistence 实现迁移为 repository adapter, repository 不再分配 version number 或解释 reference source 语义.
- [x] 4.3 将 generation event, metadata suggestion, album/search/gallery 和 task SQLite persistence 分别迁移到对应 adapter modules.
- [x] 4.4 将 managed filesystem behavior 迁移到 `ManagedFileStore` adapter, 包括 date bucket path, temp write, checksum, image dimension parsing 和 MIME/extension policy.
- [x] 4.5 将 manifest, library lifecycle, registry alias, unregister, backup import/export 和 repair/integrity 实现迁移到 infrastructure modules.
- [x] 4.6 将 provider adapters 对齐新的 `ImageProvider` port, 保持 fake provider, Codex CLI provider 和 Grok boundary 行为兼容.
- [x] 4.7 实现 SQLite-backed composition root, 装配 repositories, file store, registry, providers, clock, id generator 和 transaction manager.
- [x] 4.8 迁移 infrastructure tests, 覆盖 SQLite migration/backfill, repository persistence, filesystem managed writes, backup/restore 和 compatibility open flow.

## 5. Runtime Integration 迁移

- [x] 5.1 将 `imglab-cli` 改为调用 application facade 或 use cases, 保持 existing CLI commands 和 JSON output shape 不变.
- [x] 5.2 将 `imglab-daemon` runtime state 从直接持有 concrete local service 收敛为持有 application facade 和 daemon-owned runtime state.
- [x] 5.3 将 daemon route handlers 改为通过 application facade 执行 library open, task create/list/detail, reorder, retry, duplicate 和 log-related task lookup.
- [x] 5.4 将 daemon scheduler/executor 改为调用 task/generation/metadata use cases, 不在 runtime 层重复 generation planning 或 task state rule.
- [x] 5.5 将 `apps/desktop/src-tauri` commands 改为调用 application facade, 保持 command input/output view 和 error mapping 兼容.
- [x] 5.6 将 Tauri view mappers 与 core domain/application model 显式分离, runtime-specific view 不回流到 domain.
- [x] 5.7 更新 provider crate imports, 移除对旧 DTO namespace 的 primary dependency, 保持 compatibility exports 只用于必要过渡.
- [x] 5.8 运行 CLI, daemon 和 Tauri focused tests, 修复 runtime integration regressions.

## 6. Desktop Frontend Ownership 收敛

- [x] 6.1 将 `App.tsx` 收敛为 shell composition 和 top-level controller wiring, 移除 workflow-specific state machine 和 transport orchestration 细节.
- [x] 6.2 将 Gallery workflow 的 controller, pure state, derived state, screen props 和 Tauri adapter mapping 收敛到 gallery-owned modules.
- [x] 6.3 将 Albums workflow 的 controller, smart/manual album state, asset board context 和 query mapping 收敛到 albums-owned modules.
- [x] 6.4 将 Review workflow 的 controller, review draft state, generation task handoff 和 suggestion mapping 收敛到 review-owned modules.
- [x] 6.5 将 Queue/Task workflow 的 controller, task detail, attempt logs, reorder/retry/duplicate actions 和 task output links 收敛到 task-owned modules.
- [x] 6.6 将 Settings/Libraries/Logs workflow 的 controller, diagnostics, library management actions 和 log reads 收敛到 settings-owned modules.
- [x] 6.7 拆分 `workbench-state.ts`, 将 pure state helpers 移到 workflow-owned state modules, 保持 Node tests 覆盖关键 state transitions.
- [x] 6.8 运行 desktop tests/build, 并手工检查 960px compact desktop 下主要 workflow 操作仍可达.

## 7. Compatibility Cleanup 与文档

- [x] 7.1 清理不再需要的 old shared DTO primary imports, 让新代码通过 domain/application/interface contracts boundary import.
- [x] 7.2 删除或降级旧 compatibility exports, 只保留确实需要的 public compatibility surface.
- [x] 7.3 更新 `docs/development.md` module map, 明确 DDD 分层, composition root, runtime integration 和验证命令.
- [x] 7.4 更新 `README.md` 或相关 provider docs 中已经过时的 core module 描述.
- [x] 7.5 为 architecture checks 增加文档说明, 包括如何运行, 报告哪些依赖方向和复杂度风险.
- [x] 7.6 更新 OpenSpec task notes, 标明 public behavior, persisted schema 和 user-visible behavior 是否保持兼容.

### 7.x Compatibility Notes

- Public behavior: CLI commands, daemon routes, Tauri commands, desktop workflow reachability and provider names are intended to remain compatible. Runtime layers now enter core through application facade or explicit interface contracts.
- Persisted schema: The SQLite schema entrypoint and existing resource library layout are preserved. Repository adapters no longer allocate asset version numbers, but they persist caller-provided `version_number`, parent version, generation event, MIME and managed file metadata.
- User-visible behavior: Asset version labels remain asset-scoped numeric versions such as `v1`, `v2`. Imported first versions are assigned by application/domain as `version_number = 1`; generated child versions are assigned from the application-visible version set and validate that a same-asset parent belongs to the target asset.
- Compatibility surface: Root-level `imglab-core` compatibility exports are downgraded to `#[doc(hidden)]` legacy exports. Legacy modules remain public for compatibility, but new runtime and provider code should import from `domain`, `application`, `infrastructure`, or `interface_contracts` boundaries instead of the old shared DTO namespace.

## 8. Verification 与收尾

- [x] 8.1 运行 architecture dependency check, 确认 domain 不依赖 infrastructure/runtime, runtime 不绕过 application boundary 复刻已迁移业务规则.
- [x] 8.2 运行 complexity/ownership scan, 报告 core, daemon 和 desktop 剩余大文件, 高复杂度函数和重复规则, 并修复本 change 范围内必须处理的问题.
- [x] 8.3 运行 `cargo fmt --all --check`.
- [x] 8.4 运行 `cargo test -p imglab-core`.
- [x] 8.5 运行 `cargo test -p imglab-cli`.
- [x] 8.6 运行 `cargo test -p imglab-daemon`.
- [x] 8.7 运行 provider crate focused tests 或 `cargo test -p imglab-provider-codex -p imglab-provider-grok`.
- [x] 8.8 运行 `npm test --prefix apps/desktop`.
- [x] 8.9 运行 `npm run build --prefix apps/desktop`.
- [x] 8.10 验证 existing resource library compatibility, 至少覆盖 open, gallery query, import/generate smoke path 或等价 fixture flow.
- [x] 8.11 验证 CLI JSON, daemon API response shape 和 Tauri command view 未发生非预期变化.
- [x] 8.12 运行 `openspec validate refactor-core-ddd-boundaries --strict`.
- [x] 8.13 完成最终 code review, 确认 DDD boundary, low duplication, low cyclomatic complexity 和 compatibility requirements 均已满足.

### 8.x Verification Notes

- Architecture dependency check: `scripts/check-architecture.sh` passed.
- Ownership scan: largest remaining files are legacy compatibility services/tests, `StudioAppController.tsx`, and workflow screen/controller modules. `crates/imglab-core/src/library/tests.rs` remains as compatibility and cross-context regression coverage, while migrated reusable rules have owner-local focused tests under domain/application/infrastructure modules. No domain dependency reversal, old desktop `workbench-state` primary imports, or SQLite-side asset version allocation was found in the scan.
- CLI smoke: isolated temporary registry and library covered `init`, `import`, `search`, `generate --provider fake`, and post-generate `search`. Import first version and generated first version both returned asset-scoped `version_number = 1`.
- API/view compatibility: CLI JSON smoke output remained stable; daemon API response shape and Tauri command view coverage passed through focused Rust tests.
- OpenSpec validation: `openspec validate refactor-core-ddd-boundaries --strict` returned success. The trailing PostHog network flush error is telemetry noise and exited with code 0.
- Final review: `git diff --check` passed. Remaining root-level compatibility exports are `#[doc(hidden)]`, while legacy modules remain public for compatibility. New runtime/provider paths use application, infrastructure composition, or interface-contract boundaries. Follow-up review tightened the application facade so the `assets` owner is an application use-case owner instead of the legacy local service, removed duplicated asset use-case rules, and expanded architecture checks to cover application, runtime rule bypasses, and desktop source imports from the compatibility state barrel. No final review blocker was found.
