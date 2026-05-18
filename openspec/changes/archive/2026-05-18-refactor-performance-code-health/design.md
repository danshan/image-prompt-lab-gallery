## Context

本变更基于 `docs/PERFORMANCE_REVIEW.md` 的完整 Tech & Code Review. 当前项目的主要性能风险不是单点慢函数, 而是多层边界叠加:

- Rust core 的 Gallery/Search read path 先加载资产, 再对每个 asset 查询 version, generation event, tags, pending review count 和 album membership.
- SQLite schema 已有 task 相关索引, 但 gallery/search 热路径表缺少关键索引.
- checksum 和图片尺寸读取仍可能把大文件完整读入内存.
- desktop legacy `generate_image` command 仍同步执行长任务, 与 daemon task manager 方向不一致.
- frontend `main.tsx` 过大, workflow state, IPC, view rendering 和 helper 混在一起, 使每次局部状态变化容易触发过多 re-render.
- daemon HTTP handling 和 scheduler tick 使用粗粒度 state 边界, 在长任务和无任务轮询时有不必要成本.
- Settings app logs 会扫描过宽的 temp 范围.

用户要求使用 OpenSpec 推动, 且希望一次性完成完整 cleanup. 因此本设计把完整范围纳入同一个 change, 但实施内部仍按 wave 切分, 以便 review, 测试和回滚.

## Goals / Non-Goals

**Goals:**

- 修复 `docs/PERFORMANCE_REVIEW.md` 中已确认且仍适用于当前代码的性能风险.
- 保持 Gallery/Search, desktop workflow 和 daemon API 的用户可见语义稳定.
- 将 core read path 从 N+1 查询改为 batch preload 和 in-memory assembly.
- 为 SQLite 热路径添加索引, 并记录 SQLite sufficiency checkpoint.
- 降低大文件 checksum 和图片尺寸读取的内存峰值.
- 降低 frontend 渲染, IPC waterfall 和 polling 的无效工作.
- 降低 daemon 无任务 tick 和普通 HTTP 查询的锁/clone 成本.
- 把 review 发现落实到 OpenSpec requirements 和 tasks, 再进入实现.

**Non-Goals:**

- 不引入与性能 cleanup 无关的新产品功能.
- 不重做桌面视觉设计.
- 不默认替换 SQLite.
- 不在本次直接引入 PostgreSQL, DuckDB, Tantivy 或新的搜索服务.
- 不改变 managed resource library layout.
- 不改变 loopback daemon HTTP contract.
- 不实现 native OpenAI 或 Grok provider.

## Decisions

### 先修查询形态和索引, 不先替换 SQLite

选择先保留 SQLite, 重写 read path 和索引, 再通过 sufficiency checkpoint 决定是否需要补充或替换存储.

原因:

- 当前瓶颈主要是 N+1 查询, 缺索引和不合适的调用位置. 替换数据库不能自动修复这些问题.
- 项目是 local-first desktop, SQLite 的部署成本和迁移模型仍然最符合当前 MVP.
- 若未来需要全文搜索或分析型查询, 可以先考虑 SQLite FTS5, projection table, Tantivy sidecar 或 DuckDB supplemental.

替代方案:

- 直接 PostgreSQL: 能力强, 但引入 server lifecycle, packaging 和 local setup 成本.
- DuckDB 主存储: 适合分析, 不适合作为当前 OLTP-style library fact store.
- Tantivy 主存储: 适合搜索索引, 不适合作为 canonical metadata store.

### Gallery/Search 使用 batch read model

Gallery read path 应加载 base assets 后, 批量加载 latest versions, latest generation events, version counts, tags, pending review counts, album membership 和 sort orders. 组装逻辑在内存中完成.

原因:

- 消除 `O(N * queries)` 和 sort comparator 内 SQL.
- 保持 `GalleryReadService` public API 稳定.
- 允许逐步引入 projection table 或 FTS, 不改变上层调用者.

替代方案:

- 单条巨型 SQL join: 查询数最低, 但 tags aggregation, latest version fallback 和 smart album 组合会让 SQL 复杂度快速上升.
- projection table: 适合后续大图库优化, 但第一步先修现有 read path 更可控.

### Desktop 先按 workflow 拆分, 不引入全局状态库

