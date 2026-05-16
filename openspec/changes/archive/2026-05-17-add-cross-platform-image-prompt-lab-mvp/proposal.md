## Why

AI Agent 图片生成会产生大量 prompt, 参数, 输入图片, 输出图片和后续整理动作. 仅靠普通文件夹很难追溯生成来源, 比较版本, 审核 AI 自动打标结果, 或通过 CLI 批量管理产物.

本变更建立一个 local-first, 单用户, 跨平台的图片 Prompt Lab MVP, 让桌面端和 CLI 共享同一个 Rust core, 以可维护的方式支撑生成, 管理, 审核, 相册和版本追溯.

## What Changes

- 新增 monorepo 工程骨架, 包含 Rust core, CLI, provider crates 和 Tauri + React 桌面应用.
- 新增 managed resource library 格式, 每个资源库使用独立目录, `manifest.json`, SQLite 数据库和受管理图片文件.
- 新增资源库 registry, 支持创建, 打开, 切换, 隐藏, 导入和导出.
- 新增 `Asset + Version` 数据模型, 保存图片文件, prompt, provider 参数, generation event 和 parent-child lineage.
- 新增 provider adapters, 一期支持 Codex CLI experimental adapter, OpenAI API stable provider 和 Grok 图片生成.
- 新增文生图和基于图片生成能力.
- 新增 review-first AI metadata suggestion, 支持自动标题, 标签, 分类和描述建议.
- 新增 manual albums 和 smart albums.
- 新增 CLI 主路径, 包含 JSON output, dry-run 和稳定错误输出.
- 新增 Tauri 桌面工作台, 包含 Gallery, Albums, Review Inbox, Generation Queue, Settings 和 Inspector.
- 不引入加密, 多用户协作, 云同步, 外部引用文件或图谱式 lineage UI.
- 无 breaking changes, 因为当前仓库尚未存在产品能力或公开 API.

## Capabilities

### New Capabilities

- `resource-library`: 管理本地资源库生命周期, 文件布局, SQLite schema, registry, 导入导出, 隐藏和完整性校验.
- `asset-versioning`: 管理 asset, asset version, 文件 hash, derivative, generation event 和版本 lineage.
- `image-generation`: 通过 provider adapters 支持 Codex CLI `gpt-image-2`, OpenAI API 和 Grok 的文生图与图生图.
- `metadata-review`: 为导入和生成的图片创建 AI metadata suggestions, 并支持人工接受, 编辑或拒绝.
- `albums-search`: 支持 manual albums, smart albums, tag, rating, category, provider, date, status 和 text search.
- `cli-automation`: 提供 `imglab` CLI, 支持资源库, 生成, 导入, 导出, 标签, 评分, 相册, 搜索和 suggestion review.
- `desktop-workbench`: 提供 Tauri + React 桌面工作台, 支持 Gallery, Albums, Review Inbox, Generation Queue, Settings 和 Inspector.

### Modified Capabilities

无. 当前没有既有 OpenSpec capability 需要修改.

## Impact

- 新增 Rust workspace, 预计包含 `imglab-core`, `imglab-cli`, `imglab-provider-codex`, `imglab-provider-grok` 等 crates.
- 新增 Tauri + React + TypeScript 桌面应用.
- 新增 SQLite schema, migrations 和本地资源库文件布局.
- 新增 provider credential resolution, command/API request/response persistence 和 normalized error model.
- 新增 CLI 命令面, JSON output contract, dry-run behavior 和 stable exit codes.
- 新增 core unit tests, integration tests, CLI tests 和前端 state/view tests.
- 新增依赖方向约束: GUI 和 CLI 必须通过 Rust core 写入资源库, 不允许各自实现独立持久化路径.
