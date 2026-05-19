## ADDED Requirements

### Requirement: Studio Overview Library Status

系统 SHALL 提供 Studio Overview 所需的当前 library status read model, 至少包含 current library display name, root path, schema version, storage size, integrity status, integrity issue count, registered library count 和 missing library count. 该 read model SHALL 用于 Library Context 和 Settings diagnostics.

#### Scenario: 加载 Library Context Summary
- **WHEN** 桌面应用加载 Studio Console Library Context
- **THEN** 系统返回当前 library display name, root path, storage size, integrity status 和 registered library summary

#### Scenario: Missing Libraries Summary
- **WHEN** app-level registry 中存在 root path missing 的 registered libraries
- **THEN** Studio Overview 返回 missing library count, Settings diagnostics 可展示对应恢复入口

### Requirement: Provider Health Summary

系统 SHALL 提供 Studio Overview 所需 provider health summary. Provider health summary SHALL 至少包含 provider id, display name, availability, supported operations, credential/configuration state 和 recoverable error message. 该 summary 仅表达 UI-visible health, 不要求改变 provider execution strategy.

#### Scenario: Provider Available
- **WHEN** provider 配置可用且支持至少一个 operation
- **THEN** provider health summary 将该 provider 标记为 available, 并返回 supported operations

#### Scenario: Provider Configuration Error
- **WHEN** provider 缺失 credential, CLI 不可用或 configuration invalid
- **THEN** provider health summary 返回 unavailable 或 degraded state 和可恢复错误信息, 不发起 generation request

#### Scenario: Provider Health 不改变 Provider Strategy
- **WHEN** 系统刷新 provider health summary
- **THEN** 系统只执行轻量配置/capability 检查, 不改变 provider adapter execution strategy, 不创建 generation event
