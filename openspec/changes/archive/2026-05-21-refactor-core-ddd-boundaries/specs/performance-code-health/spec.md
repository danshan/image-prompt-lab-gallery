## ADDED Requirements

### Requirement: DDD Boundary Refactor SHALL complete in one change

System SHALL complete the DDD boundary refactor in this change, covering core domain/application/infrastructure boundaries, runtime integration updates, frontend workflow ownership cleanup, tests, docs, and validation. The implementation MUST NOT leave the project in a half-migrated state where major write flows still depend on both old concrete local service orchestration and new application use cases as competing primary paths.

#### Scenario: Core write flows use one primary architecture

- **WHEN** import, generation, metadata review, album, gallery, task, CLI, daemon and Tauri write paths are reviewed after implementation
- **THEN** each migrated path uses the new application facade or use case boundary as its primary architecture
- **AND** old concrete service orchestration is not a competing primary path for the same behavior

#### Scenario: Change is not split into future architecture changes

- **WHEN** tasks for this change are completed
- **THEN** remaining work may include future product capabilities
- **AND** remaining work MUST NOT include unfinished core DDD boundary migration required by this proposal

### Requirement: Architecture checks SHALL enforce dependency direction

System SHALL include verification that prevents domain modules from depending on infrastructure or runtime modules, and prevents runtime layers from bypassing application/use case boundaries for migrated write flows.

#### Scenario: Dependency direction check runs

- **WHEN** maintainability verification is executed
- **THEN** the verification reports whether domain modules import SQLite, filesystem IO, daemon, Tauri, CLI parser or desktop view modules
- **AND** the verification reports whether application modules import SQLite, filesystem IO, runtime crates, infrastructure adapters or legacy library implementation modules
- **AND** any violation is fixed or explicitly documented as a blocker

#### Scenario: Runtime bypass check runs

- **WHEN** CLI, daemon or Tauri code is migrated
- **THEN** verification or review confirms runtime code calls the application facade or use case entrypoints instead of recreating domain decisions locally

#### Scenario: Desktop compatibility barrel check runs

- **WHEN** desktop workflow ownership is migrated
- **THEN** verification reports any desktop source module that imports the legacy `workbench-state` barrel as a primary state owner
- **AND** migrated workflow code imports workflow-owned modules directly

### Requirement: Complexity review SHALL cover each migrated domain

System SHALL review each migrated bounded context for reusable logic, duplication, ownership clarity, and cyclomatic complexity risk. The refactor MUST reduce or preserve complexity in critical paths; it MUST NOT hide complexity by moving large functions into newly named DDD modules.

#### Scenario: Domain complexity scan is documented

- **WHEN** a bounded context migration is completed
- **THEN** tasks or verification notes identify remaining large files, high-complexity functions, duplicated rules, and deferred cleanup if any

#### Scenario: Shared rules are not duplicated

- **WHEN** two migrated use cases need the same domain rule
- **THEN** code review confirms the rule is reused through a policy, domain service, helper, or shared application component

#### Scenario: Facade owners do not duplicate focused use cases

- **WHEN** an application facade owner is introduced to group focused use cases
- **THEN** code review confirms the owner delegates to the focused use cases or shared components
- **AND** the owner does not create a second copy of already-migrated validation, allocation or persistence command construction logic

### Requirement: Behavior-preserving refactor SHALL include public contract tests

System SHALL verify that the DDD refactor preserves public behavior for CLI JSON output, daemon loopback API, desktop workflows, provider generation behavior, and persisted library access.

#### Scenario: Public behavior checks pass

- **WHEN** implementation claims the DDD refactor is complete
- **THEN** Rust formatting, Rust core tests, daemon tests, CLI tests, desktop frontend tests, desktop build and OpenSpec validation pass
- **AND** any unavailable or failing check is documented with cause and residual risk

#### Scenario: Existing library compatibility is checked

- **WHEN** implementation completes
- **THEN** verification includes opening or exercising a pre-refactor test library fixture or equivalent compatibility flow
- **AND** the result confirms no user-visible migration is required solely because of code reorganization

### Requirement: Tests SHALL prefer owning boundaries while preserving regression coverage

System SHALL add or relocate tests so new or migrated domain rules, application use cases, infrastructure repositories and runtime adapters are tested near their owning modules. Legacy large integration test files MAY remain as compatibility regression suites when they exercise public behavior, persisted library compatibility, or cross-context flows. They MUST NOT be the primary home for newly migrated domain/application rules, and their remaining role MUST be documented in verification notes.

#### Scenario: Domain rule tests are focused

- **WHEN** asset version, reference source, task state transition or generation planning rules are migrated
- **THEN** tests for those rules live near the owning domain or application modules

#### Scenario: Infrastructure tests cover adapter behavior

- **WHEN** SQLite or filesystem adapters are migrated
- **THEN** tests verify persistence, migration/backfill and managed file behavior without duplicating unrelated domain rule tests

#### Scenario: Legacy regression tests remain documented

- **WHEN** a legacy large test file remains after the refactor
- **THEN** verification notes identify it as compatibility or cross-context regression coverage
- **AND** newly migrated reusable rules are still covered by owner-local focused tests
