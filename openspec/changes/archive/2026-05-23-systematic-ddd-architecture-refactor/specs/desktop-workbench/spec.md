## ADDED Requirements

### Requirement: Workflow controllers own async orchestration

Desktop workflow async orchestration SHALL live in workflow-owned controller modules. Root composition modules SHOULD remain responsible for composition and shared wiring rather than detailed cross-workflow state machines.

#### Scenario: Root controller remains composition-focused

- **WHEN** gallery, albums, review, queue, settings, logs, or updates behavior changes
- **THEN** workflow-specific async actions and pending state live in the owning workflow controller
- **AND** root composition does not become the primary owner of that workflow's state machine

### Requirement: Refresh and polling policy is explicit

Desktop refresh behavior SHALL define which workflow owns each refresh, when full refresh is required, and when debounce, background polling, stale-while-refresh, or event-driven updates are sufficient.

#### Scenario: Task completion refreshes dependent workflows

- **WHEN** a task completes and affects gallery, metadata review, albums, or logs
- **THEN** refresh fan-out follows the documented workflow ownership policy
- **AND** it avoids unnecessary repeated full refreshes
