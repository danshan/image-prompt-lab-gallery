## ADDED Requirements

### Requirement: Generation request planning SHALL be shared across CLI, desktop, and daemon
System SHALL centralize provider normalization, operation inference, default model labeling, input loading, and generation request construction so transport layers do not drift.

#### Scenario: CLI and desktop prepare equivalent generation requests
- **WHEN** CLI and desktop receive equivalent generation inputs
- **THEN** they must use the same planning rules for provider id, operation, model label, input file, input version, and parameters JSON

#### Scenario: Daemon image task uses shared planning where applicable
- **WHEN** daemon executes an image generation task
- **THEN** it must use shared planning for rules that are not daemon-specific
- **AND** daemon-specific task status, log path, retry, and cancellation behavior must remain in daemon executor code

#### Scenario: Provider execution remains adapter-owned
- **WHEN** a provider-specific command or API call is executed
- **THEN** provider crates must own provider-specific command construction, authentication assumptions, output parsing, and validation
- **AND** the shared planner must not depend on provider implementation details

