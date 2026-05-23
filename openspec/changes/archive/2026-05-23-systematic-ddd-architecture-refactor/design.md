# Design: Systematic DDD Architecture Refactor

## Overview

The refactor will proceed in waves. Each wave must preserve public behavior unless this change's specs explicitly define a behavior change and validation path.

## Architecture Direction

- Domain owns invariants and policies.
- Application owns use case orchestration and depends on ports.
- Infrastructure owns SQLite, filesystem, registry, provider adapters, migrations, and composition.
- Interface contracts own runtime-facing DTO compatibility and mappers.
- Legacy `library/*` remains bounded compatibility or adapter surface during migration, not the primary home for new business logic.

## Persistence and Search Decision

SQLite remains the baseline. Before changing storage architecture, the implementation must measure target workloads and compare:

- tuned SQLite schema/index/query plans,
- SQLite FTS5/projection tables,
- Tantivy embedded index,
- DuckDB analytical/read-model sidecar,
- PostgreSQL or another client-server database for future remote or multi-user needs.

Any chosen option must document portability, migration, rollback, backup/restore, repair, rebuild, testability, and distribution implications.

## Refactor Waves

1. Audit and baseline.
2. Core boundary consolidation.
3. Persistence/search decision and read-model hardening.
4. Runtime and frontend ownership cleanup.
5. Tests, guardrails, and closeout.

## Validation

Validation must include architecture checks, public contract tests, compatibility checks, OpenSpec validation, and performance evidence for persistence/search decisions.
