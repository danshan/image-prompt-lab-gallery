## ADDED Requirements

### Requirement: Daemon implementation SHALL separate transport, routing, scheduling, and execution
The daemon SHALL keep loopback HTTP transport, route dispatch, scheduler orchestration, task execution, runtime state, log handling, and response view mapping in separate implementation boundaries.

#### Scenario: Route behavior remains stable after split
- **WHEN** daemon route handling is refactored into route modules
- **THEN** existing endpoint paths, token authentication behavior, response status codes, and response JSON shapes must remain compatible

#### Scenario: Scheduler changes are isolated
- **WHEN** scheduler runnable-work checks, recovery, retry, or tick execution changes
- **THEN** the change must be implementable in scheduler or executor modules without editing raw HTTP parsing code

#### Scenario: Executor changes are isolated
- **WHEN** image generation or metadata task execution changes
- **THEN** the change must be implementable in executor modules without editing route parsing or runtime file handling

