## ADDED Requirements

### Requirement: Background Automation Daemon
系统 SHALL 支持 background automation daemon. 用户在 Settings 开启后, 系统 MUST 安装或启用 OS-level background agent, 使 daemon 能在 desktop app 未启动时继续执行 automation-enabled libraries 中的 scheduled generation jobs.

#### Scenario: Enable Background Daemon
- **WHEN** 用户在 Settings 开启 background daemon
- **THEN** 系统启用 macOS LaunchAgent
- **AND** daemon 启动后写入 runtime file 和 token file

#### Scenario: Disable Background Daemon
- **WHEN** 用户在 Settings 关闭 background daemon
- **THEN** 系统禁用 LaunchAgent
- **AND** daemon 停止领取新的 schedules 和 tasks
- **AND** 当前 running attempt 默认允许 graceful drain

### Requirement: Automation Enabled Library Scanning
Background daemon SHALL only open and scan libraries that are explicitly marked `automation_enabled`.

#### Scenario: Scan Opted-In Libraries
- **WHEN** background daemon 启动
- **THEN** daemon 从 app registry 读取 automation-enabled libraries
- **AND** 只为这些 libraries 执行 schedule runner

#### Scenario: Skip Non-Opted-In Libraries
- **WHEN** library 未开启 automation
- **THEN** background daemon 不得打开或扫描该 library 的 schedules

### Requirement: Schedule Runner Loop
Daemon SHALL run a schedule runner loop separate from the existing task scheduler loop. Schedule runner SHALL create scheduled generation runs, perform prompt expansion, enqueue image generation tasks, reconcile linked task completion, and perform output post-processing.

#### Scenario: Due Schedule Enqueues Task
- **WHEN** schedule runner 发现 due scheduled generation job
- **THEN** daemon 创建 run
- **AND** daemon 创建普通 image generation task

#### Scenario: Linked Task Completes
- **WHEN** linked image generation task completed
- **THEN** schedule runner 执行 album/tag post-processing
- **AND** run 更新为 completed

### Requirement: Daemon Discovery Priority
Desktop daemon client SHALL prefer a healthy background daemon over app-owned sidecar when background daemon is enabled.

#### Scenario: Background Daemon Healthy
- **WHEN** desktop app 启动且 background daemon runtime file 指向健康 daemon
- **THEN** desktop app 复用 background daemon
- **AND** 不启动重复 sidecar

#### Scenario: Background Daemon Misconfigured
- **WHEN** background daemon enabled 但 health check 失败
- **THEN** Settings Automation 展示 recoverable diagnostic
- **AND** UI 提供 repair action
