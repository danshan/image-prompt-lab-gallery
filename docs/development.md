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
cargo test -p imglab-cli
cargo test -p imglab-daemon
cargo test -p imglab-desktop
cargo test -p imglab-provider-codex -p imglab-provider-grok
scripts/check-architecture.sh
openspec validate refactor-core-ddd-boundaries --strict
```

桌面前端:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
npm run dev --prefix apps/desktop -- --host 127.0.0.1
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

## 模块地图

当前 core 采用 DDD boundary. 新代码优先按 `domain -> application -> infrastructure -> runtime interface` 的依赖方向组织, runtime 不应直接复刻已经迁移到 domain 或 application 的业务规则.

```text
crates/imglab-core/src/
  domain/
    shared/             # identities and shared value objects
    library/            # resource library identity, manifest, schema compatibility
    asset/              # asset aggregate, versions, lineage, version number policies
    generation/         # generation operation, provider capability and request policy
    metadata_review/    # suggestion review status and confidence policies
    album/              # manual and smart album rules
    task/               # task status machine, attempts, retry and scheduler policies
  application/
    facade.rs           # stable ImgLabApplication entrypoint
    ports/              # repositories, storage, provider, clock, id and transaction ports
    use_cases/          # write orchestration for library, assets, generation, review, albums, tasks
    read_models/        # query/read shapes shared by application callers
    testing.rs          # fake/in-memory ports for application tests
  infrastructure/
    composition.rs      # SQLite-backed composition root
    sqlite/             # context-owned repository adapters and schema entrypoint
    filesystem/         # managed file store adapter
    registry/           # local library registry adapter
  interface_contracts/
    dto.rs              # runtime-facing DTO compatibility surface
  compatibility.rs      # doc-hidden legacy root exports for downstream migration
  library/              # compatibility local-library facade and legacy service adapters
  dto.rs                # legacy DTO definitions still exposed through interface contracts
  provider.rs           # provider compatibility traits and fake provider surface

crates/imglab-daemon/src/
  lib.rs                # module hub and public daemon entrypoint
  runtime.rs            # daemon runtime state and application facade ownership
  runtime_io.rs         # runtime and token file persistence
  transport.rs          # local HTTP parsing, auth, response serialization
  routes.rs             # API route dispatch and request parsing
  scheduler.rs          # daemon-owned recovery, runnable checks, scheduler ticks
  executors.rs          # task execution through application use cases and provider ports
  logs.rs               # task attempt logs and log tail reads
  task_dto.rs           # daemon task request DTOs
  views.rs              # daemon response DTOs and conversions

apps/desktop/src-tauri/src/
  lib.rs                # Tauri plugin setup, managed state, command registration, startup
  errors.rs             # command error mapping
  paths.rs              # registry paths, daemon paths, library path normalization, reveal helpers
  services.rs           # desktop application facade construction
  views.rs              # command input and output DTOs
  view_mappers.rs       # domain-to-command-view conversion
  commands/             # Tauri commands split by workflow

apps/desktop/src/
  main.tsx              # React bootstrap only
  app/App.tsx           # shell composition wrapper
  app/StudioAppController.tsx
                        # top-level desktop controller wiring
  app/types.ts          # frontend DTOs and app-level types
  app/mock-data.ts      # browser preview fixtures
  app/tauri-adapter.ts  # invoke, dialog, runtime and image path adapter
  app/utils.ts          # focused formatting, prompt, path and thumbnail helpers
  app/workflows/
    gallery/            # gallery controller, query, derived state and pure state
    albums/             # album controller, smart/manual album query and state
    review/             # review controller, draft state and screen props
    tasks/              # queue/task controller, detail, derived state and transitions
    settings/           # library/log settings controller and state
    library/            # shared library selection state
    shared/             # generic pure state helpers
  app/screens/          # screen/workflow components
  app/components/       # small shared UI components
  app/hooks/            # workflow hook entrypoints
  workbench-state.ts    # compatibility barrel for existing Node tests
