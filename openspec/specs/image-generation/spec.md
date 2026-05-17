## ADDED Requirements

### Requirement: 支持文生图

系统 SHALL 支持通过 Codex CLI imagegen adapter, OpenAI API provider 和 Grok provider 从文本 prompt 生成图片.

#### Scenario: 成功文生图

- **WHEN** 用户选择 provider, model 和 prompt 发起文生图
- **THEN** 系统调用对应 provider adapter, 保存输出图片为 asset version, 记录 generation event, 并把 generation event 绑定到新建 asset 和 output version

#### Scenario: 文生图 Metadata 可追溯

- **WHEN** 文生图成功创建新 asset
- **THEN** 该 asset 的 Gallery read model 和 asset detail read model 能通过绑定的 generation event 返回 provider, model, prompt 和 parameters

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

系统 MUST 在调用 provider 前校验 provider, model, prompt, input image 和 parameters.

#### Scenario: 参数无效

- **WHEN** 用户提交 provider 不支持的参数组合
- **THEN** 系统返回 `InvalidGenerationParameters` 且不创建成功状态的 generation event

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
