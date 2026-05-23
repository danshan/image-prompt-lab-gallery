## MODIFIED Requirements

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Large owner is refactored

- **WHEN** a hotspot file such as a large controller, read model, repository, or regression suite is refactored
- **THEN** the resulting files have clear owners
- **AND** tests move or remain according to those owners

#### Scenario: Gallery read model is split incrementally

- **WHEN** gallery read-model code is split
- **THEN** search, gallery list, asset detail, version tree, album filter context, and file context concerns are separated in focused waves
- **AND** each wave preserves public behavior and records remaining split targets
