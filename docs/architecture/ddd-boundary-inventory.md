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
| Metadata suggestion create | `application::use_cases::metadata_review::CreateMetadataSuggestionUseCase` | CLI suggestion create, daemon metadata tasks, Tauri review flows | Partially migrated | Daemon constructs the focused use case from the application library adapter. Future work should expose it through the application facade instead of constructing it ad hoc. |
| Metadata review accept/reject | `application::use_cases::metadata_review::ReviewMetadataSuggestionUseCase` | CLI suggestion accept/reject, Tauri review flows | Transitional compatibility | CLI still calls the legacy service directly. The behavior owner should be the review application owner. |
| Album create/add/remove/reorder/smart query | `application::use_cases::albums::AlbumUseCase` | CLI album commands, Tauri album workflows | Transitional compatibility | Tauri uses application composition. CLI still exposes direct service helpers and should move toward the album use case. |
| Gallery query/detail/inspector read model | `application::use_cases::albums::QueryGalleryUseCase` | Tauri gallery, daemon task output lookup | Migrated query owner with large legacy adapter | Query behavior enters through application use case, but implementation remains concentrated in `library/gallery.rs`. |
| Search | `application::use_cases::albums::SearchUseCase` | CLI search, Tauri search/gallery flows | Transitional compatibility | CLI still calls `SearchService` on `LocalLibraryService`; future work should use `app.search()`. |
| Task create/list/detail/reorder/retry/duplicate/output/event | `application::use_cases::tasks::TaskUseCase` | daemon API, Tauri queue workflows | Migrated application owner with daemon orchestration concerns | Repository operations are wrapped by `TaskUseCase`; task transition and output-link policy still need stronger core ownership. |

## Explicitly Bounded Runtime Legacy Usage

The following runtime files are allowed to mention `LocalLibraryService` during this change:

| File | Current reason | Desired end state |
| --- | --- | --- |
| `crates/imglab-cli/src/main.rs` | CLI command helpers still use `LocalLibraryService` for library, search, tag, rating, album, and suggestion compatibility commands. | Move commands to application facade/use-case owners in focused waves while preserving CLI JSON. |
| `crates/imglab-daemon/src/lib.rs` | Daemon prelude imports core compatibility types for route/runtime modules. | Replace broad prelude imports with focused application/interface-contract imports. |
| `crates/imglab-daemon/src/runtime.rs` | `DaemonState` stores `SqliteImgLabApplication<FakeImageProvider>` and exposes a `service()` compatibility accessor for open-library and recovery paths. | Remove the generic service accessor or narrow it to library lifecycle once runtime paths use explicit owners. |

New runtime files must not introduce direct `LocalLibraryService` usage. If a future change needs an exception, it must update this inventory and explain why the use is compatibility or adapter-only.

## Guardrail Policy

`scripts/check-architecture.sh` now has two related checks:

1. Runtime modules must not allocate business semantics locally, such as version numbers.
2. Runtime modules must not introduce new direct `LocalLibraryService` usages outside the bounded allowlist above.

This is intentionally stricter for new files than for legacy files. It prevents boundary drift while allowing the remaining transitional surfaces to be reduced incrementally.

## Next Refactor Targets

1. Move CLI search, album, rating/tag, and metadata review commands from direct service helpers to application use-case owners.
2. Expose metadata suggestion creation through the application facade instead of constructing a focused use case in daemon runtime.
3. Replace daemon `service()` accessor with narrower lifecycle operations.
4. Split gallery read-model implementation after persistence/search workload evidence is recorded.
