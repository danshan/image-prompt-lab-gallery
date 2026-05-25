## ADDED Requirements

### Requirement: Scheduled Generation Persistence
Resource library SHALL persist scheduled generation jobs, runs, and run outputs in SQLite. Migration MUST preserve existing assets, versions, generation events, tasks, albums, tags, and metadata suggestions.

#### Scenario: Migrate Existing Library
- **WHEN** 系统打开旧 schema library
- **THEN** migration 创建 scheduled generation tables
- **AND** 旧数据保持可读

#### Scenario: New Library Has Empty Schedules
- **WHEN** 系统创建新 library
- **THEN** scheduled generation tables 存在
- **AND** schedule list 为空

### Requirement: Library Automation Opt-In
Resource library registry SHALL persist per-library automation opt-in. Default value MUST be disabled for existing and newly registered libraries.

#### Scenario: Default Disabled
- **WHEN** library 被创建或注册
- **THEN** `automation_enabled` 默认为 false

#### Scenario: Enable Library Automation
- **WHEN** 用户在 Settings Automation 开启某个 library 的 automation
- **THEN** registry 保存 `automation_enabled = true`

### Requirement: Backup Restore Includes Schedules
Library backup and restore SHALL preserve scheduled generation job, run, and run output records inside the library database.

#### Scenario: Export Backup
- **WHEN** 用户导出 library backup zip
- **THEN** backup 包含 scheduled generation persistence tables

#### Scenario: Restore Backup
- **WHEN** 用户恢复包含 schedules 的 backup
- **THEN** restored library 保留 scheduled generation records
- **AND** automation opt-in 仍由 app registry 控制
