# 跨平台 AI 图片 Prompt Lab MVP 设计

## 摘要

构建一个跨平台桌面应用和 CLI, 用于管理 AI Agent 图片生成的 prompt, 生成图片, 元数据, 相册和版本 lineage. 一期采用 `Tauri + React + TypeScript` 作为桌面端, `Rust core + SQLite` 作为本地业务和持久化核心, CLI 与桌面端共享同一个 Rust core.

系统面向单一用户, local-first, 支持多个独立 managed resource library. 一期支持 Codex CLI imagegen experimental adapter, OpenAI API stable provider 和 Grok provider, 支持文生图和基于图片生成, 支持自动标题, 自动分类, 自动打标等 AI metadata suggestion, 但所有 AI 建议必须经人工 review 后才写入正式元数据.

## 目标

- 提供跨平台桌面应用, 支持生成, 浏览, 审核, 评分, 打标, 相册和版本追溯.
- 提供 CLI, 支持文生图, 图生图, 导入, 导出, 搜索, 标签, 评分, 相册和资源库管理.
- 使用 Rust core 作为唯一业务源, 避免 GUI 和 CLI 产生两套写路径.
- 使用本地文件系统和 SQLite 管理资源库.
- 支持多个资源库的创建, 打开, 切换, 隐藏, 导入和导出.
- 支持 Codex CLI imagegen adapter, OpenAI API 和 Grok 图片生成 provider.
- 支持 managed library: 导入和生成图片都复制进资源库目录.
- 支持 `Asset + Version` 模型, 保留 prompt, 参数, source version, provider request/response 和版本关系.
- 支持 manual albums 和 smart albums.
- 支持 AI metadata suggestion 的 review-first 工作流.

## 非目标

- 不支持多用户协作.
- 不支持云同步.
- 不支持资源库加密或敏感字段加密.
- 不支持引用外部图片作为权威来源.
- 不实现 Photoshop 式图片编辑器.
- 不实现完整图谱式 lineage 可视化.
- 不实现后台 daemon, IPC API 或本地 HTTP API, 但 Rust core 的 service boundary 应保留未来演进空间.
- 不支持 Codex 和 Grok 以外的 provider, 但 provider interface 需要可扩展.

## 整体架构

一期采用 monorepo 结构:

```text
image-prompt-lab-gallery/
  crates/
    imglab-core/
    imglab-cli/
    imglab-provider-codex/
    imglab-provider-grok/
  apps/
    desktop/
  docs/
  openspec/
```

主要边界:

```text
Tauri Desktop App
  - React + TypeScript UI
  - Gallery, Albums, Review Inbox, Generation Queue, Settings
  - 通过 Tauri command 调用 Rust core

CLI
  - 自动化入口
  - init, library, generate, import, export, tag, rate, album, search
  - JSON output, dry-run, stable exit codes

Rust Core
  - business service boundary
  - SQLite schema and migrations
  - managed file layout
  - asset/version lineage
  - metadata suggestion review
  - album and search
  - provider abstraction

Local Resource Library
  - manifest.json
  - library.sqlite
  - originals, derivatives, sidecars, exports, trash
```

Rust core 是唯一业务源. GUI 和 CLI 不直接实现资源库写入, 不直接绕过 core 修改 SQLite 或 managed file layout. 所有生成, 导入, metadata review, tag, rating, album 和 search 都通过 core service 完成.

## Core 模块

Rust core 按业务能力拆分:

- `LibraryService`: 创建, 打开, 校验, 隐藏, 切换, 导入导出资源库.
- `AssetService`: 导入图片, 计算 hash, 管理 asset 和 version, 生成 derivative.
- `GenerationService`: 文生图, 图生图, generation event, provider 调用, job 状态.
- `MetadataReviewService`: 创建 AI suggestion, 接受, 编辑, 拒绝, 写入 canonical metadata.
- `AlbumService`: manual album, smart album, album item 排序.
- `SearchService`: 基于 text, tag, rating, provider, date, status, category 的查询.
- `IntegrityService`: 文件存在性, hash 校验, SQLite 与文件布局一致性检查.

Core service 接口应以 domain command 和 DTO 为边界, 不泄漏 UI state, SQLite row 或 provider-specific payload.

## 资源库布局

每个资源库是独立目录:

```text
MyImageLab.library/
  manifest.json
  library.sqlite
  originals/
    2026/05/<version_id>.<ext>
  derivatives/
    thumbnails/<version_id>.jpg
    previews/<version_id>.jpg
  sidecars/
    <asset_id>.json
  exports/
  trash/
```

`manifest.json` 保存资源库级元数据: library id, display name, schema version, created timestamp, app compatibility 和 flags. `library.sqlite` 是权威索引和事务边界. `originals` 保存导入或生成的原始图片. `derivatives` 保存缩略图和预览图. `sidecars` 是可读快照, 用于导出, 调试和未来互操作, 但不是一期写入权威.

