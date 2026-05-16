## ADDED Requirements

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
