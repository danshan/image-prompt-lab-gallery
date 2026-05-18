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
cargo test -p imglab-core
cargo test -p imglab-daemon
cargo test -p imglab-desktop
openspec validate "add-generate-task-manager-daemon"
```

桌面前端:

```bash
cd apps/desktop
npm test
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

Generate 工作流由 desktop 调度本地 daemon task 完成. UI 的主要入口是 Tasks Queue, 其 task detail 会展示 input snapshot, attempts, timeline, output links 和 attempt log tail. Review Inbox 的 field regeneration 和 full suggestion regeneration 也会创建 daemon metadata task, 完成后通过 task output handoff 更新本地 review draft 或 pending suggestion.

桌面端完整运行:

```bash
cd apps/desktop
npm run tauri dev
```

开发环境下 desktop 会优先读取 `IMGLAB_DAEMON_RUNTIME_DIR/runtime.json` 中已运行 daemon 的连接信息. 如果不存在可用 daemon, desktop 会尝试启动 sidecar:

- `IMGLAB_DAEMON_BIN` 指向 daemon binary 时优先使用该路径.
- 未设置时, desktop 会在当前 executable 同目录查找 `imglab-daemon`.
- runtime directory 默认是系统 temp 下的 `imglab-desktop-daemon`.

本地验证 sidecar 前可以先构建 daemon:

```bash
cargo build -p imglab-daemon
```

也可以手动启动 daemon, 再让 desktop 发现它:

```bash
export IMGLAB_DAEMON_RUNTIME_DIR=/tmp/imglab-desktop-daemon
cargo run -p imglab-daemon
```

daemon runtime file 包含 API version, pid, loopback port 和 token file path. Desktop daemon client 会读取 token 并通过 local HTTP API 调用:

- `GET /v1/health`
- `GET /v1/capabilities`
- `POST /v1/libraries/open`
- `POST /v1/tasks` 和 `POST /v1/tasks/batch`
- `GET /v1/tasks?library_id=<library-id>` 和 `GET /v1/tasks/<task-id>`
- `POST /v1/tasks/reorder`
- `POST /v1/tasks/<task-id>/cancel`
- `POST /v1/tasks/<task-id>/retry`
- `POST /v1/tasks/<task-id>/duplicate`
- `GET /v1/tasks/<task-id>/events`
- `GET /v1/tasks/<task-id>/logs/tail`

daemon 只绑定 loopback address, 所有 task 和 log API 都要求本地 session token. Attempt log 会写入 daemon-owned log root, desktop Settings Logs 会把 task attempt logs 和 Codex adapter logs 一起列出.

## 数据位置

资源库是 managed directory, 包含:

- `manifest.json`.
- `library.sqlite`.
- `originals/imported`.
- `originals/generated`.
- `exports`.

SQLite 是权威索引. 导出 sidecar 只用于迁移, 调试和外部工具读取.
