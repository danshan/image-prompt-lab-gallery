## ADDED Requirements

### Requirement: 基于已有 Version 生成同 Asset 新版本

系统 SHALL 支持以已有 managed asset version 作为输入执行图生图, 并将输出保存为同一 asset 下的下一数字版本.

#### Scenario: Existing Version 图生图成功

- **WHEN** 用户以 `input_version_id` 和 prompt 发起图生图
- **THEN** 系统调用支持图生图的 provider, 创建同一 asset 下的新 version, 设置 `parent_version_id` 为 input version id, 并设置 version number 为当前最大值加 `1`

#### Scenario: Existing Version 图生图记录事件

- **WHEN** existing version 图生图成功创建 output version
- **THEN** 系统记录 generation event, 其中 `input_asset_version_id` 指向 input version, `output_version_id` 指向 output version

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
