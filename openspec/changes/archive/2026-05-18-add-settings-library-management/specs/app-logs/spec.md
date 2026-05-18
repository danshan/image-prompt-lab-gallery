## MODIFIED Requirements

### Requirement: Settings Logs 保留全局日志浏览

系统 SHALL 保留 `Settings / Logs` 作为 app-owned logs 的全局浏览入口, 但 task execution 排查应优先通过 Task Detail 展示结构化 timeline 和 attempt logs. `Settings / Logs` MUST 与 `Settings / Libraries` 的资源库生命周期维护操作分离.

#### Scenario: Settings Logs 展示 Task Logs

- **WHEN** 用户打开 `Settings / Logs`
- **THEN** 桌面应用可以展示 task attempt logs 和 metadata generation logs, 包含 kind, modified time, size 和 preview

#### Scenario: Task Detail 提供上下文

- **WHEN** 用户从 Task Detail 查看日志
- **THEN** 桌面应用同时展示 task status, attempts, timeline events 和 output links, 不要求用户回到 Settings 才能理解失败原因

#### Scenario: Logs 子页不展示 Library 生命周期操作

- **WHEN** 用户打开 `Settings / Logs`
- **THEN** 桌面应用展示日志浏览和刷新能力, 且不展示 Create Library, Open Existing Library, Import Zip, Export Zip, Rename 或 Close 操作
