# Tasks: Systematic DDD Architecture Refactor

## 1. Audit and Baseline

- [x] 1.1 Write `docs/architecture/ddd-systematic-code-review.md`.
- [x] 1.2 Record public contracts for CLI JSON, daemon API, Tauri commands, SQLite schema, `manifest.json`, managed file layout, and backup/restore behavior.
- [x] 1.3 Record file-size, dependency, query-path, polling, and direct legacy-service evidence.
- [x] 1.4 Run `scripts/check-architecture.sh` and record the result.

## 2. Core Boundary Consolidation

- [x] 2.1 Inventory migrated write flows and identify their primary application owners.
- [x] 2.2 Identify remaining runtime paths that use `LocalLibraryService` as a primary-looking boundary.
- [x] 2.3 Refactor or explicitly bound legacy service usage behind application/use-case or compatibility surfaces.
- [x] 2.4 Extend architecture checks for new direct runtime bypasses.

## 3. Persistence, Search, and Read-Model Hardening

- [ ] 3.1 Build or define a synthetic resource library fixture for target workload evaluation.
- [ ] 3.2 Measure gallery, search, smart album, version tree, and task queue query paths.
- [ ] 3.3 Compare SQLite tuning, SQLite FTS5/projection tables, Tantivy, DuckDB, and PostgreSQL against the documented decision criteria.
- [ ] 3.4 Document the selected persistence/search path and migration constraints before implementation.
- [ ] 3.5 Refactor read-model owners only after query evidence identifies the lowest-complexity path.

## 4. Runtime and Frontend Ownership Cleanup

- [ ] 4.1 Keep CLI, daemon, and Tauri command layers as adapters around application behavior.
- [ ] 4.2 Move task transition and output-link rules toward core task/generation ownership where they currently live in daemon orchestration.
- [ ] 4.3 Split `StudioAppController.tsx` responsibilities by workflow-owned controller boundaries.
- [ ] 4.4 Define refresh and polling policy for gallery, review, tasks, logs, and settings.

## 5. Tests, Guardrails, and Closeout

- [ ] 5.1 Move new domain/application rule tests near owning modules.
- [ ] 5.2 Keep large regression suites documented as compatibility or cross-context coverage.
- [ ] 5.3 Validate public CLI, daemon, Tauri, desktop, and persistence behavior.
- [ ] 5.4 Run `openspec validate systematic-ddd-architecture-refactor --strict`.
- [ ] 5.5 Run `openspec validate --specs --strict`.
- [ ] 5.6 Archive the change only after tasks and validation evidence are complete.
