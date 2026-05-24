# AGENTS.md

## 0. 协作对象与默认工作方式

- 当前协作对象是 Honghao.
- 默认 Honghao 是资深后端 / 数据库工程师, 熟悉 Java, Rust, Go, Python 等主流语言及其生态.
- Honghao 重视 "Slow is Fast": 推理质量, 抽象边界, 架构演进和长期可维护性优先于短期速度.
- 解释, 讨论, 分析和总结使用简体中文. 使用 English punctuation symbols, 中文与 English words 之间保留空格.
- 代码, 注释, 标识符, commit message, Markdown 代码块内内容全部使用 English.
- 不讲基础语法或入门概念, 除非用户明确要求.

## 1. 当前项目事实

本仓库是 local-first AI image prompt lab desktop app. 当前 MVP 的事实来源以当前 worktree 为准, 主要事实如下:

- Tauri 2 + React 19 + TypeScript desktop shell.
- Rust workspace 作为共享业务层, `imglab-core` 是 CLI, daemon, Tauri backend 和 provider adapter 的业务事实来源.
- SQLite + local filesystem managed resource library.
- GUI-first, CLI 支持自动化和批处理.
- 单用户, local-first, 支持多个独立本地 resource library.
- 支持 text-to-image 和 image-to-image service boundary.
- Asset-level version lineage 是当前主版本模型.
- AI metadata suggestions 必须人工 review 后才写入 canonical asset metadata.
- 当前可用 provider: `fake`, `codex-cli` / `codex`.
- Grok provider crate 是边界占位, native implementation deferred.
- Desktop Generate workflow 通过 local daemon task 执行, 不在 Tauri process 内直接运行 Codex CLI.

主要入口:

```text
apps/desktop
apps/desktop/src-tauri
crates/imglab-core
crates/imglab-cli
crates/imglab-daemon
crates/imglab-provider-codex
crates/imglab-provider-grok
docs/development.md
docs/providers.md
openspec/specs
```

## 2. 架构边界

### 2.1 Rust workspace

`imglab-core` 当前采用 DDD boundary:

- `crates/imglab-core/src/domain`: business invariants, value objects, context-local policies. 不依赖 SQLite, filesystem IO, daemon, Tauri, CLI parser 或 desktop view types.
- `crates/imglab-core/src/application`: ports, use cases, read models, `ImgLabApplication` facade. 通过 ports 读取事实和提交 persistence command.
- `crates/imglab-core/src/infrastructure`: SQLite repositories, filesystem store, registry, provider composition adapters.
- `crates/imglab-core/src/interface_contracts`: runtime-facing DTO compatibility surface.
- `crates/imglab-core/src/library`, `dto.rs`, `provider.rs`, `compatibility.rs`: legacy compatibility surface. 新代码不要把这些当 primary boundary.

Runtime integration:

- `imglab-cli` 构造 application facade 后执行 library, asset, generation, album, metadata, task workflows.
- `imglab-daemon` 拥有 local HTTP runtime, scheduler, task executor, attempt logs 和 token-authenticated loopback API.
- `apps/desktop/src-tauri` 是 GUI adapter, 负责 Tauri commands, path handling, daemon sidecar discovery, error mapping 和 view mapping.
- Provider crates 只执行外部 provider, 返回 normalized provider output. 不分配 asset `version_number`, 不写 SQLite, 不解释 same-asset parent rule, 不返回 desktop 或 daemon view DTO.

业务规则 ownership:

- Asset version number, parent chain, reference source, generation operation inference 和 generation event persistence 属于 core application/domain.
- Task status transition, retry, output link 和 scheduler policy 属于 task bounded context 与 daemon scheduler boundary.
- Tauri, daemon, CLI 不应复制 core 已有的业务规则.

### 2.2 Desktop frontend

Desktop 前端位于 `apps/desktop/src`.

- `app/App.tsx` 主要负责 shell composition.
- `app/StudioAppController.tsx` 负责 top-level controller wiring.
- `app/workflows/*` 是 workflow state ownership 的主入口.
- `app/screens/*` 和 workflow-owned screen modules 负责 UI rendering.
- `workbench-state.ts` 是 compatibility barrel, 主要服务 existing Node tests. 新 workflow module 不应把它当 primary state owner.

当前一等 compact desktop 最小宽度目标是 `960px`. Gallery, Albums, Review, Queue, Settings 的高频路径必须在该宽度下可达.

## 3. Resource Library 与持久化契约

Resource library 是 managed local directory, 当前包含:

```text
manifest.json
library.sqlite
originals/imported
originals/generated
exports
```

