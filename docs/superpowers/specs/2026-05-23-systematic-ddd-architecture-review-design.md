# Systematic DDD Architecture Review Design

## Executive Summary

This work will produce a systematic architecture review and an executable OpenSpec change for the local-first image prompt lab desktop app. The review is not limited to line-level code smells. It will evaluate DDD boundaries, bounded context ownership, runtime adapter responsibilities, persistence and query engine choices, performance risks, frontend workflow ownership, code health, and verification guardrails.

The recommended delivery path is:

1. Write a repo-wide review document that records findings, evidence, impact, recommendations, and validation strategy.
2. Create an OpenSpec change named `systematic-ddd-architecture-refactor` that turns the review into staged, verifiable refactor work.
3. Defer implementation until the written review and OpenSpec design are approved.

The primary design constraint is behavior preservation. Refactors must keep CLI JSON, daemon loopback API, Tauri command payloads, resource library SQLite compatibility, manifest identity, managed file layout, and backup/restore semantics stable unless a spec explicitly defines a behavior change and its migration path.

## Goals

- Produce a systematic DDD-oriented code review across Rust core, CLI, daemon, Tauri backend, desktop frontend, and persistence.
- Identify architecture issues, performance risks, code health problems, redundant logic, long methods, oversized files, and weak test ownership.
- Define a target architecture that makes business rule ownership explicit and prevents runtime adapters or repositories from becoming competing business owners.
- Evaluate persistence and query engine options, including SQLite improvements, SQLite FTS5/projection tables, Tantivy, DuckDB, and PostgreSQL.
- Convert the review into a staged OpenSpec change with clear acceptance criteria and validation commands.

## Non-Goals

- Do not implement refactors before the review and OpenSpec design are approved.
- Do not redesign desktop visuals or layout as part of this architecture review.
- Do not force a database replacement before workload evidence justifies it.
- Do not change the resource library schema, manifest format, managed file layout, or backup/restore semantics without an explicit migration and rollback design.
- Do not introduce multi-user collaboration, cloud sync, encryption, or remote service behavior in this change.

## Evidence Sources

The review will use current worktree evidence first:

- Current module structure under `crates/imglab-core`, `crates/imglab-cli`, `crates/imglab-daemon`, `apps/desktop/src-tauri`, and `apps/desktop/src`.
- Current OpenSpec specs under `openspec/specs`.
- Existing design docs under `docs/superpowers/specs`.
- Existing guardrail script `scripts/check-architecture.sh`.
- File size scans, dependency scans, query path scans, polling scans, and direct legacy-service usage scans.

Initial evidence already identifies these hotspots:

- `apps/desktop/src/app/StudioAppController.tsx` remains a large orchestration owner.
- `crates/imglab-core/src/library/gallery.rs` combines gallery query, detail loading, version tree, album filters, smart album filtering, and file context behavior.
- `crates/imglab-core/src/library/tasks.rs` combines task repository behavior, task mutation, retry/duplicate logic, output links, and tests.
- `crates/imglab-core/src/library/tests.rs` is a large cross-context regression suite and should not become the primary home for new domain/application rules.
- `crates/imglab-cli/src/main.rs` still passes `LocalLibraryService` through multiple command helpers and needs review for primary-boundary leakage.
- Gallery, smart album, version tree, and task polling paths require performance review and workload-based validation.

## Review Document

The architecture review will be written to:

```text
docs/architecture/ddd-systematic-code-review.md
```

The document will contain:

```text
# DDD Systematic Code Review

## Executive Summary
## Scope and Constraints
## Current Architecture Snapshot
## Findings
### DDD Boundary Findings
### Bounded Context Findings
### Runtime Adapter Findings
### Persistence and Query Engine Findings
### Performance Findings
### Frontend Workflow Findings
### Code Health Findings
### Testing and Guardrail Findings
## Persistence and Query Engine Options
## Target Architecture
## Refactor Roadmap
## Verification Strategy
## Deferred / Non-Goals
```

Each finding will use this structure:

```text
Finding ID:
Severity: Critical / High / Medium / Low
Area: DDD / Runtime / Persistence / Performance / Frontend / Code Health / Tests
Evidence:
Problem:
Impact:
Recommendation:
Validation:
```

