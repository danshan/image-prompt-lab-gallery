## ADDED Requirements

### Requirement: Task transition ownership is centralized

Task status transition, retry classification, attempt lifecycle, event persistence, and output link semantics SHALL have a single core owner. The daemon scheduler SHALL execute ticks, provider process boundaries, cancellation markers, and log IO without duplicating core task decisions.

#### Scenario: Scheduler delegates task decisions

- **WHEN** the daemon executes, retries, cancels, or completes a task
- **THEN** state transition and output-link semantics are delegated to the core task/generation owner
- **AND** daemon-specific code only handles runtime execution concerns
