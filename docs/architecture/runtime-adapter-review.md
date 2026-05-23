# Runtime Adapter Review

## Purpose

This document records the adapter-only review for daemon, Tauri, and CLI runtime layers under the `systematic-ddd-architecture-refactor` change.

The target boundary is:

- Runtime adapters validate inputs, map DTOs, call application owners, and map errors.
- Domain/application owners hold business rules, state transitions, persistence decisions, and compatibility policies.
- Transitional direct `LocalLibraryService` usage stays explicitly inventoried until a focused use case exists.

## Daemon

| Surface | Current owner call | Adapter-only state | Remaining risk |
| --- | --- | --- | --- |
| Task create/list/detail/reorder/retry/duplicate/output/event | `app.tasks()` | Acceptable | HTTP request parsing and response mapping remain in daemon routes. |
| Scheduler transition policy | `domain::task::policies` plus `app.tasks()` | Improved | Scheduler still orchestrates execution timing and logs, which is runtime-owned. |
| Metadata suggestion task result | `app.metadata_review().create_suggestion()` | Improved | Provider text synthesis is still deferred; current fake suggestion generation is scheduler-local orchestration. |
| Gallery output lookup | `app.gallery()` | Acceptable | Read-model implementation remains large in the compatibility library adapter. |
| Library open/recovery | `app.library_lifecycle()` | Improved | Recovery still uses task compatibility calls through the legacy service accessor. |

Daemon should not allocate asset version numbers, infer canonical metadata review policy, or mutate task status without task domain/application policy.

## Tauri

| Surface | Current owner call | Adapter-only state | Remaining risk |
| --- | --- | --- | --- |
| Library commands | `app.library_lifecycle()` | Improved | Registry and managed-library persistence are still implemented by `LocalLibraryService` behind the repository port. |
| Gallery, album, review, task commands | application/use-case owners and interface DTO mapping | Acceptable | Command modules should remain thin and avoid duplicating frontend workflow state rules. |
| Daemon sidecar integration | daemon client and process discovery | Runtime-owned | This layer may own process health, token path, and sidecar discovery, but not generation persistence rules. |
| Path and file picker handling | Tauri adapter | Runtime-owned | Path normalization must not define resource-library schema behavior. |

Tauri commands should remain DTO boundaries. If a command starts choosing business defaults, allocating identifiers, or interpreting domain transitions, the logic should move to core.

## CLI

| Surface | Current owner call | Adapter-only state | Remaining risk |
| --- | --- | --- | --- |
| Search, rating, album create/add | application owners | Improved | CLI JSON contract must remain stable. |
| Metadata suggestion list/create/accept/reject | `app.metadata_review()` plus library open compatibility | Improved | The list command still opens the library through lifecycle compatibility to resolve `LibraryId`. |
| Tag add | direct library compatibility surface | Documented gap | No focused tag use case exists yet. A future `TagUseCase` or metadata command owner should take this path. |
| Library lifecycle, repair, export | `app.library_lifecycle()` | Improved | The legacy service remains the adapter, but it is no longer the CLI owner for these workflows. |

The CLI remains the largest compatibility surface because it exposes many maintenance commands directly. New CLI behavior should enter through `ImgLabApplication` owners first.

## Follow-up Guardrails

1. Keep `scripts/check-architecture.sh` blocking new runtime `LocalLibraryService` usage outside the explicit allowlist.
2. Add a focused tag use-case owner before migrating tag mutation commands.
3. Keep daemon scheduler tests near daemon orchestration, and keep task transition policy tests near core task policy.
4. Treat Tauri command growth as a signal to extract a core owner or interface contract mapper.