Severity will mean:

- Critical: likely data, schema, public contract, lineage, or task consistency risk.
- High: competing business owners, runtime bypass, domain dependency violation, or clear performance risk.
- Medium: oversized owner, long method, duplicated mapper/query/action logic, unclear tests owner.
- Low: local naming, minor duplication, documentation drift, or small consistency issue.

## Target DDD Architecture

The target architecture keeps `imglab-core` as the business source of truth:

```text
crates/imglab-core
  domain/
    library/
    asset/
    generation/
    metadata_review/
    album/
    task/
    shared/
  application/
    use_cases/
    ports/
    read_models/
    facade.rs
  infrastructure/
    sqlite/
    filesystem/
    registry/
    providers/
    composition.rs
  interface_contracts/
```

Layer rules:

- `domain` owns invariants and policies. It does not depend on SQLite, filesystem IO, daemon, Tauri, CLI, or frontend view types.
- `application` owns use case orchestration and depends on ports. It does not depend on concrete SQLite, filesystem, registry, runtime adapters, or legacy library implementation modules.
- `infrastructure` owns SQLite repositories, filesystem storage, registry, provider adapters, migrations, and composition. It persists decisions but does not own business decisions.
- `interface_contracts` owns runtime-facing compatibility DTOs and mappers. It must not become the primary domain model.
- legacy `library/*` may remain as compatibility or adapter surface during migration, but new business logic should not use it as the primary boundary.

## Bounded Context Ownership

The review will assess these bounded contexts:

- Resource Library: library identity, manifest, registry alias, schema compatibility, backup/restore contract.
- Asset / Version: asset aggregate, version number, version name, parent chain, reference source, promoted source, lineage.
- Generation: generation planning, operation inference, provider capability, provider output normalization.
- Metadata Review: suggestion lifecycle, confidence normalization, accept/reject semantics, canonical metadata write boundary.
- Albums / Search: manual album membership, smart album query grammar, gallery filters, sort semantics, search/read model.
- Task Manager / Daemon: task state transition, retry, attempt, event, output link, scheduler policy.

For each context, the review will identify:

- current owner,
- desired owner,
- competing owners,
- repeated rules,
- persistence boundary,
- runtime adapter touchpoints,
- test owner.

## Persistence and Query Engine Options

SQLite remains the current baseline because it supports local-first portability and a single managed library file. The review must still evaluate alternatives and supplements.

Options to evaluate:

1. SQLite with better schema, indexes, query plans, WAL, busy timeout, and transaction-boundary tuning.
2. SQLite plus FTS5 and projection tables for search and gallery read models.
3. SQLite plus Tantivy as an embedded search/faceted index.
4. DuckDB as an optional analytical/read-model sidecar.
5. PostgreSQL or another client-server database as a future path for multi-user, remote service, or cloud-sync scenarios.

The decision will use these criteria:

- local-first portability,
- transaction correctness,
- search/query workload fit,
- backup/restore semantics,
- migration and rollback risk,
- index rebuild and repair story,
- desktop distribution complexity,
- testability and observability.

No DB replacement will be recommended as implementation work unless evidence shows SQLite cannot meet target workloads after reasonable schema/index/query improvements.

## OpenSpec Change

The OpenSpec change will be:

```text
openspec/changes/systematic-ddd-architecture-refactor/
```

Expected artifacts:

```text
proposal.md
design.md
tasks.md
specs/core-ddd-architecture/spec.md
specs/performance-code-health/spec.md
specs/resource-library/spec.md
specs/task-manager-daemon/spec.md
specs/desktop-workbench/spec.md
```

Spec delta responsibilities:

- `core-ddd-architecture`: strengthen primary ownership, application ports, legacy service convergence, and bounded context rule ownership.
- `performance-code-health`: add systematic review tracking, hotspot thresholds, long method/large owner cleanup, and duplicated logic cleanup requirements.
- `resource-library`: add persistence/query engine decision gates, migration/rollback requirements, and backup/restore implications.
- `task-manager-daemon`: clarify task transition ownership, scheduler adapter boundary, retry consistency, and output link consistency.
- `desktop-workbench`: clarify frontend workflow ownership, controller size expectations, transport boundaries, refresh/polling policy, and compact desktop constraints.