导入图片时复制文件到 `originals`. 外部源文件不参与后续一致性判断. 删除操作一期可以先移动到 `trash`, 后续再设计清理策略.

## 数据模型

核心表:

```text
assets
  id, library_id, media_type, title, description, category,
  rating, status, created_at, updated_at, captured_at

asset_versions
  id, asset_id, parent_version_id, generation_event_id,
  file_path, sha256, width, height, mime_type, version_label, created_at

generation_events
  id, asset_id, output_version_id, provider, provider_model, operation_type,
  prompt, negative_prompt, input_asset_version_id,
  parameters_json, raw_request_json, raw_response_json,
  status, started_at, completed_at, error_code, error_message

metadata_suggestions
  id, asset_id, source, suggested_title, suggested_description,
  suggested_tags_json, suggested_category, confidence_json,
  status, created_at, reviewed_at

tags
  id, name, color, created_at

asset_tags
  asset_id, tag_id, source, confirmed_at

albums
  id, name, description, kind, smart_query_json, created_at, updated_at

album_items
  album_id, asset_id, sort_order, added_at

library_registry
  id, name, root_path, hidden, created_at, last_opened_at
```

`assets` 表示逻辑作品. `asset_versions` 表示具体图片文件. 图生图, variation 和 prompt 修改后重新生成都创建新的 version, 并通过 `parent_version_id` 和 `generation_event_id` 保留 lineage.

Canonical title, description, category, rating, tags 和 album membership 默认挂在 asset 上. 如未来需要 version-specific metadata, 应新增明确字段或关联表, 不复用 asset 级字段表达两种语义.

## Provider 抽象

一期实现 provider adapters:

- `CodexCliImageProvider`: experimental, 通过本地 `codex exec` 复用 Codex 登录态和 imagegen skill, 从输出文本解析最终图片路径.
- `OpenAiApiImageProvider`: stable, 使用 OpenAI API key 和官方图片 API.
- `GrokImageProvider`: stable, 支持 Grok 图片生成.

Provider interface:

```text
ImageProvider
  - validate_parameters
  - generate_from_text
  - generate_from_image
  - normalize_response

ProviderCredentialStore
  - resolve_credentials
  - validate_credentials
```

Core 负责参数校验, credential resolution, command/API request 构造, response normalization, raw request/response persistence 和错误归一化. Provider-specific payload 只能存在于 provider adapter 和 persisted raw payload 中, 不应扩散到 GUI 或 CLI. Codex CLI adapter 必须从 Codex 输出文本中解析最终图片路径并校验文件存在.

## AI Metadata Suggestions

AI metadata flow 采用 review-first:

1. 导入或生成 asset 后创建 metadata suggestion job.
2. AI 输出 title, tags, category, description 和 confidence.
3. Core 写入 `metadata_suggestions`, 状态为 `pending_review`.
4. 用户在 Review Inbox 中接受, 编辑或拒绝.
5. 接受或编辑后才写入 `assets` 和 `asset_tags`.
6. 重新生成建议会创建新记录, 不覆盖 review history.

这个设计牺牲一部分自动化效率, 换取更高的数据可信度, 避免 AI 批量污染资源库.

## 桌面 UI

桌面端采用三栏 workbench:

```text
Library Sidebar | Workspace | Inspector
```

左侧 sidebar:

- Gallery
- Albums
- Review Inbox
- Generation Queue
- Settings
- Library switcher

中间 workspace:

- Gallery grid
- Album view
- Review inbox
- Generation composer
- Generation queue
- Search results

右侧 inspector:

- Title, rating, status, dates
- Prompt, negative prompt, provider, model, parameters
- Tags and album membership
- AI suggestions and review actions
- Versions and parent-child lineage
- File information and integrity status

核心 GUI 工作流:

- `Generate`: 文生图或图生图, 输出变成 asset version.
- `Review`: 接受, 编辑或拒绝 AI metadata suggestions.
- `Organize`: 评分, 打标, 分类, 加入 manual/smart albums.
- `Trace`: 查看源图, prompt, 参数, provider raw response 和 lineage.

Inspector 在窄窗口下需要可折叠. 一期 lineage 可用列表表达, 不需要图谱视图.

## CLI

一期 CLI 命令面:

```text
imglab init <path> --name <name>
imglab library list --json
imglab library open <path>
imglab library hide <library-id>
imglab generate --library <path> --provider codex-cli --prompt <text> --json
imglab generate --library <path> --input <asset-version-id> --prompt <text> --json
imglab import --library <path> <files...> --json
imglab export --library <path> --album <id> --out <path>
imglab tag add --library <path> <asset-id> <tag>
imglab rate --library <path> <asset-id> <1-5>
imglab album create --library <path> --name <name>
imglab album add --library <path> <album-id> <asset-id>
imglab search --library <path> --query <query> --json
imglab suggestion list --library <path> --status pending --json
imglab suggestion accept --library <path> <suggestion-id>
imglab suggestion reject --library <path> <suggestion-id>
```

