## MODIFIED Requirements

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Gallery task origin projection is separated

- **GIVEN** gallery cards need task origin information from task outputs and task rows
- **WHEN** gallery task origin code is refactored
- **THEN** task-specific SQL, parsing, and lookup maps SHOULD live in a focused owner separate from gallery card composition
- **AND** the refactor MUST preserve gallery card task origin payloads and task origin precedence
