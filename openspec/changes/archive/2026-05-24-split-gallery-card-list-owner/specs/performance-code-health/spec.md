## MODIFIED Requirements

### Requirement: Hotspot refactors are ownership-based

Large files, long methods, and duplicated logic SHALL be split by ownership and change reason rather than arbitrary line count.

#### Scenario: Gallery card projection is separated

- **GIVEN** gallery card list projection combines latest versions, generation events, tags, review counts, albums, task origins, and version tree labels
- **WHEN** gallery card code is refactored
- **THEN** card-specific SQL and DTO assembly SHOULD live in a focused owner separate from `GalleryReadService` orchestration
- **AND** the refactor MUST preserve Gallery query public behavior
