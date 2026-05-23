## ADDED Requirements

### Requirement: Systematic review findings are tracked to implementation tasks

Architecture review findings SHALL be recorded with severity, area, evidence, impact, recommendation, and validation. High and critical findings MUST map to OpenSpec tasks or be explicitly deferred with rationale.

#### Scenario: Finding maps to task

- **WHEN** a review finding is marked High or Critical
- **THEN** the finding has a corresponding task, validation item, or explicit deferred decision

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Large owner is refactored

- **WHEN** a hotspot file such as a large controller, read model, repository, or regression suite is refactored
- **THEN** the resulting files have clear owners
- **AND** tests move or remain according to those owners

### Requirement: Persistence performance decisions use workload evidence

Performance refactors for gallery, search, smart albums, version tree, and task queue SHALL use workload evidence before choosing a storage or indexing architecture.

#### Scenario: Query engine decision is evidence-based

- **WHEN** implementation proposes SQLite tuning, FTS5/projection tables, Tantivy, DuckDB, or PostgreSQL
- **THEN** the proposal includes workload evidence, migration impact, and backup/restore implications
