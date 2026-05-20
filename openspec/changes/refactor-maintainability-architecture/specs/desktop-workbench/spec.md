## ADDED Requirements

### Requirement: Desktop frontend SHALL organize code by workflow
The desktop frontend SHALL separate workflow UI, workflow state hooks, transport adapters, mock preview data, and pure utilities so that a change to one workflow does not require editing the top-level application entry.

#### Scenario: Gallery workflow changes are localized
- **WHEN** gallery filtering, asset selection, lightbox, or gallery refresh behavior changes
- **THEN** the change should be implementable in gallery workflow modules and shared utilities
- **AND** unrelated review, queue, settings, and album components should not require edits

#### Scenario: Preview mode data is isolated
- **WHEN** the app runs without a Tauri runtime
- **THEN** mock preview data must come from explicit preview/mock modules
- **AND** production Tauri transport code must not depend on mock-only branches for correctness

### Requirement: Desktop Tauri backend SHALL expose commands through workflow modules
The Tauri backend SHALL group command handlers by workflow and keep serializable view mapping in a dedicated boundary.

#### Scenario: Command group owns only transport concerns
- **WHEN** a command handler receives frontend input
- **THEN** it must map input to a core or daemon request, invoke the service, and map the result to a view
- **AND** it must not duplicate core business rules that belong in `imglab-core`

