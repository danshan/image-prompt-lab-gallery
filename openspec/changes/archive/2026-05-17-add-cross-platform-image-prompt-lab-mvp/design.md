## Context

当前仓库只有 OpenSpec 配置和已确认的产品设计文档, 尚未存在应用代码或既有 capability. 本变更需要从零建立跨平台桌面应用, CLI 和本地资源库核心模型.

关键约束:

- 单用户, local-first.
- 桌面端采用 Tauri + React + TypeScript.
- Core 采用 Rust + SQLite.
- GUI 和 CLI 必须共享同一个 Rust core.
- 图片文件采用 managed library, 导入和生成后复制进资源库.
- 一期支持 Codex CLI experimental adapter, OpenAI API stable provider 和 Grok native provider client.
- AI metadata 必须 review-first.
- 不做资源库加密, 多用户协作, 云同步, 外部引用文件或图谱式 lineage UI.

## Goals / Non-Goals

**Goals:**

- 建立 monorepo 骨架, 支撑 Rust core, CLI, provider crates 和 Tauri desktop.
- 定义本地资源库布局和 SQLite schema.
- 支持资源库生命周期, asset/version lineage, generation event, metadata suggestion, albums 和 search.
- 支持 CLI 自动化主路径, 包含 JSON output, dry-run 和稳定错误输出.
- 支持桌面端 Gallery, Albums, Review Inbox, Generation Queue, Settings 和 Inspector.
- 通过测试覆盖 core, CLI 和前端关键状态流.

**Non-Goals:**

- 不实现加密或访问控制.
- 不实现云同步或远程服务.
- 不实现多用户并发协作.
- 不支持外部文件引用作为权威来源.
- 不实现完整图谱式 lineage 可视化.
- 不实现 daemon, IPC API 或本地 HTTP API.

## Decisions

### Decision: 使用 Tauri + React + TypeScript 作为桌面端

桌面端使用 Tauri shell 承载 React + TypeScript UI. Tauri command 调用 Rust core, React 负责三栏 workbench, gallery, review inbox, generation composer 和 inspector.

选择理由:

- 跨平台打包能力符合需求.
- Rust core 可以被 Tauri 和 CLI 复用.
- React 生态更适合复杂状态界面和长期组件演进.

替代方案:

- Electron + React: UI 生态成熟, 但体积和资源占用更重, 与 Rust core 的边界收益较弱.
- Flutter + Rust core: UI 一致性强, 但 Rust bridge 和桌面生态复杂度更高.

### Decision: Rust core 是唯一业务源

所有资源库写操作必须通过 Rust core service. GUI 和 CLI 不直接修改 SQLite, 不直接写 managed layout 的业务状态.

选择理由:

- 避免 GUI 和 CLI 行为分叉.
- 资源库一致性, migration, integrity check 和 error model 可以集中维护.
- 后续如需 daemon 或 IPC, 可以复用 service boundary.

替代方案:

- GUI 和 CLI 分别实现持久化: 初期快, 但长期一致性风险高.
- 先做 GUI-only core: 产品验证快, 但 CLI 自动化会变成补丁式接入.

### Decision: 每个资源库是 managed directory

导入和生成图片都复制到资源库目录. SQLite 是权威索引, sidecar 只是导出和调试快照.

选择理由:

- 文件生命周期可控.
- hash 校验和导入导出更简单.
- 多资源库移动和备份更清晰.

替代方案:

- 引用外部文件: 节省空间, 但路径漂移, 权限和 hash drift 会让一致性复杂.
- 默认 managed, 可选引用: 灵活, 但一期需要处理两套生命周期.

### Decision: 使用 Asset + Version 模型

`assets` 表示逻辑作品, `asset_versions` 表示具体图片文件. 文生图, 图生图和 variation 会创建 version, generation event 解释 version 的来源.

选择理由:

- 符合用户对同一作品多次迭代的管理心智.
- title, rating, tags, albums 等 canonical metadata 可以稳定挂在 asset 上.
- 图生图和 prompt 修改后重生成可以保留 parent lineage.

替代方案:

- 每个输出都是独立 asset: 模型通用, 但用户层聚合更复杂.
- hybrid grouping: 灵活, 但一期交互和 schema 更重.

### Decision: Provider adapters 分为 stable API 和 experimental CLI

一期将图片 provider 分为两类: stable API provider 和 experimental command provider. OpenAI API provider 和 Grok provider 走 native API client. Codex 走本地 `codex exec` external command adapter, 复用用户本机 Codex 登录态和 imagegen skill, 但标记为 experimental.

选择理由:

