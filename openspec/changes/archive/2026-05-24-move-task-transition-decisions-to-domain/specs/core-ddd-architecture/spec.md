## MODIFIED Requirements

### Requirement: Migrated behavior has one primary application owner

Migrated write flows SHALL have one primary application use-case owner. Runtime adapters and legacy compatibility services MUST NOT reimplement business decisions for version allocation, lineage, reference source classification, generation operation inference, task transition, metadata review lifecycle, or resource library lifecycle behavior.

#### Scenario: Daemon task transition decisions use core policy

- **WHEN** daemon recovery, cancel, successful attempt, failed attempt, or canceled attempt paths decide the next task status
- **THEN** the status decision SHOULD come from the core task domain policy
- **AND** daemon code SHOULD remain responsible for runtime IO, provider execution, loopback transport, and application owner invocation
