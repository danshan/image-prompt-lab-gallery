## ADDED Requirements

### Requirement: Generate Workspace 使用 Queue-Centric Task Workflow
桌面应用 SHALL 将 Generate workspace 设计为 Batch Composer, Tasks Queue 和 Task Detail 三栏工作流, 并通过 daemon task API 展示多任务状态.

#### Scenario: 打开 Generate Workspace
- **WHEN** 用户打开 Generate workspace
- **THEN** 桌面应用展示 batch task drafts, task queue list 和 selected task detail, 并显示 daemon 连接状态

#### Scenario: 创建 Batch Draft
- **WHEN** 用户点击 Add task
- **THEN** 桌面应用创建一个独立 task draft card, card 内 prompt editor 支持多行内容且不会按 newline 拆分 task

#### Scenario: Batch Enqueue
- **WHEN** 用户点击 Enqueue all
- **THEN** 桌面应用将每个 draft 的 prompt, provider, operation, params 和 source version snapshot 作为独立 task input 提交到 daemon

#### Scenario: Structured Import
- **WHEN** 用户导入结构化 JSON task list
- **THEN** 桌面应用将 JSON 中的每个 task object 转为 draft card, 并在提交前允许用户检查和编辑

### Requirement: Tasks Queue 支持多任务管理
桌面应用 SHALL 在 Tasks Queue 中展示 running, queued, retry waiting, completed, failed 和 canceled tasks, 并提供符合 task state 的操作.

#### Scenario: 展示 Task Row
- **WHEN** daemon 返回 task list
- **THEN** 每条 task row 展示 task type, prompt summary, provider, status, wait reason, attempt count, next retry time 和 quick actions

#### Scenario: 人工排序 Queued Tasks
- **WHEN** 用户拖动或点击 move up / move down 调整 queued task 顺序
- **THEN** 桌面应用调用 daemon reorder API, 并只提交 queued tasks 的新顺序

#### Scenario: 禁止排序 Non-Queued Tasks
- **WHEN** task 状态为 running, retry waiting, completed, failed 或 canceled
- **THEN** 桌面应用不展示 drag handle 或 move up / move down 控件

#### Scenario: Task Quick Actions
- **WHEN** task 支持 cancel, retry 或 duplicate
- **THEN** 桌面应用展示对应 action, 并通过 daemon API 执行后刷新 queue 和 detail

### Requirement: Task Detail 展示 Timeline, Logs 和 Outputs
桌面应用 SHALL 为 selected task 展示 input snapshot, attempts, structured timeline, live log tail, raw log preview, output links 和错误详情.

#### Scenario: 查看 Running Task Detail
- **WHEN** 用户选择 running task
- **THEN** Task Detail 展示当前 attempt, structured timeline, live log tail, cancel action 和 input snapshot

#### Scenario: 查看 Failed Task Detail
- **WHEN** 用户选择 failed task
- **THEN** Task Detail 展示 last error, error classification, attempt history, raw log preview, retry 或 duplicate action

#### Scenario: 查看 Completed Image Task
- **WHEN** 用户选择 completed image generation task
- **THEN** Task Detail 展示 asset, version, generation event 和 review suggestion output links

### Requirement: Review Inbox 显示 Task State Mirror
桌面应用 SHALL 在 Review Inbox 中将 metadata generation 的局部 loading state 关联到 daemon task, 并允许从 Review 表单打开 task detail.

#### Scenario: Field Generation Running
- **WHEN** 用户触发 Review field generation 且 task 正在运行
- **THEN** 对应字段显示 generating 状态和 Open task detail 入口, 其他字段仍可编辑

#### Scenario: Field Generation Retry Waiting
- **WHEN** Review field generation task 处于 retry waiting
- **THEN** Review Inbox 显示 retry waiting 和 next retry time, 且不覆盖当前 field draft

#### Scenario: Field Generation Result Stale
- **WHEN** metadata field generation 完成但用户已切换 suggestion 或修改 base revision
- **THEN** Review Inbox 显示 generated result available, 不得静默覆盖当前 draft
