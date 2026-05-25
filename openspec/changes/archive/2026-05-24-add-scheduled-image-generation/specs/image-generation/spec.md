## ADDED Requirements

### Requirement: Prompt Expansion Provider
系统 SHALL 提供 prompt expansion provider capability. Prompt expansion provider MUST 接收 base prompt, dynamic prompt, provider/model config 和 execution context, 并返回 expanded image prompt 与 provider metadata.

#### Scenario: Fake Prompt Expansion
- **WHEN** prompt expansion provider 为 `fake`
- **THEN** 系统返回 deterministic expanded prompt
- **AND** 测试可以稳定断言输出

#### Scenario: Codex CLI Prompt Expansion
- **WHEN** prompt expansion provider 为 `codex-cli`
- **THEN** daemon 通过 Codex CLI 生成 expanded image prompt
- **AND** run history 保存 provider metadata

### Requirement: Schedule Origin Prompt Snapshot
Image generation task created from scheduled generation SHALL preserve schedule origin and final prompt snapshot.

#### Scenario: Dynamic Prompt Task
- **WHEN** dynamic scheduled generation 创建 image generation task
- **THEN** task input 包含 expanded prompt
- **AND** generation event 保存该 prompt snapshot

#### Scenario: Fixed Prompt Task
- **WHEN** fixed scheduled generation 创建 image generation task
- **THEN** task input 使用 fixed prompt
- **AND** generation event 保存该 prompt snapshot
