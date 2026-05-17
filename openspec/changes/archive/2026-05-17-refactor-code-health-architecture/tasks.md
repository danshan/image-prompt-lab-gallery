## 1. Checksum 语义切换

- [x] 1.1 将 core 新导入 asset version 的 checksum 写入从 MD5 切换为 SHA-256, 并确保 `sha256`, `checksum_algorithm`, `checksum` 写入一致.
- [x] 1.2 将 generation output 和 child version 创建路径的 checksum 写入切换为 SHA-256.
- [x] 1.3 更新 repair dry-run, 使其识别 checksum algorithm 或 checksum 不符合当前 SHA-256 标准的 rows.
- [x] 1.4 更新 repair apply, 使其重新计算 SHA-256 并同步更新 `sha256`, `checksum_algorithm`, `checksum`.
- [x] 1.5 保持 integrity check 按 row 自身 `checksum_algorithm` 校验, 并覆盖 legacy MD5 row 仍可读取的测试.

## 2. Public DTO 和 Transport Output 收敛

- [x] 2.1 从 core public version DTO 的业务语义中移除或弱化 `sha256`, 以 `checksum_algorithm` 和 `checksum` 作为 canonical fields.
- [x] 2.2 更新 CLI import, generate 和其他 version JSON output, 不再暴露误导性 `sha256` 业务字段.
- [x] 2.3 更新 Tauri `VersionView` 和 frontend `Version` type, 使用 canonical checksum fields.
- [x] 2.4 更新 CLI, desktop 和 core tests 中对 `sha256` 字段的断言.

## 3. Desktop Core Read Model 边界

- [x] 3.1 移除 Tauri `gallery_items` command, `GalleryItemsInput` 和 `GalleryItemView`.
- [x] 3.2 从 Tauri invoke handler 中取消注册 `gallery_items`.
- [x] 3.3 确认真实 Tauri Gallery 只调用 `query_gallery`, Inspector 只调用 `get_asset_detail`.
- [x] 3.4 保留 browser preview/mock 本地 filtering, 但确保该路径不影响真实 Tauri mode.

## 4. Generation Orchestration 收敛

- [x] 4.1 在 core 中提取共享 generation request builder, 覆盖 provider name normalization, operation inference 和 default model label.
- [x] 4.2 提取共享 input image loading 逻辑, 支持 explicit input file 和 library version input.
- [x] 4.3 让 CLI generation command 使用共享 builder 和 input loading.
- [x] 4.4 让 Tauri generation command 使用共享 builder 和 input loading.
- [x] 4.5 移除 provider mismatch 等 dispatch 后低收益重复校验, 保留 provider-owned parameter validation.

## 5. Core 模块拆分

- [x] 5.1 将 schema version, migration 和 schema helper 移入 `crates/imglab-core/src/library/schema.rs`.
- [x] 5.2 将 manifest IO, managed paths, checksum, image dimensions 和 storage size helper 移入 `storage.rs`.
- [x] 5.3 将 import, child version, generation event rows, lineage 和 version row mapping 移入 `assets.rs`.
- [x] 5.4 将 Gallery query, asset detail, file context 和 read-model helper 移入 `gallery.rs`.
- [x] 5.5 将 metadata suggestions, tag attach 和 metadata review actions 移入 `metadata.rs`.
- [x] 5.6 将 album create/add 和 smart album validation 移入 `albums.rs`.
- [x] 5.7 将 repair dry-run/apply row model 和 repair helper 移入 `repair.rs`.
- [x] 5.8 将 export version selection 和 sidecar writing 移入 `export.rs`.
- [x] 5.9 保持 `LocalLibraryService` public entry point 和 trait implementations 对外稳定.

## 6. 重复逻辑清理

- [x] 6.1 提取共享 tag upsert/attach helper, 并让 manual add tag 和 metadata review accept 共用.
- [x] 6.2 提取共享 `VersionSummary` row mapper, 替换重复 SQL mapping.
- [x] 6.3 提取 latest generation event id lookup, 替换 summary/detail 两套重复查询.
- [x] 6.4 提取 rating/min rating validation helper, 替换分散校验.
- [x] 6.5 清理已不再使用的 imports, DTO, helper 和 tests fixture.

## 7. 文档和规格同步

- [x] 7.1 更新 `docs/development.md` 中 checksum, database filename 和验证命令描述.
- [x] 7.2 更新 `docs/providers.md` 中 generation metadata 和 provider boundary 描述.
- [x] 7.3 更新 current OpenSpec specs, 将 SHA-256 标准, canonical checksum fields 和 desktop core read model 边界同步到 `openspec/specs/`.
- [x] 7.4 保持 archived changes 不改写历史.

## 8. 验证

- [x] 8.1 运行 `cargo fmt --all --check`.
- [x] 8.2 运行 `cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli`.
- [x] 8.3 运行 `cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok`.
- [x] 8.4 运行 `cd apps/desktop && npm run test && npm run build`.
- [x] 8.5 如果本地 Tauri 依赖可离线使用, 运行 `cargo check --offline -p imglab-desktop`.
- [x] 8.6 运行 `openspec validate refactor-code-health-architecture`.
