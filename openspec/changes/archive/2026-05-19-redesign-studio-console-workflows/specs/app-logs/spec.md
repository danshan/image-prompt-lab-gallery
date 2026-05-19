## MODIFIED Requirements

### Requirement: Settings Logs 保留全局日志浏览

系统 SHALL 保留 `Settings / Logs` 作为 app-owned logs 的全局浏览入口, 但 task execution 排查应优先通过 Task Detail 展示结构化 timeline, attempts, output links 和 attempt logs. `Settings / Logs` MUST 与 library lifecycle 操作和 provider diagnostics 分区清晰.

#### Scenario: Settings Logs 展示 App Logs
- **WHEN** 用户打开 `Settings / Logs`
- **THEN** 桌面应用展示 app-owned logs, task attempt logs 和 metadata generation logs, 包含 kind, modified time, size 和 preview

#### Scenario: Settings Log Deep Links To Task
- **WHEN** 某条 log 可关联到 daemon task 或 task attempt
- **THEN** Settings Logs 提供 Open task detail 入口

#### Scenario: Task Detail 提供主要调试上下文
- **WHEN** 用户从 Task Detail 查看日志
- **THEN** 桌面应用同时展示 task status, attempts, timeline events 和 output links, 不要求用户回到 Settings 才能理解失败原因

## ADDED Requirements

### Requirement: Diagnostics Overview

系统 SHALL 为 Settings 和 Library Context 提供 diagnostics overview, 至少包含 daemon status, provider health summary, recent app log summary 和可恢复配置错误. Diagnostics overview MUST NOT 暴露任意文件系统扫描能力, 仍只读取 app-owned roots 和已知 provider log roots.

#### Scenario: 查看 Provider Diagnostics
- **WHEN** 用户打开 Settings diagnostics
- **THEN** UI 展示 provider health, credential/capability status, daemon status 和可恢复错误信息

#### Scenario: Diagnostics 不扫描 System Temp Root
- **WHEN** 系统刷新 diagnostics overview
- **THEN** 系统只枚举 app-owned log roots 和已知 provider log roots, 不遍历整个 system temp directory

### Requirement: Logs Workflow 状态覆盖

Settings Logs 和 diagnostics SHALL 覆盖 loading, empty, error 和 recovery states.

#### Scenario: Logs Empty State
- **WHEN** 当前没有 app-owned logs
- **THEN** Settings Logs 展示 empty state, 不展示旧 log preview

#### Scenario: Log Preview Read Error
- **WHEN** log preview 读取失败或路径不被允许
- **THEN** Settings Logs 展示可恢复错误, 并保留 logs list 可用