```

`imglab-core` remains the business source of truth. CLI, daemon, and Tauri command layers should call the application facade, use cases, or explicit interface contracts instead of re-implementing provider normalization, model defaults, operation inference, version-number allocation, retry rules, task transitions, or library mutation semantics.

## DDD 边界与 Composition Root

`domain` 只表达业务事实和 invariant. 它可以依赖 shared value objects 和 domain-local policies, 但不能依赖 SQLite, filesystem, daemon, Tauri, CLI parser, or desktop view types.

`application` 负责 orchestration. Use cases 通过 ports 读取事实, 组合 domain policy, 决定 transaction boundary, 再把明确的 persistence command 交给 repository adapters. 例如 asset child version 的 `version_number` 由 application/domain policy 决定, SQLite repository 只执行 insert/update.

`infrastructure` 实现 ports. 当前 SQLite-backed composition root 位于 `crates/imglab-core/src/infrastructure/composition.rs`, 通过 `sqlite_application(registry_path, provider)` 装配 repositories, managed file store, registry, clock, id generator 和 transaction manager.

Runtime integration:

- `imglab-cli` 在 command handler 中构造 `sqlite_application(...)`, 并通过 application facade 执行 library, asset, search, generation, album, metadata 和 task workflows.
- `imglab-daemon` 的 `DaemonState` 持有 `SqliteImgLabApplication<P>` 和 daemon-owned runtime state. Routes, scheduler 和 executors 通过 application facade/use cases 修改业务状态.
- `apps/desktop/src-tauri` 通过 `services::desktop_app()` 构造 desktop application facade. Tauri commands 只做 input validation, error mapping 和 command view mapping.
- `apps/desktop/src` 的 workflow state 放在 `app/workflows/*`. Screen 和 hook 通过 workflow-owned controller/state/derived modules 协作, 避免把 Gallery, Albums, Review, Queue 和 Settings 的状态机重新集中到单一大文件.

## Architecture Check

架构依赖检查:

```bash
scripts/check-architecture.sh
```

该脚本当前扫描 `crates/imglab-core/src/domain`, `crates/imglab-core/src/application`, runtime crates 和 desktop workflow modules, 并报告以下风险:

- Domain modules 直接导入 `rusqlite`.
- Domain modules 直接使用 `std::fs`.
- Domain modules 依赖 daemon, CLI, Tauri, infrastructure 或 legacy concrete local-library service.
- Application modules 直接依赖 SQLite, filesystem, runtime crates, infrastructure 或 legacy library modules.
- Runtime modules 重新引入 `LocalGenerationService` 或 asset `version_number` 分配规则.
- Desktop source modules 把 `workbench-state` 当作 primary state owner 导入.

这不是完整静态分析器. 它是一个低成本 guardrail, 用来阻止最危险的依赖方向回流. 如果新增 domain boundary, application port, runtime adapter 或 workflow state owner, 应优先扩展该脚本的搜索模式, 再把新规则写入本节.

## 桌面端

桌面端前端位于 `apps/desktop/src`. Tauri command 层位于 `apps/desktop/src-tauri/src`. `lib.rs` 只负责 Tauri plugin setup, managed state, command registration 和 app startup, 具体 command 按 workflow 放在 `commands/` 下.

前端通过 `@tauri-apps/api/core` 的 `invoke` 调用 Rust command. 浏览器预览环境没有 Tauri runtime, 当前 UI 保留 mock state 以便快速验证布局和状态切换.

Generate 工作流由 desktop 调度本地 daemon task 完成. UI 的主要入口是 Tasks Queue, 其 task detail 会展示 input snapshot, attempts, timeline, output links 和 attempt log tail. Review Inbox 的 field regeneration 和 full suggestion regeneration 也会创建 daemon metadata task, 完成后通过 task output handoff 更新本地 review draft 或 pending suggestion.

桌面端完整运行:

```bash
cd apps/desktop
npm run tauri dev
```

开发环境下 desktop 会先尝试复用健康的 background automation daemon, 再读取 `IMGLAB_DAEMON_RUNTIME_DIR/runtime.json` 中已运行 daemon 的连接信息. 如果不存在可用 daemon, desktop 会尝试启动 sidecar:

- `IMGLAB_DAEMON_BIN` 指向 daemon binary 时优先使用该路径.
- 未设置时, desktop 会在当前 executable 同目录查找 `imglab-daemon`.
- runtime directory 默认是系统 temp 下的 `imglab-desktop-daemon`.
- background automation daemon runtime 默认位于 `~/Library/Application Support/Image Prompt Lab/daemon`.
- Settings Automation 会安装 macOS LaunchAgent `com.imagepromptlab.daemon`, 并通过 `IMGLAB_REGISTRY` 指向 app registry, 使 app 未启动时 daemon 仍能恢复 automation-enabled libraries.

本地验证 sidecar 前可以先构建 daemon:

```bash
cargo build -p imglab-daemon
```

也可以手动启动 daemon, 再让 desktop 发现它:

```bash
export IMGLAB_DAEMON_RUNTIME_DIR=/tmp/imglab-desktop-daemon
cargo run -p imglab-daemon
```

也可以模拟 background daemon runtime:

```bash
cargo build -p imglab-daemon
export IMGLAB_DAEMON_RUNTIME_DIR=/tmp/imglab-background-daemon
export IMGLAB_REGISTRY=/tmp/imglab-dev-registry.sqlite
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
- `GET /v1/schedules?library_id=<library-id>`
- `POST /v1/schedules`
- `GET`, `PUT`, `DELETE /v1/schedules/<job-id>`
- `POST /v1/schedules/<job-id>/enable`
- `POST /v1/schedules/<job-id>/disable`
- `POST /v1/schedules/<job-id>/run-now`
- `GET /v1/schedules/<job-id>/runs`

daemon 只绑定 loopback address, 所有 task, schedule 和 log API 都要求本地 session token. Attempt log 会写入 daemon-owned log root, desktop Settings Logs 会把 task attempt logs, prompt-expansion logs 和 background daemon logs 一起列出.

## 数据位置

资源库是 managed directory, 包含:

- `manifest.json`.
- `library.sqlite`.
- `originals/imported`.
- `originals/generated`.
- `exports`.

SQLite 是权威索引. 导出 sidecar 只用于迁移, 调试和外部工具读取.
