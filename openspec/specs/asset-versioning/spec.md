## ADDED Requirements

### Requirement: 管理逻辑 Asset

系统 SHALL 使用 asset 表示一个逻辑图片作品, 并在 asset 上维护 canonical title, description, category, rating, status, tags 和 album membership.

#### Scenario: 创建导入图片的 asset

- **WHEN** 用户导入图片且未指定已有 asset
- **THEN** 系统创建一个新 asset, 创建首个 asset version, 并将 canonical metadata 关联到 asset

### Requirement: 管理 Asset Version

系统 SHALL 使用 asset version 表示具体图片文件, 保存 file path, hash, 尺寸, MIME type, parent version 和创建时间.

#### Scenario: 基于已有图片生成新版本

- **WHEN** 用户基于某个 asset version 发起图生图
- **THEN** 系统在同一个 asset 下创建新 version, 并记录 parent version id

### Requirement: 记录 Generation Event

系统 SHALL 为每次生成请求记录 generation event, 包含 provider, model, operation type, prompt, input version, parameters, raw request, raw response, status 和错误信息.

#### Scenario: Provider 生成失败

- **WHEN** provider 返回失败或超时
- **THEN** 系统保存失败状态和 normalized error, 并保留可用于调试的 raw response 或错误摘要

### Requirement: 保留版本 Lineage

系统 SHALL 能从任意 asset version 追溯其 parent chain 和对应 generation events.

#### Scenario: 查看版本来源

- **WHEN** 用户查看某个生成 version 的 lineage
- **THEN** 系统返回该 version 的 source version, generation event, prompt 和参数摘要

### Requirement: 提供 Asset Detail Read Model

系统 SHALL 提供 asset detail read model, 聚合 asset metadata, 当前 version, generation event, tags, albums, lineage 和 file context.

#### Scenario: 查看 Asset Detail

- **WHEN** 用户选择一个 Gallery asset
- **THEN** 系统返回该 asset 的 prompt, provider/model, tags, albums, versions, lineage 和 file metadata

### Requirement: Gallery Card 展示 Version 摘要

系统 SHALL 在 Gallery card read model 中包含当前或首选 version id, version label 和 version count.

#### Scenario: 展示多版本 Asset

- **WHEN** 一个 asset 包含多个 versions
- **THEN** Gallery card read model 返回 version count 和当前展示 version 的 label 或 id

### Requirement: Lineage Detail 包含 Parent Chain 摘要

系统 SHALL 在 asset detail read model 中返回当前 version 的 parent chain 摘要.

#### Scenario: 查看当前 Version 来源

- **WHEN** 用户在 Inspector 查看当前 version lineage
- **THEN** 系统返回当前 version, parent version 和相关 generation event 摘要
