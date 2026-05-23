## Purpose

Define image generation flows, provider boundaries, metadata capture, and request validation behavior.
## Requirements
### Requirement: 支持文生图

系统 SHALL 支持通过 Codex CLI imagegen adapter, OpenAI API provider 和 Grok provider 从文本 prompt 生成图片.

#### Scenario: 成功文生图

- **WHEN** 用户选择 provider, model 和 prompt 发起文生图
- **THEN** 系统调用对应 provider adapter, 保存输出图片为 asset version, 记录 generation event, 并把 generation event 绑定到新建 asset 和 output version

#### Scenario: 文生图 Metadata 可追溯

- **WHEN** 文生图成功创建新 asset
- **THEN** 该 asset 的 Gallery read model 和 asset detail read model 能通过绑定的 generation event 返回 provider, model, prompt 和 parameters

#### Scenario: 生成完成后保持当前 Workflow

- **WHEN** 用户发起 Generate Image 且生成成功
- **THEN** 系统创建 asset/version 和 generation event, Gallery 可展示生成结果, 桌面端保持用户当前 workflow

#### Scenario: 生成产生 Pending Suggestion

- **WHEN** 生成成功并创建 metadata suggestion
- **THEN** 该 suggestion 出现在 Review Inbox, Review badge 更新, 可包含 title 和 JSON schema prompt draft, 且 asset detail 可展示 review pending state

#### Scenario: Factual Generation Metadata 立即可见

- **WHEN** 用户选择新生成的 asset
- **THEN** Gallery 或 Inspector 可展示 provider, model, prompt, parameters, lineage 和 file metadata, 但 pending suggestion 的 title, schema prompt, tags, description 和 category 不得在接受前作为 canonical metadata 展示

#### Scenario: 文生图创建默认 Title

- **WHEN** 文生图成功创建新 asset 且用户尚未提供 canonical title
- **THEN** 系统基于 generation prompt 为该 asset 创建默认 title

### Requirement: 支持基于图片生成

系统 SHALL 支持以已有 asset version 作为输入执行图生图生成.

#### Scenario: 成功图生图

- **WHEN** 用户选择 source asset version 并提供 prompt
- **THEN** 系统将 source version 传给 provider, 保存输出 version, 记录 input asset version id, 并把 generation event 绑定到 output version

#### Scenario: 图生图 Metadata 可追溯

- **WHEN** 图生图成功创建 child version
- **THEN** child version 的 lineage 和 asset detail 能通过绑定的 generation event 返回 provider, model, prompt, input version 和 parameters

### Requirement: 支持 Codex CLI Adapter

系统 SHALL 支持通过本地 `codex exec` 调用 Codex CLI adapter, 并从 Codex 文本输出中解析最终生成图片路径.

#### Scenario: Codex CLI 输出图片

- **WHEN** Codex CLI adapter 输出可解析的绝对图片路径且文件存在
- **THEN** 系统将这些文件纳入 managed library, 并把 command, stdout/stderr 和解析结果记录到 generation event raw payload

### Requirement: 校验 Provider 参数

系统 MUST 在调用 provider 前校验 provider, model, prompt, input image 和 parameters. CLI 和 desktop MUST 共享一致的 generation request construction, provider dispatch, operation inference 和 input image validation 规则.

#### Scenario: 参数无效

- **WHEN** 用户提交 provider 不支持的参数组合
- **THEN** 系统返回 `InvalidGenerationParameters` 且不创建成功状态的 generation event

#### Scenario: CLI 和 Desktop 参数校验一致

- **WHEN** CLI 和 desktop 以相同 provider, prompt, input file 或 input version 发起 generation
- **THEN** 两者使用一致的 operation inference, input image validation 和 provider capability check

#### Scenario: Provider Dispatch 后不重复校验 Provider Mismatch

- **WHEN** generation orchestration 已经根据 provider name 选择具体 provider
- **THEN** provider-specific validation 只校验该 provider 拥有的参数约束, 不重复执行低收益 provider mismatch 检查

### Requirement: 归一化 Provider 错误

系统 SHALL 将 provider-specific 错误归一化为 domain error, 同时持久化 raw request/response 以便追溯.

#### Scenario: Credential 缺失

- **WHEN** provider credential 无法解析
- **THEN** 系统返回 `CredentialMissing`, 不发起远程生成请求, 并给 GUI 和 CLI 提供可恢复错误信息

#### Scenario: Codex CLI 输出路径无效

- **WHEN** Codex CLI adapter 的输出无法解析出存在的图片文件
- **THEN** 系统返回 `GenerationFailed`, 并持久化 command output 以便排查

### Requirement: Generation 支持 Provider Capability 检查

系统 MUST 在执行 generation 前检查 provider 是否支持请求的 operation 和输入组合.

#### Scenario: Provider 不支持图生图

- **WHEN** 用户以 `input_version_id` 发起图生图且当前 provider 不支持该 capability
- **THEN** 系统返回 `UnsupportedProviderCapability`, 不创建成功状态的 output version

### Requirement: Inspector Variation 入口使用当前 Version

系统 SHALL 支持从当前选中的 asset version 构造 generation request, 并将该 version 作为 `input_version_id`.

#### Scenario: 从当前 Version 发起 Variation

