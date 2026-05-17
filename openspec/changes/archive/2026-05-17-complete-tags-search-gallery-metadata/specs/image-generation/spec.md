## MODIFIED Requirements

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
