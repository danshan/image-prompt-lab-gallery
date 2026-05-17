## MODIFIED Requirements

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
