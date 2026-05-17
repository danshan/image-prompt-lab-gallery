## Why

当前 MVP 的核心业务边界已经成形, 但代码仍保留了原型阶段的架构债: desktop command 中存在绕过 core 的 direct SQL read, core 单文件承担过多职责, CLI 与 desktop generation orchestration 重复, checksum metadata 还存在 MD5 写入 `sha256` 列的语义错误. 现在需要在继续扩展 provider, gallery 和版本能力前, 先把业务事实来源和持久化语义压实.

## What Changes

- 将当前 asset version checksum 标准从 MD5 调整为 SHA-256.
- 保留读取历史 checksum metadata 的兼容能力, 但 repair 将历史 MD5 或不符合当前标准的 row 标准化为 SHA-256.
- 从 public DTO, CLI JSON 和 Tauri transport view 中移除或弱化误导性的 `sha256` 业务字段, 以 `checksum_algorithm` 和 `checksum` 作为 canonical fields.
- 移除 Tauri `gallery_items` direct SQL command, 真实桌面 Gallery 和 Inspector read path 必须通过 core read model.
- 将 `imglab-core` 中过大的 `library.rs` 拆分为按职责划分的内部模块, 保持 `LocalLibraryService` public entry point 稳定.
- 收敛 CLI 和 desktop generation orchestration, 共享 provider dispatch, operation inference, input loading 和 request building.
- 提取 tag attach, version row mapping, latest generation event lookup, rating validation 等重复逻辑.
- 更新当前 docs 和 OpenSpec specs, 清理 MD5 当前标准, 旧 `sha256` 业务字段和 `gallery_items` direct SQL read 的过期描述.

## Capabilities

### New Capabilities

- `code-health-architecture`: 约束 core 模块边界, transport 边界, generation orchestration 复用和过期路径清理的架构健康要求.

### Modified Capabilities

- `resource-library`: checksum 当前标准从 MD5 改为 SHA-256, repair normalization 和 file context checksum 展示随之调整.
- `asset-versioning`: asset version file metadata 从通用 hash/历史 `sha256` 语义收敛到 canonical checksum algorithm 和 checksum.
- `desktop-workbench`: desktop read path 必须使用 core gallery/detail read model, 不得保留 `gallery_items` direct SQL command.
- `image-generation`: CLI 和 desktop generation request construction 与 provider dispatch 必须共享一致的 core orchestration 边界.
- `cli-automation`: CLI JSON output 不得暴露误导性 `sha256` 业务字段, 应输出 canonical checksum metadata.

## Impact

- Affected code:
  - `crates/imglab-core/src/library.rs` 及拆分后的 `crates/imglab-core/src/library/*`.
  - `crates/imglab-core/src/dto.rs` 和 service traits.
  - `crates/imglab-cli/src/main.rs` 及 CLI tests.
  - `apps/desktop/src-tauri/src/lib.rs`.
  - `apps/desktop/src/main.tsx` 和 frontend state/tests 中引用 version checksum 字段的位置.
  - `docs/development.md`, `docs/providers.md`, `openspec/specs/*`.
- API impact:
  - Public version DTO 和 JSON output 以 `checksum_algorithm` 和 `checksum` 为稳定字段.
  - `sha256` 不再作为业务字段暴露; 如短期保留兼容字段, 值必须是真实 SHA-256 digest.
- Data impact:
  - Existing libraries remain readable.
  - Running repair may update stored checksum metadata from MD5 to SHA-256 without changing managed file bytes, asset ids, version ids, or file paths.
- Dependency impact:
  - No new runtime storage dependency is required.
  - No daemon, IPC, HTTP API, native OpenAI provider, or native Grok provider is introduced.
