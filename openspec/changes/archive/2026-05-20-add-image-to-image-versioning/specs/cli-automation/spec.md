## ADDED Requirements

### Requirement: CLI 输出数字版本信息

CLI SHALL 在 generation 和 import 输出中包含数字 version number 和 version name, 同时保留内部 UUID 供自动化使用.

#### Scenario: CLI 文生图输出 Version Number

- **WHEN** 用户通过 CLI 成功执行 text-to-image
- **THEN** CLI 输出 asset id, version id, version number 和 `vN` version name

#### Scenario: CLI Existing Version 图生图输出下一版本

- **WHEN** 用户通过 `--input-version` 成功执行 image-to-image
- **THEN** CLI 输出同一 asset 的 output version id, version number 和 `vN` version name

### Requirement: CLI Uploaded Reference 输出 Reference Summary

CLI SHALL 支持通过 `--input-file` 上传 reference image 进行图生图, 并在成功输出中包含 output asset/version summary 和 reference asset/version summary.

#### Scenario: CLI Uploaded Reference 图生图成功

- **WHEN** 用户通过 `--input-file` 成功执行 image-to-image
- **THEN** CLI 输出 output asset id, output version id, output version name, reference asset id, reference version id 和 reference version name

#### Scenario: CLI 保留 Version UUID 输入

- **WHEN** 用户通过 CLI 指定 `--input-version`
- **THEN** CLI 接受内部 version UUID 作为 machine input, 并在 human-readable output 中展示数字 version name
