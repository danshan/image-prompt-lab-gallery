## MODIFIED Requirements

### Requirement: Daemon implementation SHALL separate transport, routing, scheduling, and execution
The daemon SHALL keep loopback HTTP transport, route dispatch, scheduler orchestration, task execution, runtime state, log handling, and response view mapping in separate implementation boundaries. These boundaries SHALL be represented as real Rust modules with explicit imports and `pub(crate)` sharing where needed, rather than root-level `include!` composition.

#### Scenario: Route behavior remains stable after split
- **WHEN** daemon route handling is refactored into route modules
- **THEN** existing endpoint paths, token authentication behavior, response status codes, and response JSON shapes must remain compatible

#### Scenario: Scheduler changes are isolated
- **WHEN** scheduler runnable-work checks, recovery, retry, or tick execution changes
- **THEN** the change must be implementable in scheduler or executor modules without editing raw HTTP parsing code

#### Scenario: Executor changes are isolated
- **WHEN** image generation or metadata task execution changes
- **THEN** the change must be implementable in executor modules without editing route parsing or runtime file handling

#### Scenario: Module boundaries are explicit
- **WHEN** developers inspect `crates/imglab-daemon/src/lib.rs`
- **THEN** it declares explicit modules for runtime, transport, routing, scheduling, execution, logs, task DTOs and views
- **AND** implementation files use normal Rust imports instead of depending on a shared root `include!` scope
