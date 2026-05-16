# Image Prompt Lab 开发运行

本文档覆盖当前 MVP 的本地开发入口. 项目采用 Rust workspace + Tauri React 桌面端, CLI 和桌面端共享 `imglab-core`.

## 前置条件

- Rust stable toolchain.
- Node.js 和 npm.
- SQLite 由 Rust 依赖内置使用.
- 如需真实 Codex 生图, 本机需要可用的 `codex` CLI 和已登录的 Codex 环境.

## 常用验证命令

```bash
cargo fmt --all --check
cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok
cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli
openspec validate "add-cross-platform-image-prompt-lab-mvp"
```

桌面前端:

```bash
cd apps/desktop
npm run build
npm run dev -- --host 127.0.0.1
```

Tauri Rust crate 需要完整 Cargo 依赖缓存. 如果离线检查提示缺失依赖, 先在允许联网的环境中准备依赖:

```bash
cargo check -p imglab-desktop
```

## CLI 示例

建议为开发测试指定独立 registry, 避免污染默认临时 registry.

```bash
export IMGLAB_REGISTRY=/tmp/imglab-dev-registry.sqlite
cargo run --offline -p imglab-cli -- init /tmp/imglab-library --name Dev
cargo run --offline -p imglab-cli -- import --library /tmp/imglab-library /tmp/source.png --json
cargo run --offline -p imglab-cli -- search --library /tmp/imglab-library --json
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --json
```

真实 Codex 生图:

```bash
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider codex-cli --prompt "a quiet product render" --json
```

`--dry-run` 可用于写操作预览:

```bash
cargo run --offline -p imglab-cli -- import --library /tmp/imglab-library /tmp/source.png --dry-run --json
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --dry-run --json
```

## 桌面端

桌面端前端位于 `apps/desktop/src`. Tauri command 层位于 `apps/desktop/src-tauri/src/lib.rs`.

前端通过 `@tauri-apps/api/core` 的 `invoke` 调用 Rust command. 浏览器预览环境没有 Tauri runtime, 当前 UI 保留 mock state 以便快速验证布局和状态切换.

桌面端完整运行:

```bash
cd apps/desktop
npm run tauri dev
```

## 数据位置

资源库是 managed directory, 包含:

- `manifest.json`.
- `imglab.sqlite`.
- `originals/imported`.
- `originals/generated`.
- `exports`.

SQLite 是权威索引. 导出 sidecar 只用于迁移, 调试和外部工具读取.
