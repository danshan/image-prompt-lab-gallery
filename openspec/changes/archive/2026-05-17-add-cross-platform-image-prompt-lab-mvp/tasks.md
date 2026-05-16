## 1. 工程骨架

- [x] 1.1 创建 Rust workspace, 包含 `imglab-core`, `imglab-cli`, `imglab-provider-codex`, `imglab-provider-grok` crates.
- [x] 1.2 创建 Tauri + React + TypeScript desktop app 骨架.
- [x] 1.3 配置共享 lint, format, test 命令和基础 CI 入口.
- [x] 1.4 建立 core DTO, domain error 和 service trait 的基础模块.

## 2. Resource Library

- [x] 2.1 实现 managed library 目录创建, `manifest.json` 写入和必要目录初始化.
- [x] 2.2 实现 SQLite schema migration, schema version 校验和 `SchemaMismatch` 错误.
- [x] 2.3 实现 app-level library registry, 支持 list, open, hide 和 last opened time.
- [x] 2.4 实现 import 图片到 `originals`, hash 计算, SQLite 记录和 derivative 任务接口.
- [x] 2.5 实现 export 资源库或 album 的图片和 sidecar metadata.
- [x] 2.6 实现 integrity check, 覆盖文件存在性和 hash mismatch.
- [x] 2.7 添加 resource library unit 和 integration tests.

## 3. Asset Versioning

- [x] 3.1 实现 `assets`, `asset_versions`, `generation_events` repository 和 service.
- [x] 3.2 实现导入图片创建新 asset 和首个 version.
- [x] 3.3 实现同一 asset 下创建 child version 和 parent version 关系.
- [x] 3.4 实现 generation event 持久化, 包含 prompt, parameters, raw payload 和错误字段.
- [x] 3.5 实现 version lineage 查询 API.
- [x] 3.6 添加 asset/version/generation event 测试.

## 4. Image Generation

- [x] 4.1 定义 `ImageProvider` 和 `ProviderCredentialStore` traits.
- [x] 4.2 实现 fake provider, 用于 success, failure, timeout 和 invalid parameters 测试.
- [x] 4.3 实现 Codex CLI imagegen experimental adapter, 包含命令构造, 输出路径解析和错误归一化.
- [ ] 4.4 按当前官方文档确认 OpenAI API 与 Grok image generation API, 并实现 stable native clients.
- [x] 4.5 实现文生图 service flow, 保存输出文件, version 和 generation event.
- [x] 4.6 实现图生图 service flow, 记录 input asset version id 和 parent lineage.
- [x] 4.7 实现 provider 参数校验和 normalized error mapping.
- [x] 4.8 添加 generation integration tests.

## 5. Metadata Review

- [x] 5.1 实现 `metadata_suggestions` schema, repository 和状态流.
- [x] 5.2 实现导入或生成完成后创建 pending suggestion 的 job/service.
- [x] 5.3 实现 suggestion accept, edit 和 reject.
- [x] 5.4 确保 pending suggestion 不写入 canonical asset metadata.
- [x] 5.5 实现重新生成 suggestions 时保留 review history.
- [x] 5.6 添加 metadata review tests.

## 6. Albums 和 Search

- [x] 6.1 实现 tags 和 asset_tags schema, repository 和 service.
- [x] 6.2 实现 rating, category, status 更新 service.
- [x] 6.3 实现 manual album 创建, 添加, 移除和排序.
- [x] 6.4 实现 smart album 受限 query schema 和校验.
- [x] 6.5 实现 text, tag, rating, provider, date, status, category search.
- [x] 6.6 添加 manual album, smart album 和 search tests.

## 7. CLI

- [x] 7.1 实现 `imglab init`, `library list`, `library open`, `library hide`.
- [x] 7.2 实现 `imglab import`, `export`, `search`.
- [x] 7.3 实现 `imglab generate` 文生图和图生图命令.
- [x] 7.4 实现 `tag add`, `rate`, `album create`, `album add`.
- [x] 7.5 实现 `suggestion list`, `suggestion accept`, `suggestion reject`.
- [x] 7.6 为查询和写操作实现 `--json`.
- [x] 7.7 为写操作实现 `--dry-run`.
- [x] 7.8 实现 stable exit codes 和 JSON error payload.
- [x] 7.9 添加 CLI command tests.

## 8. Desktop Workbench

- [x] 8.1 实现 Tauri command 层, 通过 Rust core 暴露资源库, asset, generation, suggestion, album 和 search 操作.
- [x] 8.2 实现三栏 layout: Library Sidebar, Workspace, Inspector.
- [x] 8.3 实现 library switcher, Gallery 和 asset selection.
- [x] 8.4 实现 Inspector metadata, prompt, parameters, tags, albums, versions 和 integrity 展示.
- [x] 8.5 实现 Generation Composer 和 Generation Queue.
- [x] 8.6 实现 Review Inbox 的 accept, edit, reject flow.
- [x] 8.7 实现 Albums view, 包含 manual albums 和 smart albums.
- [x] 8.8 添加前端 state transition 和核心视图测试.

## 9. 验收和文档

- [x] 9.1 编写开发运行文档, 覆盖 CLI 和 desktop 启动方式.
- [x] 9.2 编写 provider credential 配置文档.
- [x] 9.3 执行 core unit tests 和 integration tests.
- [x] 9.4 执行 CLI tests.
- [x] 9.5 执行 desktop frontend tests.
- [ ] 9.6 完成人工验收: 创建库, GUI/CLI 生成, 导入, review, 评分打标, manual/smart album, lineage 查看.
