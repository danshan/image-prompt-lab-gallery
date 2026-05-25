## MODIFIED Requirements

### Requirement: Desktop Schedules Workflow

Desktop SHALL provide a `Schedules` workflow for managing scheduled generation jobs and run history. Queue SHALL continue to show concrete image generation tasks, while Schedules owns recurring job management. Scheduled job rows SHALL present the job name and schedule metadata as left-aligned primary content with stable separation from job status. The job editor SHALL provide an explicit path to return from editing an existing job to creating a new job.

#### Scenario: Manage Jobs
- **WHEN** 用户打开 Schedules workflow
- **THEN** UI 展示 schedule list, selected job editor 和 run history

#### Scenario: Job Rows Are Scannable
- **WHEN** Scheduled Jobs list 展示一个或多个 jobs
- **THEN** 每个 job row 将 name 和 provider / prompt mode / schedule metadata 居左展示
- **AND** status pill 与主内容之间保留稳定空隙, 不与 name 贴合

#### Scenario: Return From Edit To Create
- **WHEN** 用户点击现有 job 的 Edit 并进入编辑 draft
- **THEN** Schedules editor 提供 New schedule 入口
- **AND** 用户点击 New schedule 后, editor 回到可创建新 job 的默认 draft

#### Scenario: Run History Links To Task
- **WHEN** run 已创建 linked image task
- **THEN** Schedules workflow 提供跳转到 Queue task detail 的入口

#### Scenario: Prompt Mode Validation
- **WHEN** 用户保存 fixed 或 dynamic prompt job
- **THEN** UI 和 backend 校验对应 prompt mode 的必填字段
