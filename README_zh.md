# Image Prompt Lab Gallery

[English README](README.md)

Image Prompt Lab Gallery 是一个 local-first 桌面应用, 用于管理 AI 图片生成 prompt, 生成图片, 元数据, 相册和 asset version lineage.

当前 MVP 基于 Tauri + React 桌面壳, Rust 核心业务层, SQLite 管理的本地 resource library, 以及支持自动化和批处理的 CLI.

## 当前状态

项目处于活跃 MVP 开发阶段. 当前稳定基线包括:

- 最新 desktop release: `v0.1.7`. 见 [GitHub Releases](https://github.com/danshan/image-prompt-lab-gallery/releases/latest).
- 基于 Tauri, React, TypeScript 的跨平台桌面壳.
- Rust workspace 中的 `imglab-core` 是 desktop, CLI, daemon 写操作共享的 DDD 业务核心.
- 基于 SQLite 与本地文件系统的 managed resource library.
- GUI-first workflow, 同时提供 CLI 自动化能力.
- text-to-image 和 image-to-image 的 service boundary.
- asset-level version lineage, Gallery version tree inspection, 以及 version promotion.
- 一等 Prompt Workspace, 支持 prompt documents, version history, variables 和 generation lineage.
- 通过 local daemon task runtime 执行 scheduled image generation.
- Library content lifecycle workflows, 支持本地 library 对比, merge, deduplicate 和 cleanup.
- AI metadata suggestions 必须经过人工 review 后才写入 canonical metadata.
- 当前可用图片 provider: `fake` 和 Codex CLI imagegen adapter.
- Grok provider crate 已保留边界, native implementation 暂缓.

## 截图

![Studio Gallery workspace](docs/screenshots/studio-gallery.png)

![Settings library management](docs/screenshots/studio-settings.png)

## 功能概览

- **Gallery**: 浏览所有 managed assets, 按 provider, album, rating 和 review state 过滤, 检查当前 asset detail, 并查看生成图片的 version lineage.
- **Albums**: 管理 manual 和 smart collections, 但 album selection 不会隐式改变 Gallery scope.
- **Prompt Workspace**: 创建 prompt documents, 维护 prompt versions, 将 prompt version 直接用于 generation, 并保留 prompt-to-image lineage.
- **Generation and schedules**: 执行 text-to-image 和 image-to-image workflows, 通过 daemon queue 管理 generation tasks, 并配置 recurring scheduled image generation.
- **Review Inbox**: 在 AI metadata suggestions 写入 canonical asset metadata 前进行人工 review.
- **Settings**: 在紧凑 desktop console 中管理 local libraries, providers, app updates, automation diagnostics 和 logs.
- **CLI**: 通过脚本初始化 libraries, import assets, search, generate images, 并执行 batch operations.

## 仓库结构

```text
apps/desktop              Tauri and React desktop application
crates/imglab-core        DDD core: domain, application ports/use cases, infrastructure adapters
crates/imglab-cli         CLI for library, asset, search, generation, album, and metadata workflows
crates/imglab-provider-codex
                          Codex CLI imagegen provider adapter
crates/imglab-provider-grok
                          Grok provider boundary placeholder
docs/development.md       Local development and validation notes
docs/providers.md         Provider behavior and configuration notes
openspec/specs            Current product and architecture specifications
openspec/changes          Proposed and archived spec-driven changes
```

`imglab-core` 围绕明确边界组织:

- `domain`: business invariants and reusable policies.
- `application`: ports, use cases, read models, and the `ImgLabApplication` facade.
- `infrastructure`: SQLite, filesystem, registry, and provider composition adapters.
- `interface_contracts`: runtime-facing DTO compatibility surface.

Runtime layers 应调用 application facade 或明确的 interface contracts, 不应复制 generation planning, asset version allocation, task transitions 或 library mutation semantics.

## 前置条件

- Rust stable toolchain.
- Node.js 和 npm.
- Tauri 2 所需的平台依赖.
- 如果需要使用 Codex CLI imagegen provider, 本机需要可用并已登录的 `codex` CLI.

SQLite 通过 Rust 依赖使用, 不需要单独运行 SQLite server.

## Desktop Release

当前 desktop release 路径是通过 GitHub Releases 发布的 macOS Tauri build. Release artifacts 包含 updater artifacts 和供 Tauri updater endpoint 使用的 `latest.json`.

当前 release signing 使用 macOS ad-hoc signing 和 Tauri updater signing. 它不是 Apple Developer ID notarization, 因此首次打开下载应用时仍可能需要处理 macOS Gatekeeper 提示.

版本规则, signing boundary, GitHub secrets 和 release 验证步骤见 [docs/release.md](docs/release.md).

## 快速开始

在仓库根目录运行核心检查:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run build --prefix apps/desktop
scripts/check-architecture.sh
```

构建并运行 desktop frontend:

```bash
cd apps/desktop
npm install
npm run build
npm run dev -- --host 127.0.0.1
```

运行完整 Tauri 桌面应用:

```bash
cd apps/desktop
npm run tauri dev
```

更多开发命令见 [docs/development.md](docs/development.md).

## CLI 用法

开发测试时建议使用独立 registry, 避免污染默认本地状态:

```bash
export IMGLAB_REGISTRY=/tmp/imglab-dev-registry.sqlite
cargo run --offline -p imglab-cli -- init /tmp/imglab-library --name Dev
cargo run --offline -p imglab-cli -- import --library /tmp/imglab-library /tmp/source.png --json
cargo run --offline -p imglab-cli -- search --library /tmp/imglab-library --json
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --json
```

支持的写操作可以使用 `--dry-run` 预览影响:

```bash
cargo run --offline -p imglab-cli -- import --library /tmp/imglab-library /tmp/source.png --dry-run --json
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --dry-run --json
```

## Providers

当前 MVP 将 provider 分为两类:

- Experimental CLI provider: 复用本机命令及其已有授权状态.
- Stable native provider: 通过公开 API 和显式 credential 调用.

当前可用 provider:

- `fake`: 用于本地 smoke test 的 deterministic provider.
- `codex-cli` / `codex`: 调用本机 `codex exec`, 然后将生成图片导入 managed library.

Native OpenAI API 和 Grok provider 会在稳定实现边界明确后再落地.

Provider 细节见 [docs/providers.md](docs/providers.md).

## Resource Library 模型

Resource library 是一个 managed local directory, 包含:

- `manifest.json`
- `library.sqlite`
- `originals/imported`
- `originals/generated`
- `exports`

SQLite 是权威索引. Exported sidecar files 仅用于迁移, 调试和外部工具读取.

## Specification Workflow

本仓库使用 OpenSpec-style workflow 管理产品, 架构和行为变化. 当前 specs 位于 [openspec/specs](openspec/specs), active 和 archived changes 位于 [openspec/changes](openspec/changes).

涉及实质行为变化时, 应先更新或创建相关 OpenSpec artifacts, 再进入实现.

## License

本项目使用 [MIT License](LICENSE).
