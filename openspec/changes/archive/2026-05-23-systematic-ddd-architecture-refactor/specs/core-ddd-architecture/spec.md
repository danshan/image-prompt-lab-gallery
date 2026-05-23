## ADDED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, or metadata review lifecycle.

#### Scenario: Runtime adapter delegates migrated behavior

- **WHEN** CLI, daemon, or Tauri code performs a migrated write flow
- **THEN** it delegates business behavior to the application/use-case boundary
- **AND** it only performs input parsing, transport mapping, process execution, logging, or error mapping owned by that runtime

### Requirement: Legacy service usage is explicitly bounded

Legacy `library/*` service usage SHALL be documented as compatibility, infrastructure adapter, or transitional surface. New business rules MUST be added to domain/application owners.

#### Scenario: New behavior does not enter legacy service first

- **WHEN** a new business rule is added for an existing bounded context
- **THEN** the rule is implemented in the owning domain/application module
- **AND** legacy service code delegates to that owner or remains an adapter
