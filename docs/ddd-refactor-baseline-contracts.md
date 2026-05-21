# DDD 重构前行为契约基线

本文档记录 `refactor-core-ddd-boundaries` 实施前需要保持兼容的 public contracts。它不是新产品设计, 只作为 DDD 重构期间的回归参照。

## Core Boundary

当前 `imglab-core` 是 CLI, daemon, Tauri backend 和 provider crates 的共享业务入口。重构期间必须保持以下可见行为:

- Resource library 创建, 打开, registry 列表, alias rename, unregister, backup zip import/export, repair 和 integrity check 行为保持稳定.
- Asset import 创建新 asset 和首个 version, version 使用 UUID 作为内部标识, 并返回 asset 内递增的 `version_number` 与 `version_name`.
- Existing-version image-to-image 在同一 asset 下创建 child version.
- Uploaded-reference image-to-image 将 input file 导入为独立 reference asset/version, output asset 不并入 reference asset lineage.
- Generation event 继续记录 provider, model, operation, prompt, input version, output version, raw request, raw response, status 和错误信息.
- Metadata suggestion 仍需人工 review 后才写入 canonical asset metadata.
- Album, gallery, smart query, review 和 task read models 的可见字段保持兼容.
- SQLite 仍是 resource library 主事实存储, 本次重构默认不提升 schema version.

## CLI Contract

`imglab-cli` 当前提供以下主入口, 重构后 command name, option 语义, exit code 分类和 JSON shape 不应发生非预期变化:

```text
imglab help
imglab init <library-path> --name <name> [--dry-run] [--json]
imglab library list [--include-hidden] [--json]
imglab library open <library-path> [--json]
imglab library hide <library-id> [--dry-run] [--json]
imglab library repair --library <library-path> [--apply] [--json]
imglab import --library <library-path> <file-path> [--dry-run] [--json]
imglab export --library <library-path> --out <output-path> [--album <album-id>] [--dry-run] [--json]
imglab search --library <library-path> [--query <query>] [--json]
imglab generate --library <library-path> --prompt <prompt> [--provider <provider>] [--input-file <path>] [--input-version <version-id>] [--parameters <json>] [--dry-run] [--json]
imglab tag ...
imglab rate ...
imglab album ...
imglab suggestion ...
```

Canonical version JSON fields remain:

```text
asset_id
version_id
version_number
version_name
file_path
checksum_algorithm
checksum
```

Errors continue to map from core domain errors to stable exit behavior and JSON error payloads where supported.

## Daemon API Contract

Daemon remains local-only and token-authenticated. API version remains `v1`. Existing loopback API shape remains compatible:

```text
GET /v1/health
GET /v1/capabilities
POST /v1/libraries/open
POST /v1/tasks
POST /v1/tasks/batch
GET /v1/tasks?library_id=<library-id>
GET /v1/tasks/<task-id>
POST /v1/tasks/reorder
POST /v1/tasks/<task-id>/cancel
POST /v1/tasks/<task-id>/retry
POST /v1/tasks/<task-id>/duplicate
GET /v1/tasks/<task-id>/events
GET /v1/tasks/<task-id>/logs/tail
```

Task model compatibility requirements:

- Task, attempt, event 和 output link 仍持久化在 resource library 中.
- Scheduler 继续尊重 global concurrency, provider concurrency, priority 和 queue position.
- Retry policy 继续区分 transient 和 non-transient errors.
- Attempt logs 仍写入 app-owned log root, 且 log tail 读取必须拒绝非 task-owned arbitrary paths.

## Tauri Backend Contract

Tauri command layer remains a GUI adapter. 重构后:

- Command input/output view shape 保持兼容.
- Error mapping 保持面向 UI 的 recoverable message.
- Path normalization, reveal helpers, dialog integration, daemon sidecar discovery 和 runtime file/token handling 行为保持稳定.
- Write operations 继续通过 core/application 层作为业务事实来源, 不在 Tauri command 中复制 domain rule.

## Desktop Frontend Contract

Desktop UX 不在本次 change 中重设计。以下 workflow 的可见行为保持稳定:

- Studio shell 和 compact desktop 操作可达性保持稳定.
- Gallery query, selection, detail, inspector, lightbox 和 reference source display 保持兼容.
- Generation composer 继续通过 daemon task 进入 queue.
- Albums workspace 支持 manual/smart album 主路径.
- Review workspace 支持 suggestion accept/reject, field regeneration 和 task handoff.
- Queue workspace 支持 task detail, attempt timeline, logs, reorder, retry 和 duplicate.
- Settings 保持 Libraries / Logs / diagnostics 语义.

## Provider Contract

当前 provider behavior:

- `fake` provider 用于 deterministic smoke tests.
- `codex-cli` / `codex` provider 通过本机 `codex exec` 和 imagegen skill 生成图片, 不指定模型或输出路径, 从 stdout/stderr 解析最终图片路径后导入 library.
- Grok provider crate 保留 boundary placeholder, native implementation deferred.
- Native provider credential resolution 应继续通过 `ProviderCredentialStore` 或后续等价 port 注入, 不读取 Codex 内部授权文件.

## Resource Library Compatibility

本次 DDD 重构默认不改变:

- `manifest.json` identity 和 portable library metadata.
- `library.sqlite` schema version.
- Managed image file layout: `originals/$year/$month/$uuid.$extension`.
- Registry alias, unregister, backup zip clone-on-conflict 和 import/export 语义.
- Current checksum standard: `SHA-256`, while historical `MD5` metadata remains readable.

如果实现中必须改变上述任一持久化契约, 必须先更新 OpenSpec artifacts, 明确 upgrade, rollback 和兼容验证。

## Baseline Verification

2026-05-21 baseline results before DDD implementation:

```text
cargo fmt --all --check
passed

cargo test -p imglab-core
passed: 59 passed, 0 failed, 1 ignored

cargo test -p imglab-daemon
passed: 16 passed, 0 failed

cargo test -p imglab-cli
passed: 6 passed, 0 failed

npm test --prefix apps/desktop
passed: 29 passed, 0 failed

npm run build --prefix apps/desktop
passed
```
