## ADDED Requirements

### Requirement: Workbench 展示数字版本号

Desktop workbench SHALL 在 Gallery 和 Inspector 中使用数字 version name 展示 asset version, 不得默认使用 UUID 派生版本名称.

#### Scenario: Gallery Card 展示数字版本

- **WHEN** Gallery card 展示包含多个 versions 的 asset
- **THEN** card 显示当前 version 的 `vN` version name 和 version count

#### Scenario: Inspector Version List 展示数字版本

- **WHEN** Inspector 展示 asset versions
- **THEN** version list 按数字 version number 展示 `v1`, `v2`, `v3` 等版本名称

### Requirement: Inspector Variation 使用当前 Version

Desktop workbench SHALL 从当前选中的 asset version 发起 `Generate variation`, 并将该 version 作为 `input_version_id`.

#### Scenario: 从当前 Version 发起 Variation

- **WHEN** 用户在 Inspector 中选择某个 version 并点击 `Generate variation`
- **THEN** Desktop 提交包含该 version UUID 的 generation request, 成功后展示同一 asset 的下一数字版本

### Requirement: Workbench 展示 Uploaded Reference Source

Desktop workbench SHALL 在 output asset detail 中单独展示 uploaded reference source, 并允许用户打开 reference asset detail.

#### Scenario: 展示 Reference Source

- **WHEN** 用户查看由 uploaded reference 生成的 output asset
- **THEN** Inspector source 区域展示 reference asset/version summary, 且不把 reference version 展示为 parent version

#### Scenario: 预览 Reference Source 原图

- **WHEN** 用户查看由 uploaded reference 生成的 output asset
- **THEN** Inspector source 区域展示 reference image 缩略图
- **AND** 用户点击缩略图时打开全屏 image preview

#### Scenario: 基于 Reference Source 重新生成

- **WHEN** 用户在 Reference Source 区域点击 regenerate
- **THEN** Desktop 打开 generation composer, 让用户补充 prompt
- **AND** Desktop 提交 image-to-image task, 使用 reference version 的 managed file path 作为 `inputFile`
- **AND** 输出创建新的 generated asset, 不在 reference asset 下创建 child version

#### Scenario: 打开 Reference Asset

- **WHEN** 用户点击 reference source link
- **THEN** Workbench 打开对应 reference asset detail, 并显示 reference version 的文件上下文
