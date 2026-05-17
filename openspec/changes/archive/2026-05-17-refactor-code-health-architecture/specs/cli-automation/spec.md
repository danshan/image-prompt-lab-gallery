## MODIFIED Requirements

### Requirement: 支持 JSON Output
CLI SHALL 在查询和写操作中支持 `--json`, 输出稳定 JSON payload. 涉及 asset version 文件完整性 metadata 的 JSON payload SHALL 使用 `checksum_algorithm` 和 `checksum` 作为 canonical checksum fields, 不得把历史 `sha256` 列作为业务字段暴露.

#### Scenario: JSON 搜索输出

- **WHEN** 用户执行 `imglab search --library <path> --query <query> --json`
- **THEN** CLI 输出包含匹配 asset 列表的 JSON, 且不输出非结构化正文

#### Scenario: JSON Version 输出包含 Canonical Checksum

- **WHEN** 用户执行返回 asset version 摘要的 CLI 命令并启用 `--json`
- **THEN** CLI JSON 输出包含 `checksum_algorithm` 和 `checksum`, 且不要求调用方读取 `sha256`
