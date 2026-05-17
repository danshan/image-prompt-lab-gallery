## MODIFIED Requirements

### Requirement: 管理 Asset Version
系统 SHALL 使用 asset version 表示具体图片文件, 保存 file path, checksum algorithm, checksum, 尺寸, MIME type, parent version 和创建时间. Public read model MUST 使用 checksum algorithm 和 checksum 表达文件完整性 metadata, 不得依赖历史 `sha256` 字段作为业务语义.

#### Scenario: 基于已有图片生成新版本

- **WHEN** 用户基于某个 asset version 发起图生图
- **THEN** 系统在同一个 asset 下创建新 version, 并记录 parent version id

#### Scenario: 新 Version 暴露 Canonical Checksum Metadata

- **WHEN** 系统返回 asset version read model
- **THEN** read model 包含 checksum algorithm 和 checksum, 且不要求调用方读取历史 `sha256` 字段