CLI 规则:

- 查询和写操作支持 `--json`.
- 写操作支持 `--dry-run`.
- 错误输出使用稳定字段: `code`, `message`, `details`, `recoverable`.
- CLI 不维护独立业务状态机.
- GUI 和 CLI 并发打开同一资源库时, 一期依赖 SQLite locking 和短事务, 不做实时协作同步.

## 错误模型

Core domain errors:

```text
LibraryNotFound
SchemaMismatch
ProviderUnavailable
CredentialMissing
GenerationFailed
InvalidAssetReference
FileIntegrityMismatch
ConcurrentWriteConflict
InvalidSmartAlbumQuery
UnsupportedProvider
InvalidGenerationParameters
```

CLI 将 domain errors 映射成稳定 exit code 和 JSON error. GUI 将 domain errors 映射成可恢复动作, 例如打开设置, 选择资源库, 重试生成, 修复索引或查看详细错误.

Provider raw errors 应持久化在 `generation_events.error_message` 或 `raw_response_json`, UI 默认展示 normalized message.

## 测试策略

Rust core unit tests:

- Library create, open, schema migration.
- Asset import, hash, managed file copy.
- Asset/version lineage for text-to-image and image-to-image.
- Generation event persistence.
- Metadata suggestion pending, accepted, edited, rejected.
- Manual album add, remove, ordering.
- Smart album query behavior.
- File integrity check.

Rust integration tests:

- 在临时目录创建完整资源库.
- 使用 fake provider 模拟 success, failure, timeout 和 invalid parameters.
- 验证 SQLite 与文件系统一致性.
- 验证 provider raw request/response persistence.

CLI tests:

- JSON output shape.
- Dry-run 不落盘.
- Stable exit codes.
- Import, generate, tag, rate, album, search 主路径.

Frontend tests:

- ViewModel state transitions.
- Review Inbox accept, edit, reject flows.
- Library switcher open, hide, recent libraries.
- Inspector 渲染 prompt, parameters, tags, rating, lineage.

人工验收:

- 创建并打开多个资源库.
- 通过 GUI 文生图.
- 通过 GUI 基于图片生成.
- 通过 CLI 生成图片.
- 导入本地图片.
- 审核 AI title, tag, category, description suggestions.
- 评分, 打标, 分类.
- 创建 manual album 并添加 asset.
- 创建 smart album 并验证自动更新.
- 查看 asset version lineage 和 generation event.

## 一期验收标准

- 用户可以创建, 打开, 切换和隐藏多个本地资源库.
- 用户可以通过 GUI 和 CLI 使用 Codex CLI imagegen adapter 生成图片.
- 用户可以通过 GUI 和 CLI 使用 Grok 生成图片.
- 用户可以基于已有 asset version 生成新 version.
- 用户可以导入本地图片到 managed library.
- Gallery 能展示生成和导入的图片.
- Review Inbox 能展示 AI metadata suggestions.
- 用户可以接受, 编辑或拒绝 suggestions.
- 用户可以评分, 打标, 分类和加入相册.
- Manual albums 和 smart albums 均可工作.
- Inspector 能展示 prompt, provider, 参数, raw event summary, source image 和 version lineage.
- CLI 和 GUI 的写操作都通过同一个 Rust core.
- Core 测试覆盖资源库, 导入, generation event, suggestion review, albums 和 lineage.

## 风险与缓解

- Tauri 和 Rust core 边界复杂: 使用 coarse-grained service command 和 DTO, 避免 UI 依赖 SQLite row.
- Provider API 变化: 隔离 provider adapter, 持久化 raw request/response, 使用 normalized domain error.
- 文件与 SQLite 不一致: 写文件使用临时文件加 atomic rename, SQLite 事务记录最终状态, 提供 integrity check.
- AI metadata 污染: suggestion review-first, canonical metadata 必须由用户动作写入.
- 一期 scope 过大: 加密, daemon, cloud sync, graph lineage, 外部引用文件和高级迁移均不进入一期.

## OpenSpec 变更建议

后续 OpenSpec change 建议命名为 `add-cross-platform-image-prompt-lab-mvp`.

该 change 应覆盖:

- Monorepo 工程骨架.
- Rust core service boundary.
- SQLite schema and migrations.
- Managed resource library layout.
- Codex CLI adapter, OpenAI API provider 和 Grok provider.
- CLI 主路径.
- Tauri + React desktop 主路径.
- Review-first metadata suggestions.
- Asset + Version lineage.
- Manual and smart albums.
- Core, CLI 和 frontend 测试策略.
