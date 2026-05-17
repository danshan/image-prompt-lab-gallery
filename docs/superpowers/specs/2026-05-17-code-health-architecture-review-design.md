# 代码健康与架构 Review 设计

## 背景

当前项目是一个 Rust workspace 加 Tauri React 桌面应用. MVP 已经包含 local-first resource library, SQLite 存储, managed file layout, gallery/detail read model, metadata review, albums, CLI automation, 以及通过 provider adapter 进行图片生成.

这次 review 发现了几类架构债:

- `crates/imglab-core/src/library.rs` 超过 4000 行, 混合了 service orchestration, schema migration, storage helper, SQL row mapping, read model, repair, export 和 tests.
- `apps/desktop/src-tauri/src/lib.rs` 仍然暴露 `gallery_items`, 这是绕过 core gallery read model 的 direct SQL command.
- CLI 和 Tauri generation 路径重复实现了 provider dispatch, operation inference, input loading, request construction 和 default model label.
- 当前 checksum 实现把 MD5 digest 写入历史 `sha256` 列, 这会让持久化 metadata 语义不可信.
- Tag attach, version row mapping, latest generation event lookup, rating query validation 等逻辑在多个地方重复.
- 部分旧文档还在描述历史 MVP 细节, 与当前 storage 和 read-model 行为不一致.

## 目标

- 让 `imglab-core` 成为 SQLite 查询语义, managed file layout, repair, generation event, gallery/detail read model 的唯一业务事实来源.
- Tauri command 层只负责 transport mapping: input DTO 到 core request, core response 到 serializable view.
- 通过共享 generation request building 和 provider dispatch 边界, 保持 CLI 和 desktop generation 行为一致.
- 将当前 checksum 标准切换为 SHA-256, 移除 MD5 写入 `sha256` 的误导行为.
- 保留对现有 library 的兼容读取, 同时允许 repair 将 metadata 标准化到当前标准.
- 按职责拆分 `library.rs`, 但尽量保持 public service entry point 稳定.
- 删除过期代码路径和不再增加正确性的冗余参数检查.

## 非目标

- 不引入 daemon, IPC 或 local HTTP API.
- 不实现 native OpenAI 或 Grok image provider.
- 不重做桌面端视觉设计.
- 不替换 SQLite migration framework.
- 不改变 managed resource library directory layout, 除非通过现有 repair 语义修复历史数据.
- 不引入 repository trait, 除非出现真实的测试边界或 runtime boundary 需求.

## 推荐方案

采用分阶段架构清理, 而不是一次性大重写.

1. 先修正 checksum 语义和测试.
2. 再围绕 `checksum_algorithm` 和 `checksum` 收敛 public DTO/API.
3. 再移除 `gallery_items` 等过期 transport path.
4. 然后按职责拆分 `library.rs`.
5. 最后在模块边界清楚后抽取重复 helper.

这样可以让每个行为变更都可审阅, 避免机械拆文件掩盖语义变化.

## Core 架构

`LocalLibraryService` 保持 public service entry point. 内部代码移动到 `crates/imglab-core/src/library/` 下的私有模块.

建议模块布局:

```text
crates/imglab-core/src/
  library/
    mod.rs
    service.rs
    schema.rs
    storage.rs
    assets.rs
    gallery.rs
    metadata.rs
    albums.rs
    repair.rs
    export.rs
    generation.rs
```

职责划分:

- `service.rs`: 承载 `LocalLibraryService` 构造和 trait implementations.
- `schema.rs`: 承载 schema version, migrations 和 schema helper functions.
- `storage.rs`: 承载 manifest IO, managed paths, checksum calculation, image dimensions 和 storage size.
- `assets.rs`: 承载 import, child version creation, generation event rows, lineage 和 version row mapping.
- `gallery.rs`: 承载 gallery query, asset detail, file context 和 gallery read-model helpers.
- `metadata.rs`: 承载 metadata suggestions, tag upsert/attach 和 metadata review actions.
- `albums.rs`: 承载 manual/smart album creation 和 membership updates.
- `repair.rs`: 承载 repair dry-run/apply logic 和 repair row models.
- `export.rs`: 承载 export version selection 和 sidecar writing.
- `generation.rs`: 承载 CLI 和 Tauri 可共享的 generation request building 与 orchestration helper.

模块内部优先使用接收 `&Connection` 的私有函数. 在没有真实 multi-storage 需求前, 不引入宽泛 trait 或 dependency inversion.

## Checksum 语义

当前标准 checksum algorithm SHALL 是 SHA-256.

新行为:

- 新导入计算 SHA-256.
- 新生成 version 计算 SHA-256.
- 新 child version 计算 SHA-256.
- `asset_versions.sha256` 存储真实 SHA-256 digest.
- `asset_versions.checksum_algorithm` 存储 `SHA-256`.
- `asset_versions.checksum` 存储同一个 SHA-256 digest.
- Public DTO 和 transport view 以 `checksum_algorithm` 和 `checksum` 作为 canonical fields.
- CLI 和 Tauri JSON 不应把 `sha256` 作为业务字段输出. 如果临时保留兼容字段, 其值也必须是真实 SHA-256 digest.

