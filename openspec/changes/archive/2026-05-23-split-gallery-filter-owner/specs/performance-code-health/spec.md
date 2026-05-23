## MODIFIED Requirements

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Gallery filter owner is separated

- **GIVEN** Gallery query and smart album preview share overlapping predicate semantics
- **WHEN** gallery filtering code is refactored
- **THEN** album context loading, shared predicate application, smart album preview filtering, and album-order sorting SHOULD live in a focused owner
- **AND** the refactor MUST preserve public Gallery query behavior
