## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: Tauri album commands use album application owner

- **WHEN** Tauri commands list or create albums for the selected library
- **THEN** they MUST call the album application owner
- **AND** they MUST NOT call the concrete library compatibility service as the primary album entrypoint
