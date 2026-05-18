## Why

当前项目已经具备 resource library, gallery/search, desktop workbench 和 daemon task manager 的 MVP 能力, 但 `docs/PERFORMANCE_REVIEW.md` 指出了多处会直接影响大图库体验和长期维护的性能与代码健康风险. 这些风险集中在 N+1 SQL 查询, 缺失热路径索引, 整文件读取, 同步长任务 command, frontend 单体组件, daemon 粗粒度锁和日志扫描边界.

现在需要在继续扩展 provider, gallery, task 和 metadata workflow 之前, 用 OpenSpec 方式把这些 review 发现收敛为可实施的架构与行为要求, 并在一次完整 cleanup 中完成.

## What Changes

- 优化 Gallery 和 Search core read path, 消除 per-asset tags, latest event, review count, album membership 和 album order 的 N+1 查询.
- 为 SQLite 热路径增加必要索引, 并建立 SQLite sufficiency checkpoint, 不预设必须永久保留 SQLite.
- 将 checksum 和 image dimensions 读取改为 bounded memory 路径, 避免对大图片整文件读入内存.
- 将 legacy desktop `generate_image` command 从同步长任务执行改为不会阻塞 Tauri command thread 的执行边界.
- 拆分 desktop frontend 的单体 `main.tsx`, 将高频 workflow 组件, data hooks 和纯 helper 按职责分离.
- 为 Gallery 图片加载, query refresh, derived state, IPC refresh waterfall 和 polling cleanup 增加性能约束.
- 优化 daemon request locking, scheduler no-work tick, daemon client timeout/backoff 和 task/log operational path.
- 限制 Settings app logs 只扫描 app-owned roots, 不再扫描整个 system temp directory.
- 更新性能 review 文档和当前 OpenSpec specs, 记录 SQLite 是否仍然足够以及未来触发补充索引或替换存储的条件.

## Capabilities

### New Capabilities

- `performance-code-health`: 定义跨 core, desktop, daemon 和 storage checkpoint 的性能与代码健康约束.

### Modified Capabilities

- `albums-search`: Gallery/Search 查询语义保持不变, 但 read path 必须避免 N+1 和 comparator 内数据库查询.
- `resource-library`: checksum 和 image dimensions 读取必须使用 bounded memory 路径; SQLite schema 需要包含热路径索引.
- `desktop-workbench`: Gallery 图片加载, query refresh, derived state, IPC refresh 和 frontend component 边界需要满足性能约束.
- `task-manager-daemon`: daemon request handling, scheduler tick 和 client timeout/backoff 需要保持长任务期间的响应性.
- `app-logs`: app log listing 必须限制在 app-owned roots, 不得扫描整个 system temp directory.

## Impact

- Affected Rust code:
  - `crates/imglab-core/src/library/gallery.rs`
  - `crates/imglab-core/src/library/schema.rs`
  - `crates/imglab-core/src/hash.rs`
  - `crates/imglab-core/src/library/storage.rs`
  - `crates/imglab-daemon/src/lib.rs`
  - `crates/imglab-daemon/src/main.rs`
  - `apps/desktop/src-tauri/src/lib.rs`
  - `apps/desktop/src-tauri/src/daemon_client.rs`
  - `apps/desktop/src-tauri/src/app_logs.rs`
- Affected frontend code:
  - `apps/desktop/src/main.tsx`
  - `apps/desktop/src/workbench-state.ts`
  - potential new `components`, `hooks` and `lib` modules under `apps/desktop/src/`
- Affected docs and specs:
  - `docs/PERFORMANCE_REVIEW.md`
  - `openspec/specs/albums-search/spec.md`
  - `openspec/specs/resource-library/spec.md`
  - `openspec/specs/desktop-workbench/spec.md`
  - `openspec/specs/task-manager-daemon/spec.md`
  - `openspec/specs/app-logs/spec.md`
- API impact:
  - Public behavior should remain compatible. Gallery/search result semantics, task API endpoints and desktop workflows must not change except for improved responsiveness.
- Data impact:
  - SQLite migrations add indexes. No managed file layout change is expected.
- Dependency impact:
  - No default storage replacement dependency is introduced. Supplemental search/storage backends require a later dedicated design if the checkpoint shows SQLite is insufficient.
