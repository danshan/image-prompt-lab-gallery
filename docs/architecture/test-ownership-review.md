# Test Ownership Review

## Purpose

This document records where new and existing tests should live after the systematic DDD architecture refactor. The goal is to keep rule tests close to their owners while preserving larger compatibility suites for public behavior.

## Owner-Local Tests

| Behavior | Owner | Test location | Reason |
| --- | --- | --- | --- |
| Task completion, cancellation, and failed-attempt resolution | Core task domain policy | `crates/imglab-core/src/domain/task/policies.rs` | These are pure business rules and should not require daemon scheduling. |
| Metadata suggestion list/create/accept/reject application entrypoints | Metadata review use case | `crates/imglab-core/src/application/use_cases/metadata_review.rs` | CLI and daemon should call this owner instead of testing repository behavior indirectly. |
| Album create/add/rating/search application routing | Album and search use cases | `crates/imglab-core/src/application/use_cases/albums.rs` | Runtime adapters should only verify CLI/DTO contract shape. |
| Refresh and polling policy | Desktop controller hook | `apps/desktop/src/app/hooks/controllers/refresh-policy.ts` plus existing desktop tests | The root controller should not own polling semantics directly. |

## Compatibility and Cross-Context Tests

| Suite | Kept as | Reason |
| --- | --- | --- |
| CLI command tests | Public CLI JSON compatibility | They protect command shape, dry-run behavior, and exit semantics across use-case migration. |
| Daemon scheduler tests | Cross-context runtime orchestration | They verify task execution, logs, output links, and HTTP-visible behavior. They should not become the primary place for core transition rules. |
| Desktop app tests | Workflow and UI contract regression | They protect user-facing state and rendering behavior across controller splits. |
| SQLite workload smoke script | Persistence/query evidence | It is not a unit test. It records workload shape, query plans, and timing checkpoints for DB decisions. |
| Architecture check script | Boundary guardrail | It prevents known ownership regressions that ordinary unit tests would not catch. |

## Current Closeout Evidence

The current change added owner-local task policy tests in core and kept daemon scheduler tests as cross-context coverage. Metadata review application routing now exposes list/create through `ReviewMetadataSuggestionUseCase`, so future tests for that behavior should be added at the use-case boundary before expanding CLI assertions.

Large regression suites should remain documented as compatibility coverage. They are valuable, but they should not be the only signal that a domain rule is correct.
