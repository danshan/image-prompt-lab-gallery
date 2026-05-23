## MODIFIED Requirements

### Requirement: Core DDD boundaries isolate business rules from runtime adapters

The core MUST keep domain/application ownership separate from CLI, daemon, Tauri, provider, and desktop view adapters.

#### Scenario: Gallery read-model owners are split by change reason

- **GIVEN** gallery search, version tree, promoted source, lineage, album filter, and detail read behavior evolve for different reasons
- **WHEN** the code is changed for one of these read-model concerns
- **THEN** the implementation SHOULD route that concern through a focused owner module instead of expanding the monolithic gallery adapter
- **AND** the gallery adapter MAY keep runtime-facing DTO composition while delegating specialized read-model algorithms to those owner modules
