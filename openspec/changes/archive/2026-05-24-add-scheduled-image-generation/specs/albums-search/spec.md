## ADDED Requirements

### Requirement: Scheduled Output Album Membership
系统 SHALL 将 scheduled generation output assets 自动加入 job 指定的 target manual album. 对 smart album 执行 scheduled output membership MUST fail recoverably and pause the job.

#### Scenario: Add Output To Manual Album
- **WHEN** scheduled generation linked task completed
- **THEN** 系统将 output asset 添加到 target manual album
- **AND** 重复处理同一 output 视为 no-op

#### Scenario: Target Is Smart Album
- **WHEN** scheduled generation job 的 target album 是 smart album
- **THEN** 系统拒绝保存该 job 或 pause existing job
- **AND** 系统不得写入 `album_items`

### Requirement: Scheduled Output Tags
系统 SHALL 将 scheduled generation job 中用户指定 tags 作为 confirmed tags 应用到 output assets.

#### Scenario: Apply Tags
- **WHEN** scheduled generation output post-processing 执行
- **THEN** 系统为 output asset 应用 job tags
- **AND** tag source 记录 schedule origin

#### Scenario: Empty Tags
- **WHEN** scheduled generation job 未配置 tags
- **THEN** post-processing 仍可完成
- **AND** 系统不得创建空 tag