Frontend 应按 Gallery, Albums, Review, Task, Settings, Inspector 等 workflow 拆分组件和 hooks. 第一阶段不引入 Redux/Zustand 等新依赖.

原因:

- 当前问题首先是单文件职责过多和 derived state 不稳定, 不是全局状态能力不足.
- 增加状态库可能掩盖 invalidation 问题, 并扩大迁移范围.

替代方案:

- 直接引入全局 store: 对后续大型 UI 有价值, 但当前会增加迁移风险.
- 只加 `React.memo`: 能缓解部分 re-render, 但不能解决职责混杂和 refresh waterfall.

### Daemon 保持 transport, 优化锁边界和 no-work path

Daemon 第一轮不替换 HTTP server framework. 优先缩短 state lock scope, 让 health/capabilities 尽可能 lock-free, 并避免无任务时深 clone `DaemonState`.

原因:

- 当前 loopback HTTP contract 已进入 desktop sidecar 集成.
- 主要问题是锁粒度和 tick 成本, 不是 HTTP 协议本身.
- 保留 transport 可以降低 task recovery 和 client 兼容风险.

替代方案:

- 引入 async web framework: 可扩展性更强, 但迁移范围较大.
- 改 Unix socket/named pipe: 本地安全边界更强, 但跨平台成本高.

### App logs 只扫 app-owned roots

Settings Logs 应只从 app-owned directories 读取已知 pattern 的日志. 不再扫描整个 system temp directory.

原因:

- 扫描 temp root 性能不可控, 也扩大了路径安全审计面.
- 现有 spec 已经要求 app-owned logs, 实现应收敛到该边界.

## Risks / Trade-offs

- [Risk] Gallery/Search batch read path 可能改变 latest event fallback 或 sort tie-break 行为. → Mitigation: 先补回归测试, 明确 latest version/event tie-break, 保持现有可见排序.
- [Risk] 新索引增加 SQLite 文件体积和 migration 时间. → Mitigation: 只添加热路径索引, 用 `CREATE INDEX IF NOT EXISTS`, 并覆盖 existing-library migration.
- [Risk] Frontend 拆分过程中 stale closure 或 refresh ordering 出错. → Mitigation: 先抽 pure helper 和 workflow 子组件, 再调整 debounce/memo/parallel refresh.
- [Risk] batched `Promise.all` 暴露过去依赖串行刷新的隐含顺序. → Mitigation: 只并发语义独立的 refresh, detail refresh 保持条件化.
- [Risk] Daemon lock 优化影响 recovery 或 task status transition. → Mitigation: route 层先保持 API shape, 增加 scheduler 和 request tests.
- [Risk] SQLite sufficiency checkpoint 结果显示需要额外 backend. → Mitigation: 本 change 只记录 evidence 和触发条件, 真正替换或补充 backend 需要单独 OpenSpec change.

## Migration Plan

1. 创建或补充 behavior-preserving tests, 覆盖 Gallery/Search, album order, checksum, dimensions, app logs 和 daemon scheduler.
2. 添加 SQLite indexes 并实现 core batch read path.
3. 将 hash/dimensions 改为 bounded memory.
4. 修复 legacy `generate_image` blocking command.
5. 拆分 desktop workflow 组件与 hooks, 再添加 lazy image, debounce, memo 和 refresh batching.
6. 优化 daemon lock scope, scheduler no-work path, client timeout/backoff 和 app logs root.
7. 增加 SQLite sufficiency checkpoint 文档和可复用测试/benchmark evidence.
8. 运行完整验证命令, 修复回归.

Rollback:

- 每个 wave 保持可独立回滚.
- Core service public signatures 优先保持稳定.
- Desktop 先拆结构再改行为, 避免一次 diff 同时变更 UI 与数据流.
- Daemon endpoint 和 response shape 不改变.
- Storage backend 不在本 change 中替换.

## Open Questions

- SQLite sufficiency checkpoint 的目标图库规模默认采用 10k assets, 是否需要额外固定 50k assets smoke benchmark.
- Desktop frontend 是否在后续单独 change 中引入全局状态库, 取决于本次 workflow 拆分后的 prop drilling 结果.
- Gallery projection table 或 FTS5 是否进入下一轮设计, 取决于本次 batch read path 的验证结果.
