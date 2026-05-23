## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: Daemon task paths use task application owner

- **WHEN** daemon transport, recovery, scheduler, or tests perform task repository operations
- **THEN** they SHOULD call the daemon task application owner
- **AND** they SHOULD NOT use a generic concrete local-service accessor as the primary task entrypoint