SQLite 是权威索引. Sidecar/export files 只用于迁移, 调试和外部工具读取.

默认不改变以下契约, 除非先更新 OpenSpec artifacts 并写清 upgrade / rollback / compatibility verification:

- `manifest.json` identity 和 portable library metadata.
- `library.sqlite` schema version 和 migration semantics.
- Managed image file layout.
- Registry alias, unregister, backup zip import/export 和 clone-on-conflict 语义.
- Asset internal UUID + user-visible `version_number` / `version_name`.
- Uploaded reference image 作为独立 reference asset/version 管理, 不并入 output asset lineage.
- `SHA-256` checksum 为当前标准, historical metadata 需要保持可读.

## 4. 工作流纪律

### 4.1 默认协作流程

用户明确要求的强制流程是:

1. 使用 `superpowers:brainstorming` 做前期需求分析和需求细节描述.
2. 如果任务涉及页面设计, 交互布局, visual system, desktop workflow UX 或用户可见界面调整, 必须使用 `ui-ux-pro-max` skill 完成设计分析和交互布局方案.
3. 对产品, 架构, 行为变化, 持久化契约, workflow ownership 或用户可见 UX, 必须通过 OpenSpec 完成 proposal, design, tasks, specs.
4. 按 OpenSpec artifact 实施 change.
5. 验证.
6. 需要 closeout 时同步 specs, archive change, 再按用户要求 commit / push.
7. 任何 release 或 push 后, 必须检查 GitHub CI, 并确认对应 pushed head SHA 或 release-triggered workflow 成功后才可报告完成.

如果任务只是 trivial 修复, 查询或纯解释, 可以直接处理. 如果任务会改变产品行为, 架构边界, 持久化契约, workflow state ownership 或用户可见 UX, 默认先走 brainstorming + OpenSpec, 涉及界面时追加 `ui-ux-pro-max`, 除非用户明确要求跳过.

### 4.2 Superpowers 使用

- 在进入需求分析, 设计新功能, 修改行为或重构前, 使用 `superpowers:brainstorming`.
- Brainstorming 期间先探索当前项目文件, 再提出问题或方案.
- 不要在没有设计认可的情况下直接进入大规模实现.
- 如果只是继续已经认可的 OpenSpec change, 使用对应 OpenSpec skill 或当前 artifact 继续推进.

### 4.3 UI / UX 设计流程

- 涉及页面, 交互布局, navigation, dense desktop workflow, visual hierarchy, responsive behavior 或 component-level UX 时, 必须使用 `ui-ux-pro-max` skill.
- `ui-ux-pro-max` 产出的设计判断需要落到 OpenSpec design / tasks 或 implementation notes, 避免只停留在视觉建议.
- 如果用户要求完全重做界面, 默认先产出可检查的高保真 demo 或等价设计稿, 再改生产代码.

### 4.4 OpenSpec 使用

本仓库使用 OpenSpec workflow. 配置位于 `openspec/config.yaml`, 当前要求 OpenSpec 产出使用简体中文.

常用 change 流程:

```text
openspec new change "<change-name>"
openspec status --change "<change-name>" --json
openspec instructions <artifact-id> --change "<change-name>" --json
openspec validate <change-name> --strict
openspec validate --specs --strict
```

归档时必须:

- 检查 artifact status 和 `tasks.md`.
- 将 delta specs 同步到 `openspec/specs/*`.
- 运行 `openspec validate <change-name> --strict`.
- 运行 `openspec validate --specs --strict`.
- 将 change 移动到 `openspec/changes/archive/YYYY-MM-DD-<change-name>/`.

OpenSpec CLI 可能输出 `edge.openspec.dev` 或 PostHog telemetry 网络噪声. 以 exit status 和本地 artifact/spec 状态作为判断依据.

### 4.5 Release / Push 后 CI Gate

- 任何 `git push`, release tag push, GitHub Release publish 或 release workflow 触发后, 都必须检查 GitHub Actions.
- CI 检查必须尽量锚定刚推送的 head SHA, tag 或 release-triggered run, 不以旧 run 的绿色状态替代当前变更验证.
- 推荐使用 `gh run list`, `gh run watch --exit-status` 和 `gh run view` 确认相关 workflow / job 成功.
- 如果因网络, 权限或 GitHub 状态无法确认 CI, 最终状态必须明确标记为未确认, 不得报告为完成.

## 5. 代码修改原则

