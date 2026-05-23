## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: Tauri tag mutation uses asset application owner

- **WHEN** Tauri adds a tag to an asset
- **THEN** the Tauri command adapter MUST call the asset application owner
- **AND** it MUST NOT call the concrete library compatibility service as the primary tag mutation entrypoint
