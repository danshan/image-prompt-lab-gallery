## ADDED Requirements

### Requirement: Image Generation 通过 Task Manager 执行
系统 SHALL 将 desktop 发起的文生图和图生图作为 `image_generation` task 提交给 daemon, 由 daemon 调用 provider adapter 并记录 task attempts, logs, timeline 和 output links.

#### Scenario: Enqueue Text To Image Task
- **WHEN** 用户从 Generate workspace 提交文生图 task
- **THEN** daemon 创建 `image_generation` task, 保存 prompt 和 provider params input snapshot, 并由 scheduler 在可用 slot 中执行

#### Scenario: Enqueue Image To Image Task
- **WHEN** 用户从 Inspector 或 Generate draft 使用 source version 提交图生图 task
- **THEN** daemon 创建包含 input version id 的 `image_generation` task, 并在执行前使用 core capability checks 校验 provider 是否支持该 operation

#### Scenario: Image Task Completed
- **WHEN** image generation task 成功完成
- **THEN** 系统保存 output image 为 managed asset version, 记录 generation event, 创建或链接 pending metadata suggestion, 并在 task outputs 中记录 asset, version, generation event 和 suggestion links

#### Scenario: Image Task Failed
- **WHEN** provider adapter 返回 generation error
- **THEN** daemon 根据错误分类将 task 标记为 retry waiting, failed retryable 或 failed final, 并保留 raw request, raw response 和 attempt log

### Requirement: Image Generation Commit 幂等
系统 MUST 保证 image generation task output commit 幂等, daemon recovery 或 retry 不得重复创建同一 provider result 对应的 asset version 和 generation event.

#### Scenario: Recovery Finds Existing Output Link
- **WHEN** daemon recovery 发现 image task 已有 confirmed output link
- **THEN** daemon 将 task reconcile 为 completed, 且不再次调用 provider

#### Scenario: Retry After Failed Attempt
- **WHEN** image task 因 transient error retry
- **THEN** 新 attempt 只能在前一 attempt 未 committed output 时执行 provider request

### Requirement: Image Task 保留 Review-First Metadata 语义
系统 SHALL 在 image generation task 完成后创建 pending metadata suggestion, 但不得在用户 Review 接受前将 AI metadata suggestion 写入 canonical asset metadata.

#### Scenario: Generated Asset Enters Review Inbox
- **WHEN** image generation task completed 且生成 metadata suggestion
- **THEN** Review Inbox 显示 pending suggestion, Task Detail 显示 Open review suggestion link, canonical tags 和 schema prompt 仍不被 suggestion 直接覆盖
