## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: CLI tag mutation uses asset application owner

- **WHEN** CLI adds a tag to an asset
- **THEN** the CLI adapter MUST call an asset application owner
- **AND** the CLI adapter MUST NOT call the concrete local service as the primary business entrypoint
