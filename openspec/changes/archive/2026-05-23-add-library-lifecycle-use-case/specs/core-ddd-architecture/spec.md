## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: Runtime adapter delegates migrated behavior

- **WHEN** CLI, daemon, or Tauri code performs a migrated write flow
- **THEN** it delegates business behavior to the application/use-case boundary
- **AND** it only performs input parsing, transport mapping, process execution, logging, or error mapping owned by that runtime

#### Scenario: Library lifecycle uses application owner

- **WHEN** CLI, daemon, or Tauri code creates, opens, lists, repairs, exports, imports, renames, unregisters, checks, or summarizes a resource library
- **THEN** it calls a library lifecycle application owner
- **AND** the legacy local service remains an adapter or explicitly documented compatibility surface
