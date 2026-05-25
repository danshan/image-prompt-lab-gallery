## ADDED Requirements

### Requirement: Schedules Workflow
桌面应用 SHALL 提供独立 `Schedules` workflow. 该 workflow SHALL 支持 scheduled generation job list, job editor, enable/disable, run-now, duplicate/delete 和 run history.

#### Scenario: Open Schedules Workflow
- **WHEN** 用户切换到 Schedules workflow
- **THEN** 桌面应用展示 schedule jobs 和当前 selected job detail

#### Scenario: Run Now
- **WHEN** 用户对 job 执行 Run Now
- **THEN** 桌面应用请求 daemon 立即创建 scheduled generation run
- **AND** run 仍遵守 overlap policy

### Requirement: Settings Automation Section
Settings SHALL provide an Automation section for background daemon status, launch-at-login status, and per-library automation opt-in.

#### Scenario: Toggle Background Daemon
- **WHEN** 用户在 Settings Automation 开启或关闭 background daemon
- **THEN** 桌面应用调用 Tauri service management command
- **AND** UI 展示 daemon 和 LaunchAgent 状态

#### Scenario: Toggle Library Automation
- **WHEN** 用户切换某个 library 的 automation opt-in
- **THEN** 桌面应用更新该 library 的 `automation_enabled` 设置

### Requirement: Queue Links To Schedule Run
Queue task detail SHALL show schedule origin when an image generation task was created by scheduled generation.

#### Scenario: Open Origin Schedule Run
- **WHEN** 用户查看 scheduled generation 创建的 task detail
- **THEN** Queue 提供打开对应 schedule run 的入口
