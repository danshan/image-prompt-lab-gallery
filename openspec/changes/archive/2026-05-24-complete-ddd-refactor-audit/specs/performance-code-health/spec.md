## MODIFIED Requirements

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

#### Scenario: Completion audit records final evidence

- **WHEN** the DDD refactor is declared complete
- **THEN** architecture documentation MUST map review findings, OpenSpec requirements, boundary inventory, and verification gates to current evidence
- **AND** any residual work MUST be classified as optional future hardening, compatibility cleanup, or product capability rather than required DDD boundary migration