- 优先依据当前代码结构和 existing patterns, 不要发明孤立的新抽象.
- 新抽象必须消除真实复杂度, 降低有意义重复, 或匹配当前 DDD / workflow ownership 模型.
- 避免把旧大文件平移成 context-local 大文件. 拆分应基于 ownership 和变化原因, 不是形式.
- Runtime adapter 只做 input validation, error mapping, DTO mapping 和 integration glue.
- 对非平凡逻辑改动, 优先添加或更新测试.
- 涉及 library format, schema, task persistence, provider result 或 asset lineage 时, 必须主动考虑旧 library 兼容和升级语义.
- 不提交生成产物或依赖目录, 例如 `node_modules`, `dist`, `.test-dist`, `target`, Tauri generated schemas.

## 6. UI / UX 项目约定

- 当前 desktop 是 Studio Console, 不是 marketing page.
- UI 应紧凑, 可扫描, 面向重复操作. 避免大面积装饰性 hero, marketing-style cards 或无信息密度的视觉噪声.
- Dense table 默认优先 real SVG icon actions, 最小列数, 尽量无 button chrome.
- 窄屏或笔记本宽度下一行放不下视为回归. 优先删低价值列, 例如 status/schema 这类非关键列.
- 高频切换默认优先 row click, 不优先增加额外 switch button.
- Gallery 是 all-assets browser. Album selection 不应隐式改变 Gallery scope, 必须作为显式 Gallery filter.
- Albums 页面拥有 add-to-album 的 workflow ownership, 不依赖 Gallery 的隐式选择状态.
- Settings library management 默认拆成 Libraries / Providers / Updates / Logs.
- `Close` resource library 表示 unregister, 不是 hide, 也不是物理删除文件.
- Library rename 默认是本地 registry alias, 不是修改 manifest.
- Import/export zip 默认是完整 library backup/restore, id 冲突默认 clone, 不默认覆盖.
- `Open Existing Library` 和 `Reveal in Finder` 是不同动作.

## 7. 常用验证命令

按改动范围选择最小充分验证集.

Rust core / CLI / daemon:

```bash
cargo fmt --all --check
cargo test -p imglab-core
cargo test -p imglab-cli
cargo test -p imglab-daemon
cargo test -p imglab-desktop
cargo test -p imglab-provider-codex -p imglab-provider-grok
scripts/check-architecture.sh
```

Desktop frontend:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
npm run dev --prefix apps/desktop -- --host 127.0.0.1
```

OpenSpec:

```bash
openspec validate <change-name> --strict
openspec validate --specs --strict
```

通用收尾:

```bash
git diff --check
git status --short
```

## 8. 开发和运行入口

CLI smoke flow 建议使用独立 registry:

```bash
export IMGLAB_REGISTRY=/tmp/imglab-dev-registry.sqlite
cargo run --offline -p imglab-cli -- init /tmp/imglab-library --name Dev
cargo run --offline -p imglab-cli -- import --library /tmp/imglab-library /tmp/source.png --json
cargo run --offline -p imglab-cli -- search --library /tmp/imglab-library --json
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --json
```

Desktop frontend:

```bash
cd apps/desktop
npm install
npm run build
npm run dev -- --host 127.0.0.1
```

Full Tauri desktop:

```bash
cd apps/desktop
npm run tauri dev
```

Daemon sidecar 本地调试:

```bash
cargo build -p imglab-daemon
export IMGLAB_DAEMON_RUNTIME_DIR=/tmp/imglab-desktop-daemon
cargo run -p imglab-daemon
```

## 9. 文档与外部资料

- 当前项目事实优先级: current worktree > OpenSpec specs > docs > memory / previous run summary.
- 询问 library, framework, SDK, API, CLI tool 或 cloud service 的当前用法时, 使用 Context7 获取当前文档. 不要凭旧记忆回答版本敏感问题.
- Rust dependency implementation 优先查本地 `~/.cargo/registry`, 再考虑远程文档.
- GitHub 示例优先使用 `gh` CLI.

## 10. Git 与安全

- 搜索优先使用 `rg` 或 `rg --files`.
- 避免破坏性操作. 删除文件, 重建数据库, `git reset --hard`, force push 等必须先说明风险并确认.
- 不主动建议重写历史命令.
- 偏好非交互式 Git 命令.
- 工作树可能已有用户改动. 不要 revert 未经确认的用户改动.

## 11. 当前阶段默认不做

除非用户明确变更范围, 当前阶段默认 defer:

- 多用户协作.
- Cloud sync.
- Resource library encryption.
- Advanced backup and migration.
- Photoshop-style image editing.
- Graph-style lineage visualization.
- Stable native OpenAI / Grok image clients.
- 将 daemon, IPC 或 local HTTP API 扩展成 remote service.
