## ADDED Requirements

### Requirement: 维护性能 Review 清理基线
系统 SHALL 将性能和代码健康 review 发现转化为可追踪的实现任务, 并在完成后记录验证结果和剩余风险.

#### Scenario: Review 发现进入实现任务
- **WHEN** `docs/PERFORMANCE_REVIEW.md` 中的发现被确认仍适用于当前代码
- **THEN** 对应发现必须映射到 OpenSpec tasks 或明确记录为 deferred 且说明原因

#### Scenario: 完成 Cleanup 后记录验证结果
- **WHEN** cleanup 实现完成
- **THEN** 系统文档必须记录已运行的验证命令, 失败原因或剩余风险

### Requirement: 建立 SQLite Sufficiency Checkpoint
系统 SHALL 在修复查询形态和索引后评估 SQLite 是否仍适合作为 resource library 主事实存储.

#### Scenario: SQLite 优化后满足目标
- **WHEN** Gallery/Search 在目标图库规模下满足可接受响应时间且无明显 lock contention
- **THEN** 系统继续使用 SQLite 作为主事实存储, 并记录后续触发 supplemental index 的条件

#### Scenario: SQLite 优化后仍不满足目标
- **WHEN** Gallery/Search 或并发写入在目标图库规模下仍不满足响应性要求
- **THEN** 系统必须提出单独 OpenSpec change, 评估 FTS5, projection table, Tantivy, DuckDB 或 PostgreSQL 等补充/替换方案

### Requirement: Cleanup 保持公共行为稳定
系统 MUST 在性能 cleanup 中保持既有 public service, desktop workflow 和 daemon API 的可见行为稳定, 除非 spec 明确声明行为变化.

#### Scenario: Refactor 不改变 Gallery 语义
- **WHEN** core read path 从 per-asset 查询重构为 batch read model
- **THEN** Gallery filters, sort 语义和返回字段必须与现有 spec 保持一致

#### Scenario: Refactor 不改变 Daemon API Contract
- **WHEN** daemon request handling 和 scheduler 内部实现被优化
- **THEN** loopback endpoint, token authentication 和 task response shape 必须保持兼容