- 当前 Codex CLI 没有公开图片生成 API 或可导出 API token contract, 直接读取 Codex 内部 auth 文件风险过高.
- `codex exec` 可以作为本地授权路径, 但不能指定图片模型或输出路径, adapter 必须从 Codex 文本输出中解析最终复制出的图片路径.
- stable API provider 与 experimental CLI provider 分离后, 错误模型和产物追踪仍可统一.

替代方案:

- 将 Codex auth 当作 native API credential: 不采用, 因为这是内部实现细节, 不是稳定 contract.
- 只支持 OpenAI API: 稳定但不满足通过 Codex 授权复用 `gpt-image-2` 的目标.
- 只支持 Codex CLI: 能复用本地登录态, 但自动化可靠性不足, 因此不作为唯一 provider.

Codex CLI adapter protocol:

1. Core 创建独立 job directory.
2. Adapter 调用 `codex exec --cd <job_dir> --sandbox workspace-write --json <prompt>`.
3. Prompt 使用共享 schema 描述 use case, asset type, primary request, input images, scene/backdrop 和 subject, 明确要求使用 imagegen skill.
4. Codex 生成后通常会复制图片到 `/tmp/...png` 并在最终文本中输出该路径, 原始图片保留在 `$HOME/.codex/generated_images/...`.
5. Adapter 从 Codex stdout/final text 中解析图片路径, 优先选择明确出现在 “已生成并复制到” 后的绝对路径, 其次选择 `$HOME/.codex/generated_images` 下的最近图片路径.
6. Adapter 校验文件存在, 将 command, stdout/stderr 和解析出的路径作为 raw response 保存.
7. 如果无法解析路径, 文件不存在或命令失败, adapter 返回 normalized domain error.

### Decision: AI metadata review-first

AI 自动标题, 分类, 打标和描述只写入 `metadata_suggestions`. 用户接受或编辑后才写入 canonical metadata.

选择理由:

- 避免 AI 批量污染资源库.
- 保留 review history.
- Review Inbox 可以成为稳定的人工整理入口.

替代方案:

- 自动应用并支持 undo: 效率高, 但数据可信度低.
- policy-based 自动应用: 灵活, 但一期交互和测试复杂.

### Decision: Manual albums + smart albums

Manual albums 使用显式 membership 和 sort order. Smart albums 使用受限 query 表达式, 一期只允许 tags, rating, provider, date, status, category 等稳定字段.

选择理由:

- 同时支持人工精选和自动归档.
- SQLite 查询足以支撑单用户本地场景.
- 受限 query 能降低测试和迁移风险.

替代方案:

- 只做 manual albums: 简单, 但管理效率不足.
- 只做 saved searches: 概念轻, 但产品心智不如相册直接.

## Risks / Trade-offs

- Rust/Tauri 边界复杂 → 使用 coarse-grained command DTO, 不把 SQLite row 暴露给 UI.
- Provider API 变化 → provider adapter 隔离 raw payload, core 只依赖 normalized result 和 normalized error.
- SQLite 与文件系统不一致 → 文件写入采用临时文件加 atomic rename, 再通过事务记录最终状态, 并提供 integrity check.
- Scope 过大 → 加密, 云同步, daemon, 外部文件引用和图谱 lineage 明确不进入一期.
- Smart album query 演进风险 → 一期使用受限结构化 query, 禁止自由 SQL.
- GUI/CLI 并发写入风险 → 一期依赖 SQLite locking 和短事务, 不承诺实时多进程同步体验.

## Migration Plan

这是从零开始的新能力, 没有既有用户数据迁移.

实施顺序:

1. 建立 monorepo, Rust workspace 和基础 CI/test 命令.
2. 实现资源库 layout, manifest, SQLite migration 和 registry.
3. 实现 asset/version/import/integrity core.
4. 实现 provider abstraction, fake provider tests, Codex CLI adapter, OpenAI API provider 和 Grok client.
5. 实现 metadata suggestion, albums 和 search.
6. 实现 CLI 主路径.
7. 实现 Tauri desktop 主路径.
8. 补齐 integration tests 和人工验收路径.

回滚策略:

- 因为没有生产数据迁移, 回滚以 Git revert 为主.
- 对开发期测试资源库, 使用 schema version 阻止不兼容版本打开.

## Open Questions

- OpenAI API 和 Grok 的最终 endpoint 和参数集合需要在实现 provider client 前用当前官方文档确认.
- Codex CLI adapter 依赖本机 `codex exec` 行为和登录态, 需要在验收时用真实 Codex 环境做人工验证.
- metadata suggestion 使用哪个模型或 provider 需要在实现时明确, 但不改变 review-first 数据模型.
- 缩略图和预览图的具体尺寸可以在实现阶段按 UI 性能测试确定.