## Refactor Roadmap

The OpenSpec tasks should be organized as five waves.

### Wave 1: Audit and Baseline

- Write the systematic review document.
- Record public contracts for CLI, daemon API, Tauri commands, SQLite schema, manifest, managed files, and backup/restore.
- Record hotspots using file-size, dependency, query-path, polling, and legacy-service scans.
- Run or record architecture guardrail checks.

Acceptance criteria:

- Review document exists and contains severity-ranked findings.
- Baseline public contracts are explicit.
- Hotspot evidence is concrete and reproducible.

### Wave 2: Core Boundary Consolidation

- Ensure migrated write flows have one primary application owner.
- Reduce legacy `library/*` primary ownership for new business logic.
- Keep application use cases dependent on ports rather than concrete infrastructure.
- Prevent runtime adapters from duplicating domain decisions.

Acceptance criteria:

- `scripts/check-architecture.sh` or its successor reports no domain/application dependency violations.
- CLI/daemon/Tauri migrated paths call application/use-case boundaries for business behavior.
- Legacy compatibility surfaces are documented and bounded.

### Wave 3: Persistence, Search, and Read-Model Hardening

- Build or define a synthetic resource library fixture for target workload evaluation.
- Measure gallery, search, smart album, version tree, and task queue query paths.
- Compare SQLite tuning, SQLite FTS5/projection, Tantivy, DuckDB, and PostgreSQL against the decision criteria.
- Implement only the lowest-complexity option that evidence justifies.

Acceptance criteria:

- Persistence/search decision is documented.
- Migration and rollback constraints are explicit.
- Any schema/index/projection change includes compatibility and rebuild validation.

### Wave 4: Runtime and Frontend Ownership Cleanup

- Keep CLI, daemon, and Tauri as adapter layers.
- Keep daemon scheduler focused on ticking and execution boundaries while task state semantics live in core.
- Shrink `StudioAppController.tsx` toward composition by moving async action ownership into workflow-owned controllers.
- Keep workflow screens focused on rendering and local UI interaction.

Acceptance criteria:

- Runtime adapters do not recreate version, lineage, task transition, or provider normalization rules.
- Frontend workflow actions have stable owner modules.
- Polling and refresh policy is explicit and avoids unnecessary full refresh storms.

### Wave 5: Tests, Guardrails, and Closeout

- Move new domain/application rule tests near owning modules.
- Keep large regression suites only as compatibility/cross-context coverage.
- Extend architecture checks for direct bypasses, frontend compatibility barrels, forbidden imports, and hotspot reporting.
- Validate OpenSpec specs and implementation before archive.

Acceptance criteria:

- Focused tests exist for migrated business rules.
- Compatibility tests preserve public behavior.
- OpenSpec validation and selected build/test commands pass or blockers are documented.

## Verification Strategy

Verification will be scoped by implementation wave, but the final change should select from:

```bash
scripts/check-architecture.sh
cargo fmt --all --check
cargo test -p imglab-core
cargo test -p imglab-cli
cargo test -p imglab-daemon
cargo test -p imglab-desktop
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
openspec validate systematic-ddd-architecture-refactor --strict
openspec validate --specs --strict
git diff --check
```

Performance verification should include at least one reproducible synthetic library fixture or benchmark script before recommending a persistence/search engine change.

## Risks

- A single OpenSpec change may become too large if implementation scope is not staged.
- Directly replacing SQLite would increase distribution, migration, and backup/restore complexity.
- Mechanical file splitting can hide complexity instead of reducing it.
- Moving task transition or generation output logic without contract tests can break daemon behavior.
- Frontend controller splitting can regress compact desktop workflows if workflow ownership is not preserved.

## Design Decision

Proceed with review-first architecture work:

1. Create the systematic review document.
2. Create the `systematic-ddd-architecture-refactor` OpenSpec change.
3. Ask for user review before implementation planning.
4. After approval, invoke implementation planning and execute the waves incrementally.
