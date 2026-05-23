## MODIFIED Requirements

### Requirement: Daemon task manager routes through application task owner

Daemon task manager workflows SHALL route task operations through the application task owner while preserving loopback API behavior.

#### Scenario: Task HTTP routes use task owner

- **WHEN** daemon HTTP routes create, list, reorder, cancel, retry, duplicate, or inspect tasks
- **THEN** task persistence operations SHOULD go through the task application owner
- **AND** HTTP request parsing and response mapping remain daemon-owned
