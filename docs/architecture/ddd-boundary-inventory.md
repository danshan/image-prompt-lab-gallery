# DDD Boundary Inventory

## Purpose

This document records the current primary owners for migrated write flows and the explicitly bounded legacy-service usages that remain after the systematic architecture audit. It is a working inventory for the `systematic-ddd-architecture-refactor` OpenSpec change.

The goal is not to pretend the legacy `library/*` surface has disappeared. The goal is to make remaining usage explicit, keep new behavior out of that surface, and give `scripts/check-architecture.sh` enough signal to catch new runtime bypasses.

## Migrated Write Flow Owners

| Flow | Primary owner | Runtime entrypoints | Current boundary state | Notes |
| --- | --- | --- | --- | --- |
| Library create/open/list/hide/rename/unregister/repair/export | `LocalLibraryService` through `LibraryService` compatibility surface | CLI, Tauri, daemon open-library route | Transitional compatibility | Library lifecycle still has filesystem, registry, manifest, and backup behavior coupled to the legacy service. Future work should split domain policy from infrastructure adapters before changing persistence behavior. |
| Asset import | `application::use_cases::assets::AssetUseCase` | CLI `import`, Tauri import command | Migrated application owner | Version metadata and managed-file import are already routed through the application owner in CLI. Runtime adapters should not allocate version numbers. |
| Child version creation and promoted versions | `application::use_cases::assets::AssetUseCase` | Tauri/gallery workflows, generation follow-up paths | Migrated application owner | Same-asset parent validation and next-version policy belong to asset domain/application. |
| Text-to-image and image-to-image generation | `application::use_cases::generation::GenerateImageUseCase` | CLI `generate`, daemon task execution | Migrated application owner with runtime provider dispatch | Provider process execution remains runtime/provider-owned. Version persistence, reference source behavior, and generation event persistence belong to application/domain. |
| Metadata suggestion create | `application::use_cases::metadata_review::ReviewMetadataSuggestionUseCase` | CLI suggestion create, daemon metadata tasks, Tauri review flows | Migrated application owner | CLI and daemon now enter through `app.metadata_review()`. The standalone create use case remains compatible but should not be constructed ad hoc in runtime code. |
| Metadata review list/accept/reject | `application::use_cases::metadata_review::ReviewMetadataSuggestionUseCase` | CLI suggestion list/accept/reject, Tauri review flows | Migrated application owner with library-open compatibility | CLI list still opens the library through the lifecycle surface to resolve `LibraryId`; review behavior enters through the application owner. |
| Album create/add/remove/reorder/smart query | `application::use_cases::albums::AlbumUseCase` | CLI album commands, Tauri album workflows | Migrated application owner for CLI create/add and Tauri workflows | CLI create/add now use `app.albums()`. Remaining album operations should keep using this application owner as they expand. |
| Gallery query/detail/inspector read model | `application::use_cases::albums::QueryGalleryUseCase` | Tauri gallery, daemon task output lookup | Migrated query owner with large legacy adapter | Query behavior enters through application use case, but implementation remains concentrated in `library/gallery.rs`. |
| Search | `application::use_cases::albums::SearchUseCase` | CLI search, Tauri search/gallery flows | Migrated application owner for CLI search | CLI search now opens the library through the library lifecycle surface and executes search through `app.search()`. |
| Task create/list/detail/reorder/retry/duplicate/output/event | `application::use_cases::tasks::TaskUseCase` | daemon API, Tauri queue workflows | Migrated application owner with daemon orchestration concerns | Repository operations are wrapped by `TaskUseCase`; task transition and output-link policy still need stronger core ownership. |

## Explicitly Bounded Runtime Legacy Usage

The following runtime files are allowed to mention `LocalLibraryService` during this change:

| File | Current reason | Desired end state |
| --- | --- | --- |
| `crates/imglab-cli/src/main.rs` | CLI command helpers still use `LocalLibraryService` for library lifecycle, tag, and library-open compatibility. Search, rating, album create/add, and metadata suggestion review now use application owners. | Move tag and library lifecycle commands to application facade/use-case owners in focused waves while preserving CLI JSON. |
| `crates/imglab-daemon/src/lib.rs` | Daemon prelude imports core compatibility types for route/runtime modules. | Replace broad prelude imports with focused application/interface-contract imports. |
| `crates/imglab-daemon/src/runtime.rs` | `DaemonState` stores `SqliteImgLabApplication<FakeImageProvider>` and exposes a `service()` compatibility accessor for open-library and recovery paths. | Remove the generic service accessor or narrow it to library lifecycle once runtime paths use explicit owners. |

New runtime files must not introduce direct `LocalLibraryService` usage. If a future change needs an exception, it must update this inventory and explain why the use is compatibility or adapter-only.

## Guardrail Policy

`scripts/check-architecture.sh` now has two related checks:

1. Runtime modules must not allocate business semantics locally, such as version numbers.
2. Runtime modules must not introduce new direct `LocalLibraryService` usages outside the bounded allowlist above.

This is intentionally stricter for new files than for legacy files. It prevents boundary drift while allowing the remaining transitional surfaces to be reduced incrementally.

## Next Refactor Targets

1. Add a focused tag or metadata command use case, then move CLI `tag add` off the direct service helper.
2. Introduce a library lifecycle application owner for create/open/list/hide/repair/export before narrowing runtime compatibility access.
3. Replace daemon `service()` accessor with narrower lifecycle operations.
4. Split gallery read-model implementation after persistence/search workload evidence is recorded.

## Runtime Adapter Review Notes

CLI:

- Search, rating, album create/add, and metadata suggestion review paths now route through application use-case owners.
- Tag mutation remains a documented compatibility path until a focused tag use-case API covers its current CLI JSON shape.

Daemon:

- Task queue routes use the application `TaskUseCase` through `DaemonState::tasks()`.
- Scheduler runtime code still owns provider dispatch, log IO, cancellation marker checks, and retry backoff timestamp calculation.
- Task completion, cancel, and failure status decisions now use `domain::task::policies`.
- Metadata suggestion task creation now uses `app.metadata_review()` instead of constructing a use case inside daemon runtime.

Tauri:

- Command modules continue to act as path validation, input mapping, application invocation, and view/error mapping adapters.
- No new direct runtime `LocalLibraryService` usage is allowed outside the documented allowlist enforced by `scripts/check-architecture.sh`.
