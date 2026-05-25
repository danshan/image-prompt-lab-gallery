## ADDED Requirements

### Requirement: Settings Task Queue 配置最大并发数

桌面应用 SHALL 在 Settings workflow 中提供 `Task Queue` section, 用于查看和修改 daemon task queue 的最大并发执行数. 该 section MUST 使用紧凑 form control, MUST 展示当前保存状态或可恢复错误, and MUST explain that provider-level safety limits can reduce actual parallelism.

#### Scenario: 查看 Task Queue 并发配置
- **WHEN** 用户打开 `Settings / Task Queue`
- **THEN** 桌面应用展示当前最大并发执行数, 默认值和允许范围
- **AND** 当前 resource library, Gallery query 和 drawer selection 不因进入该 section 而改变

#### Scenario: 保存合法最大并发数
- **WHEN** 用户在 `Settings / Task Queue` 输入合法最大并发数并保存
- **THEN** 桌面应用调用 daemon settings API 更新配置
- **AND** 保存成功后展示新的当前值

#### Scenario: 展示非法配置错误
- **WHEN** 用户输入小于最小值, 大于最大值或无法解析的最大并发数
- **THEN** 桌面应用阻止提交或展示 daemon 返回的可恢复 validation error
- **AND** 不改变当前已保存最大并发数

#### Scenario: Daemon Offline 时保持可恢复
- **WHEN** 用户打开 `Settings / Task Queue` 且 daemon 当前不可用
- **THEN** 桌面应用展示可恢复 offline 状态
- **AND** 不阻塞 Settings 其他 sections 或当前主 workflow

## MODIFIED Requirements

### Requirement: Settings 提供 Libraries, Providers, Updates 和 Logs Sections

桌面应用 SHALL 在 Settings workflow 中提供 `Libraries`, `Archived`, `Automation`, `Task Queue`, `Providers`, `Updates` 和 `Logs` sections. Settings 默认打开 `Libraries` section. 切换 Settings section MUST NOT 改变当前 resource library, Gallery query 或 drawer selection. Settings section 导航 SHALL 由 Settings workflow 的 page-local section controls 控制, 不依赖旧 Sidebar second-level context panel.

#### Scenario: 打开 Settings 默认进入 Libraries

- **WHEN** 用户打开 Settings
- **THEN** 桌面应用展示 `Libraries` section

#### Scenario: 切换 Settings Section 不改变当前 Library

- **WHEN** 用户在 `Settings / Libraries`, `Settings / Archived`, `Settings / Automation`, `Settings / Task Queue`, `Settings / Providers`, `Settings / Updates` 和 `Settings / Logs` 之间切换
- **THEN** 当前 resource library, Gallery query 和 drawer selection 不因子页切换而改变
