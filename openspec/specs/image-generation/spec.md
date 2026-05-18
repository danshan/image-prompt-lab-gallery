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
