## ADDED Requirements

### Requirement: Prompt Document Archive Action

Prompt Workspace SHALL provide a real archive action for prompt documents. Archive MUST persist through core / Tauri command in production. Archived prompt documents MUST be excluded from the default Prompt Library list, while prompt versions and prompt-to-output history remain available for restore or archived management.

#### Scenario: Archive Prompt Document

- **WHEN** 用户在 Prompt Library 中 archive 一个 prompt document
- **THEN** 系统将该 prompt document 标记为 archived
- **AND** 默认 Prompt Library list 不再返回该 prompt document
- **AND** prompt versions 和 prompt-to-output history 不被删除

#### Scenario: Archive Selected Prompt Updates Workspace

- **WHEN** 用户 archive 当前选中的 prompt document
- **THEN** Prompt Workspace 选择下一条 active prompt
- **AND** 如果没有 active prompt, Prompt Workspace 进入 empty draft state

#### Scenario: Archive Prompt Uses Real Command

- **WHEN** app running in Tauri 执行 prompt archive
- **THEN** desktop frontend 调用真实 Tauri command
- **AND** 不只修改 local mock prompt list

### Requirement: Prompt Document Restore

系统 SHALL 支持从 Settings Archived Content restore archived prompt document. Restore MUST make the prompt document visible in the default Prompt Library list again without modifying immutable prompt versions.

#### Scenario: Restore Archived Prompt

- **WHEN** 用户 restore 一个 archived prompt document
- **THEN** 系统恢复该 prompt document 为 active
- **AND** 默认 Prompt Library list 再次返回该 prompt document
- **AND** existing prompt versions 保持不变

### Requirement: Permanent Delete Archived Prompt Document

系统 SHALL 支持对 archived prompt document 执行 permanent delete. Permanent delete MUST require dry-run summary and confirmation, MUST reject active prompt documents, and MUST cascade delete prompt document facts and related history references according to resource-library lifecycle rules.

#### Scenario: Prompt Permanent Delete Dry Run

- **WHEN** 用户对 archived prompt document 请求 permanent delete dry-run
- **THEN** 系统返回将删除的 prompt versions 和 related references summary
- **AND** dry-run 不修改 Prompt Library

#### Scenario: Reject Active Prompt Permanent Delete

- **WHEN** 用户对 active prompt document 请求 permanent delete
- **THEN** 系统返回 domain error
- **AND** 不删除 prompt document 或 prompt versions

#### Scenario: Permanent Delete Archived Prompt

- **WHEN** 用户确认 permanent delete archived prompt document
- **THEN** 系统删除该 prompt document, prompt versions 和相关 prompt output history references
- **AND** 默认 Prompt Library list 和 archived prompt list 都不再返回该 prompt document
