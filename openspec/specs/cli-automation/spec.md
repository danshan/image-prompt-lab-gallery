## Purpose

Define CLI automation behavior, stable JSON output, and command coverage for resource library workflows.
## Requirements
### Requirement: 提供 imglab CLI

系统 SHALL 提供 `imglab` CLI, 覆盖资源库, 生成, 导入, 导出, 标签, 评分, 相册, 搜索和 suggestion review 主路径.

#### Scenario: CLI 创建资源库

- **WHEN** 用户执行 `imglab init <path> --name <name>`
- **THEN** CLI 调用 Rust core 创建资源库并输出创建结果

### Requirement: 支持 JSON Output

CLI SHALL 在查询和写操作中支持 `--json`, 输出稳定 JSON payload. 涉及 asset version 文件完整性 metadata 的 JSON payload SHALL 使用 `checksum_algorithm` 和 `checksum` 作为 canonical checksum fields, 不得把历史 `sha256` 列作为业务字段暴露.

#### Scenario: JSON 搜索输出

- **WHEN** 用户执行 `imglab search --library <path> --query <query> --json`
- **THEN** CLI 输出包含匹配 asset 列表的 JSON, 且不输出非结构化正文

#### Scenario: JSON Version 输出包含 Canonical Checksum

- **WHEN** 用户执行返回 asset version 摘要的 CLI 命令并启用 `--json`
- **THEN** CLI JSON 输出包含 `checksum_algorithm` 和 `checksum`, 且不要求调用方读取 `sha256`

### Requirement: 支持 Dry Run

CLI SHALL 对写操作支持 `--dry-run`, 预览将要发生的变化但不落盘.

#### Scenario: Dry-run 导入图片

- **WHEN** 用户执行 `imglab import --library <path> <file> --dry-run --json`
- **THEN** CLI 返回预计创建的 asset/version 摘要, 且资源库文件和 SQLite 不发生变化

### Requirement: 稳定错误输出

CLI SHALL 将 core domain errors 映射为稳定 exit code 和 JSON error fields: `code`, `message`, `details`, `recoverable`.

#### Scenario: 资源库不存在

- **WHEN** 用户对不存在的 library path 执行命令
- **THEN** CLI 返回非零 exit code 和 `LibraryNotFound` JSON error

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

