## MODIFIED Requirements

### Requirement: 支持 Task State Machine

系统 SHALL 使用明确 task state machine 管理 queued, running, retry waiting, failed, canceled, interrupted 和 completed 状态.

#### Scenario: Daemon delegates cancel transition decisions

- **WHEN** client cancels queued, retry waiting, running, cancel requested, or terminal tasks
- **THEN** daemon SHOULD use the core task domain policy to decide the next task status
- **AND** daemon MAY still own cancellation marker IO and HTTP response mapping

#### Scenario: Daemon delegates recovery transition decisions

- **WHEN** daemon recovery inspects running or cancel-requested tasks after restart
- **THEN** daemon SHOULD use the core task domain policy to decide completed, interrupted retryable, or interrupted final status
- **AND** daemon MAY still own downtime detection, event persistence orchestration, and retry timestamp comparison
