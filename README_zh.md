# Image Prompt Lab Gallery

[English README](README.md)

Image Prompt Lab Gallery 是一个 local-first 桌面应用, 用于管理 AI 图片生成 prompt, 生成图片, 元数据, 相册和 asset version lineage.

当前 MVP 基于 Tauri + React 桌面壳, Rust 核心业务层, SQLite 管理的本地 resource library, 以及支持自动化和批处理的 CLI.

## 当前状态

项目处于活跃 MVP 开发阶段. 当前稳定基线包括:

- 基于 Tauri, React, TypeScript 的跨平台桌面壳.
- Rust workspace 作为 desktop 和 CLI 写操作共享的核心业务层.
- 基于 SQLite 与本地文件系统的 managed resource library.
- GUI-first workflow, 同时提供 CLI 自动化能力.
- text-to-image 和 image-to-image 的 service boundary.
- asset-level version lineage.
- AI metadata suggestions 必须经过人工 review 后才写入 canonical metadata.
- 当前可用图片 provider: `fake` 和 Codex CLI imagegen adapter.
- Grok provider crate 已保留边界, native implementation 暂缓.

## 仓库结构

```text
apps/desktop              Tauri and React desktop application
crates/imglab-core        Core domain model, services, storage, and provider traits
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

## 前置条件

- Rust stable toolchain.
- Node.js 和 npm.
- Tauri 2 所需的平台依赖.
- 如果需要使用 Codex CLI imagegen provider, 本机需要可用并已登录的 `codex` CLI.

SQLite 通过 Rust 依赖使用, 不需要单独运行 SQLite server.

## 快速开始

在仓库根目录运行核心检查:

```bash
cargo fmt --all --check
cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok
cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli
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
