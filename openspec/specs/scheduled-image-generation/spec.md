# scheduled-image-generation Specification

## Purpose
TBD - created by archiving change add-scheduled-image-generation. Update Purpose after archive.
## Requirements
### Requirement: 定时图片生成 Job
系统 SHALL 支持在指定 resource library 中创建, 更新, 启用, 禁用和删除 scheduled image generation job. Job MUST 包含名称, prompt mode, 图片生成 provider/model/parameters, schedule rule, target manual album, tags, next run time 和状态.

#### Scenario: 创建 Fixed Prompt Job
- **WHEN** 用户创建 fixed prompt scheduled generation job
- **THEN** 系统保存固定 prompt, negative prompt, image provider, schedule rule, target manual album 和 tags
- **AND** 系统计算并保存第一次 `next_run_at`

#### Scenario: 创建 Dynamic Prompt Job
- **WHEN** 用户创建 dynamic prompt scheduled generation job
- **THEN** 系统保存 base prompt, dynamic prompt, prompt expansion provider/model, image provider, schedule rule, target manual album 和 tags
- **AND** 系统不得在创建时调用 prompt expansion provider

#### Scenario: 禁用 Job
- **WHEN** 用户禁用 scheduled generation job
- **THEN** 系统将 job 标记为 disabled 或 paused
- **AND** schedule runner 不得再为该 job 创建新的 run

### Requirement: Schedule Rule
系统 SHALL 支持每 N 分钟, 每 N 小时, 每天指定本地时间三种 schedule rule. Schedule rule MUST 使用 job 的 `timezone_id` 计算下一次触发时间.

#### Scenario: Interval Minutes
- **WHEN** job 配置为每 N 分钟执行
- **THEN** 系统按分钟间隔计算下一次 `next_run_at`

#### Scenario: Interval Hours
- **WHEN** job 配置为每 N 小时执行
- **THEN** 系统按小时间隔计算下一次 `next_run_at`

#### Scenario: Daily Time
- **WHEN** job 配置为每天 `HH:mm` 执行
- **THEN** 系统按 job 的 `timezone_id` 计算下一次本地时间触发

#### Scenario: DST Invalid Local Time
- **WHEN** daily schedule 的本地时间在 DST 切换中不存在
- **THEN** 系统跳到下一次 valid local occurrence
- **AND** run diagnostic 记录该时间策略

### Requirement: Run History
系统 SHALL 为每次 due trigger 创建 scheduled generation run. Run MUST 记录 scheduled time, started time, completed time, status, skipped reason, error, expanded prompt, linked image task 和 output counters.

#### Scenario: Due Job Creates Run
- **WHEN** active job 到达 `next_run_at`
- **THEN** schedule runner 创建 run record
- **AND** run 记录 `scheduled_for`

#### Scenario: Run Links Image Task
- **WHEN** schedule runner 为 run 创建 image generation task
- **THEN** run 记录 linked image task id
- **AND** Schedules workflow 可以展示 linked task 状态

### Requirement: Dynamic Prompt Expansion
系统 SHALL 在 dynamic prompt job 每次执行时调用 prompt expansion provider. Prompt expansion output MUST 作为 image generation task 的 prompt snapshot 保存, 并写入 run history.

#### Scenario: Successful Expansion
- **WHEN** dynamic prompt run 开始执行
- **THEN** daemon 调用 prompt expansion provider
- **AND** run 保存 expanded prompt 和 provider metadata
- **AND** image generation task 使用 expanded prompt

#### Scenario: Expansion Failure
- **WHEN** prompt expansion provider 返回失败
- **THEN** run 进入 failed 状态
- **AND** 系统不得创建 image generation task
- **AND** job 仍按 missed policy 计算下一次 `next_run_at`

### Requirement: Overlap And Missed Run Policy
系统 SHALL 对 scheduled generation job 使用 overlap skip 和 missed no-catch-up 策略.

#### Scenario: Previous Run Active
- **WHEN** job 到达下一次触发时间但存在非终态 run 或 linked task
- **THEN** 系统创建 skipped run
- **AND** skipped run 记录 `previous_run_active`
- **AND** job 推进到下一次 `next_run_at`

#### Scenario: Daemon Was Offline
- **WHEN** daemon 启动时发现 job 的 `next_run_at` 早于当前时间
- **THEN** 系统不得补跑历史 triggers
- **AND** 系统记录 missed/no-catch-up diagnostic
- **AND** job 从当前时间计算下一次 `next_run_at`

### Requirement: Output Album And Tags
系统 SHALL 在 linked image generation task completed 后, 将 output assets 加入 job 的 target manual album, 并应用 job 中用户指定 tags. Post-processing MUST be idempotent.

#### Scenario: Completed Task Post Processing
- **WHEN** linked image generation task completed 且包含 output assets
- **THEN** schedule runner 将每个 output asset 加入 target manual album
- **AND** schedule runner 为每个 output asset 应用 job tags
- **AND** run outputs 记录处理状态

#### Scenario: Post Processing Restart
- **WHEN** daemon 在 post-processing 中断后重启
- **THEN** schedule runner 根据 run outputs 继续未完成 output 处理
- **AND** 系统不得创建重复 run output row

#### Scenario: Target Album Deleted
- **WHEN** job 的 target manual album 已不存在
- **THEN** 系统将 run 标记为 failed
- **AND** 系统 pause 该 job
- **AND** 系统不得继续创建 image generation task

### Requirement: Schedules Workflow
Desktop SHALL provide a `Schedules` workflow for managing scheduled generation jobs and run history. Queue SHALL continue to show concrete image generation tasks, while Schedules owns recurring job management.

#### Scenario: Manage Jobs
- **WHEN** 用户打开 Schedules workflow
- **THEN** UI 展示 schedule list, selected job editor 和 run history

#### Scenario: Run History Links To Task
- **WHEN** run 已创建 linked image task
- **THEN** Schedules workflow 提供跳转到 Queue task detail 的入口

#### Scenario: Prompt Mode Validation
- **WHEN** 用户保存 fixed 或 dynamic prompt job
- **THEN** UI 和 backend 校验对应 prompt mode 的必填字段

