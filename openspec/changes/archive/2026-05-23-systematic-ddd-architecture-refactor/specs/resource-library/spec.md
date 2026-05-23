## ADDED Requirements

### Requirement: Persistence and query engine changes require a decision gate

Resource library persistence or query engine changes SHALL pass a documented decision gate before implementation. The decision gate MUST compare local-first portability, transaction correctness, workload fit, backup/restore behavior, migration and rollback complexity, rebuild/repair story, desktop distribution cost, testability, and observability.

#### Scenario: SQLite remains sufficient

- **WHEN** tuned SQLite meets target gallery, search, smart album, version tree, and task queue workloads
- **THEN** SQLite remains the primary resource library store
- **AND** any supplemental index is deferred or scoped with a rebuild/repair plan

#### Scenario: Supplemental index is selected

- **WHEN** FTS5, projection tables, Tantivy, DuckDB, PostgreSQL, or another engine is selected
- **THEN** the design records migration, rollback, backup/restore, rebuild, repair, and compatibility behavior before implementation
