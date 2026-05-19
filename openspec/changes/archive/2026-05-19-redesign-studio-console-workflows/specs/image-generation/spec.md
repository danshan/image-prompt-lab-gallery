## MODIFIED Requirements

### Requirement: Image Generation 通过 Task Manager 执行

系统 SHALL 将 desktop 发起的文生图和图生图作为 `image_generation` task 提交给 daemon, 由 daemon 调用 provider adapter 并记录 task attempts, logs, timeline 和 output links. Task outputs MUST 为 Studio Console 提供打开 asset, version, generation event 和 review suggestion 的稳定 links.

#### Scenario: Image Task Completed
- **WHEN** image generation task 成功完成
- **THEN** 系统保存 output image 为 managed asset version, 记录 generation event, 创建或链接 pending metadata suggestion, 并在 task outputs 中记录 asset, version, generation event 和 suggestion links

#### Scenario: Open Generated Asset From Task
- **WHEN** Task Detail 展示 completed image task
- **THEN** 用户可以通过 output link 打开生成的 asset 或 output version

#### Scenario: Open Review Suggestion From Task
- **WHEN** image task completed 后创建 pending metadata suggestion
- **THEN** Task Detail 展示 Open review suggestion link, 用户可以跳转到 Review workspace

### Requirement: Image Task 保留 Review-First Metadata 语义

系统 SHALL 在 image generation task 完成后创建 pending metadata suggestion, 但不得在用户 Review 接受前将 AI metadata suggestion 写入 canonical asset metadata. Gallery 和 Inspector MAY 展示 pending review state, 但 MUST 区分 canonical metadata 和 staged suggestion.

#### Scenario: Generated Asset Enters Review Inbox
- **WHEN** image generation task completed 且生成 metadata suggestion
- **THEN** Review Inbox 显示 pending suggestion, Task Detail 显示 Open review suggestion link, canonical tags, title, description, category 和 schema prompt 仍不被 suggestion 直接覆盖

#### Scenario: Inspector Shows Pending Review State
- **WHEN** 用户选择一个有 pending suggestion 的 generated asset
- **THEN** Inspector 显示 review pending state 和 Open review 入口, 但 canonical metadata 区域不把 pending suggestion 当作 confirmed metadata 展示

## ADDED Requirements

### Requirement: Generation Task Origin 可追溯到 Asset Board

系统 SHALL 允许 Gallery asset board 和 Inspector 展示当前 asset/version 可追溯到的 generation task origin. 如果 asset/version 由 daemon task 创建, read model MUST 返回 task id, task status summary 或可打开 task detail 的 link.

#### Scenario: Gallery 展示 Task Origin
- **WHEN** Gallery asset board 展示由 task 创建的 asset
- **THEN** asset item 包含 task origin summary 或 Open task detail link

#### Scenario: Inspector 展示 Source Task
- **WHEN** Inspector 展示由 task 创建的 asset version
- **THEN** Inspector 展示 source task context, 用户可以打开 Task Detail

### Requirement: Generation Workflow 状态覆盖

Generation/Queue workflow SHALL 覆盖 enqueue, running, completed, retry waiting, failed, canceled 和 daemon offline states, 并为每种状态展示可执行恢复操作.

#### Scenario: Daemon Offline
- **WHEN** 用户打开 Queue 且 daemon 不可用
- **THEN** Queue 展示 daemon offline state, 保留本地 draft, 并提供 refresh 或 retry connection 操作

#### Scenario: Failed Image Task Recovery
- **WHEN** image generation task failed retryable
- **THEN** Task Detail 展示错误分类, attempt log 和 Retry 操作
