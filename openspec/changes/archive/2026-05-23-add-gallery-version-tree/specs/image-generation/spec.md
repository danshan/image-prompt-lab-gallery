## MODIFIED Requirements

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