兼容行为:

- 读取现有 rows 时继续使用 `COALESCE(checksum, sha256)`.
- Integrity check 使用 row 自己的 `checksum_algorithm`.
- Repair 将旧 MD5 或 legacy SHA-256 rows 标准化到当前标准: 重新计算 SHA-256, 并更新 `sha256`, `checksum_algorithm`, `checksum`.

这意味着此前已经 repair 成 MD5 的 library, 再次 repair 时会更新为 SHA-256. 文件 bytes 不变, 只变更 metadata digest fields.

## Desktop 边界

Tauri command handler 不应为了业务 read 直接打开 SQLite.

必须清理:

- 移除 `gallery_items` 和 `GalleryItemView`.
- 移除不再使用的 `GalleryItemsInput`.
- 从 Tauri invoke handler 中取消注册 `gallery_items`.
- 确保真实 Tauri gallery rendering 只使用 `query_gallery`.
- 确保 detail rendering 只使用 `get_asset_detail`.

Browser preview/mock frontend 可以保留本地 mock data 和本地 filtering, 但必须与真实 Tauri mode 清晰隔离.

## Generation 边界

Generation orchestration 应在 CLI 和 desktop 间共享.

共享逻辑覆盖:

- `codex`, `codex-cli`, `fake` 的 provider name normalization 和 dispatch.
- 从 optional input file 或 input version 推断 operation.
- 校验 image-to-image 是否具备必要 input.
- 从 explicit input file 或 library version 加载 input bytes.
- 默认 model label 选择.
- 构造 `GenerationParameters` 和 `GenerateImageRequest`.

Provider crate 仍然负责 provider-specific execution. 例如 Codex CLI command construction 和 log capture 继续留在 `imglab-provider-codex`.

Provider-specific validation 只校验该 provider 真正拥有的参数. 当 dispatch 已经选择 provider 后, provider mismatch check 属于低收益冗余校验.

## 去重目标

在语义变更完成后抽取小 helper:

- `attach_tag` 或等价 helper, 统一 tag upsert 和 `asset_tags` insert.
- `VersionSummary` 的共享 version row mapper.
- latest generation event id 的共享 lookup.
- rating/min rating 的共享 validation.
- CLI 和 Tauri 的共享 generation request building.
- Tauri view mapping 中无 transport-specific logic 的重复 conversion helper.

避免过度抽象 SQL access. 优先使用 focused private functions, 而不是 generic repository traits.

## 文档更新

更新仍描述过期行为的 docs 和 current specs:

- 将 "MD5 current standard" 改为 "SHA-256 current standard".
- 移除或澄清旧的 `sha256` business-field 描述.
- 确保 resource library database 命名与当前实现一致: `library.sqlite`.
- 标记 `gallery_items` 风格的 direct desktop SQL read 为 obsolete.
- Archived OpenSpec changes 保留历史记录, 但 current specs 和 active docs 应反映新标准.

## 实施顺序

1. 将 checksum 写入和 repair normalization 切换到 SHA-256.
2. 更新 import, generation, child version, repair 和 integrity tests.
3. 从 public DTO 和 transport JSON 中移除或弱化 `sha256`.
4. 移除 Tauri `gallery_items` direct SQL command.
5. 引入 `library/` module structure, 按职责移动代码.
6. 抽取 tag, version mapping, latest-event, rating validation 和 generation request helper.
7. 更新 docs 和 current OpenSpec specs.

## 验证

运行:

```bash
cargo fmt --all --check
cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli
cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok
cd apps/desktop && npm run test && npm run build
```

如果 Tauri dependencies 已经可离线使用, 额外运行:

```bash
cargo check --offline -p imglab-desktop
```

关键覆盖点:

- 新 import 写入 SHA-256 metadata.
- 新 generation 写入 SHA-256 metadata.
- 新 child version 写入 SHA-256 metadata.
- Legacy MD5 rows 在 repair 前仍可读取.
- Repair dry-run 报告需要 checksum normalization 的 rows.
- Repair apply 将 checksum metadata 改写为 SHA-256.
- Integrity check 尊重每行自己的 `checksum_algorithm`.
- CLI JSON 不再暴露误导性 `sha256`.
- Tauri 不再注册或使用 `gallery_items`.
- CLI 和 desktop image-to-image validation 行为一致.

## 风险

- 从 public DTO 移除 `sha256` 可能影响仍引用它的 tests 或 frontend types. 需要配套明确的 API 和测试更新.
- 拆分 `library.rs` 会产生较大的机械 diff. 应在 checksum 语义修正后, 尽量保持拆分行为不变.
- 将 MD5 metadata repair 为 SHA-256 会改变现有 library 的 stored digest fields. 这是预期行为, 但测试必须证明 file paths 和 asset identities 不变.
- 共享 generation helper 不能把 provider implementation details 拉进 core.

## 已确认决策

- 选择 deep cleanup 范围.
- Checksum 标准是 SHA-256, 不是 MD5.
- SQLite 可以保留历史 `sha256` 列用于兼容, 但 public business API 使用 `checksum_algorithm` 和 `checksum`.
- 实施范围包含架构重构建议, 不只是删除旧代码.