- **WHEN** 用户在 Inspector 中点击 `Generate variation`
- **THEN** 系统提交包含当前 version id 的 generation request, 并在 provider 支持时创建 child version

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

### Requirement: Image Generation Commit 幂等

系统 MUST 保证 image generation task output commit 幂等, daemon recovery 或 retry 不得重复创建同一 provider result 对应的 asset version 和 generation event.

#### Scenario: Recovery Finds Existing Output Link

- **WHEN** daemon recovery 发现 image task 已有 confirmed output link
- **THEN** daemon 将 task reconcile 为 completed, 且不再次调用 provider

#### Scenario: Retry After Failed Attempt

- **WHEN** image task 因 transient error retry
- **THEN** 新 attempt 只能在前一 attempt 未 committed output 时执行 provider request

### Requirement: Image Task 保留 Review-First Metadata 语义

系统 SHALL 在 image generation task 完成后创建 pending metadata suggestion, 但不得在用户 Review 接受前将 AI metadata suggestion 写入 canonical asset metadata. Gallery 和 Inspector MAY 展示 pending review state, 但 MUST 区分 canonical metadata 和 staged suggestion.

#### Scenario: Generated Asset Enters Review Inbox
- **WHEN** image generation task completed 且生成 metadata suggestion
- **THEN** Review Inbox 显示 pending suggestion, Task Detail 显示 Open review suggestion link, canonical tags, title, description, category 和 schema prompt 仍不被 suggestion 直接覆盖

#### Scenario: Inspector Shows Pending Review State
- **WHEN** 用户选择一个有 pending suggestion 的 generated asset
- **THEN** Inspector 显示 review pending state 和 Open review 入口, 但 canonical metadata 区域不把 pending suggestion 当作 confirmed metadata 展示

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

### Requirement: 基于已有 Version 生成同 Asset 新版本

系统 SHALL 支持以已有 managed asset version 作为输入执行图生图, 并将输出保存为同一 asset 下该 input version 的 child version.

#### Scenario: Focused Version 图生图成功

- **WHEN** 用户以 focused `input_version_id` 和 prompt 发起图生图
- **THEN** 系统调用支持图生图的 provider
- **AND** 创建同一 asset 下的新 version
- **AND** 新 version 的 `parent_version_id` 设置为 focused input version id
- **AND** 新 version 在 asset version tree 中显示为 focused input version 的 child node

#### Scenario: Focused Version 图生图记录事件

- **WHEN** focused version 图生图成功创建 output version
- **THEN** 系统记录 generation event
- **AND** `input_asset_version_id` 指向 focused input version
- **AND** `output_version_id` 指向 output version

#### Scenario: 图生图成功后聚焦新 Child Version

- **WHEN** Desktop 用户从 Inspector focused version 发起图生图并成功
- **THEN** 桌面端刷新 asset detail
- **AND** focused version 切换为新创建的 child version

### Requirement: 基于 Uploaded Reference 生成独立 Output Asset

系统 SHALL 支持上传本地图片作为 reference 进行图生图. 上传 reference SHALL 作为独立 managed reference asset/version 保存, output SHALL 创建为独立 generated asset/version.

#### Scenario: Uploaded Reference 图生图成功

- **WHEN** 用户以 `input_file` 和 prompt 发起图生图
- **THEN** 系统导入 input file 为 reference asset `v1`, 调用 provider, 并创建新的 generated output asset `v1`

#### Scenario: Uploaded Reference 不成为 Parent Version

- **WHEN** uploaded reference 图生图成功
- **THEN** output version 的 `parent_version_id` 为空, generation event 的 `input_asset_version_id` 指向 reference version

### Requirement: 图生图创建前检查 Provider Capability

系统 MUST 在创建 output version 前检查 provider 是否支持图生图. 对 uploaded reference workflow, 系统 MUST 在导入 reference asset 前完成 provider capability 和请求参数校验.

#### Scenario: Provider 不支持 Uploaded Reference 图生图

- **WHEN** 用户以 `input_file` 发起图生图且 provider 不支持 image-to-image
- **THEN** 系统返回 `UnsupportedProviderCapability`, 且不创建 reference asset 或 output asset

#### Scenario: Provider 执行失败后保留 Reference

- **WHEN** uploaded reference 已导入且 provider 执行失败
- **THEN** 系统保留 reference asset, 记录 failed generation event, 且不创建 output version

### Requirement: Generation request planning SHALL be shared across CLI, desktop, and daemon
System SHALL centralize provider normalization, operation inference, default model labeling, input loading, and generation request construction so transport layers do not drift.

#### Scenario: CLI and desktop prepare equivalent generation requests
- **WHEN** CLI and desktop receive equivalent generation inputs
- **THEN** they must use the same planning rules for provider id, operation, model label, input file, input version, and parameters JSON

#### Scenario: Daemon image task uses shared planning where applicable
- **WHEN** daemon executes an image generation task
- **THEN** it must use shared planning for rules that are not daemon-specific
- **AND** daemon-specific task status, log path, retry, and cancellation behavior must remain in daemon executor code

#### Scenario: Provider execution remains adapter-owned
- **WHEN** a provider-specific command or API call is executed
- **THEN** provider crates must own provider-specific command construction, authentication assumptions, output parsing, and validation
- **AND** the shared planner must not depend on provider implementation details
