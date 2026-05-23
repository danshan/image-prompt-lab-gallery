## Purpose

Define asset and asset version metadata, lineage, canonical fields, and file integrity read models.
## Requirements
### Requirement: 管理逻辑 Asset

系统 SHALL 使用 asset 表示一个逻辑图片作品, 并在 asset 上维护 canonical title, description, category, rating, status, tags 和 album membership.

#### Scenario: 创建导入图片的 asset

- **WHEN** 用户导入图片且未指定已有 asset
- **THEN** 系统创建一个新 asset, 创建首个 asset version, 并将 canonical metadata 关联到 asset

### Requirement: 管理 Asset Version

系统 SHALL 使用 asset version 表示具体图片文件, 保存 file path, checksum algorithm, checksum, 尺寸, MIME type, parent version 和创建时间. Public read model MUST 使用 checksum algorithm 和 checksum 表达文件完整性 metadata, 不得依赖历史 `sha256` 字段作为业务语义.

#### Scenario: 基于已有图片生成新版本

- **WHEN** 用户基于某个 asset version 发起图生图
- **THEN** 系统在同一个 asset 下创建新 version, 并记录 parent version id

#### Scenario: 新 Version 暴露 Canonical Checksum Metadata

- **WHEN** 系统返回 asset version read model
- **THEN** read model 包含 checksum algorithm 和 checksum, 且不要求调用方读取 `sha256`

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

### Requirement: Asset Detail 展示 Version Tree

系统 SHALL 在 asset detail read model 中返回当前 asset 的 version tree. Version tree MUST 由同一 asset 内的 `parent_version_id` 构造. 每个 tree node MUST 包含 version id, parent version id, 用户可见 tree path label, version number, image path, created time, generation summary 和 children.

#### Scenario: 单 Root Version Tree

- **WHEN** 一个 asset 只有一个 root version 且存在 child versions
- **THEN** asset detail 返回以 root version 为根的 `version_tree`
- **AND** 每个 child node 位于其 `parent_version_id` 对应的 parent node 下

#### Scenario: 点击 Version Node 聚焦 Detail

- **WHEN** 调用方请求 asset detail 并指定 current version id
- **THEN** read model 返回该 version 作为 focused version
- **AND** preview, file context, generation event 和 lineage 均基于 focused version

### Requirement: Version Tree 使用 Path-Based 用户可见名称

系统 SHALL 为 version tree node 提供 read-model-only tree path label. Root version 的 tree path label SHOULD 为 `v1`. 同一 parent 下 children MUST 按 `created_at ASC, id ASC` 排序, 并显示为 `<parent>.<sibling_index>`.

#### Scenario: Child Versions 获得 Path Label

- **WHEN** root version `v1` 有两个 children
- **THEN** 第一个 child 的 tree path label 为 `v1.1`
- **AND** 第二个 child 的 tree path label 为 `v1.2`

#### Scenario: Deep Child Version 获得 Path Label

- **WHEN** `v1.1` 有一个 child
- **THEN** 该 child 的 tree path label 为 `v1.1.1`

#### Scenario: Internal Identity 仍使用 UUID

- **WHEN** CLI, Desktop command 或 persisted relation 引用 version
- **THEN** 系统继续使用 asset version UUID
- **AND** 不要求调用方使用 tree path label 作为 machine input

### Requirement: Version Tree Degraded State

系统 SHALL 在 asset detail 中安全处理非法或历史 version tree 数据. Invalid parent links MUST NOT 使 Gallery query 或 asset detail 整体崩溃. Read model MUST 返回可展示的 degraded state 或 orphan grouping.

#### Scenario: Parent Version Missing

- **WHEN** asset version 的 `parent_version_id` 指向缺失 version
- **THEN** read model 将该 version 放入 orphan group 或 degraded node
- **AND** 返回可用于 UI 展示的 degraded tree indicator

#### Scenario: Parent Version Belongs To Different Asset

- **WHEN** asset version 的 `parent_version_id` 指向不同 asset 的 version
- **THEN** read model 不得把两个 asset 的 tree 合并
- **AND** 返回 invalid parent degraded state

#### Scenario: Cycle In Parent Links

- **WHEN** parent links 中存在 cycle
- **THEN** read model 截断 traversal
- **AND** 返回 degraded tree indicator

### Requirement: 记录 Promoted Source Relation

系统 SHALL 使用独立 source relation 记录 `Promote as new asset` 的跨 asset 来源. `parent_version_id` MUST NOT 用于表示 promoted-from source.

#### Scenario: Promote 创建 Source Relation

- **WHEN** 用户将某个 ordinary asset version promote 为新的 Gallery asset
- **THEN** 系统创建新 asset 和 root version
- **AND** 新 root version 的 `parent_version_id` 为空
- **AND** 系统记录 `promoted_from` source relation 指向原 asset version

#### Scenario: Promoted Asset Detail 展示来源

- **WHEN** 用户查看 promoted asset 的 detail
- **THEN** read model 返回 `promoted_from` summary
- **AND** summary 包含 source asset id, source version id 和 source version tree label

### Requirement: 提供 Asset Detail Read Model

系统 SHALL 提供 asset detail read model, 聚合 asset metadata, 当前 focused version, generation event, tags, albums, version tree, focused lineage, promoted source 和 file context.

#### Scenario: 查看 Asset Detail

- **WHEN** 用户选择一个 Gallery asset
- **THEN** 系统返回该 asset 的 prompt, provider/model, tags, albums, version tree, focused lineage 和 file metadata

### Requirement: Gallery Card 展示 Version 摘要

系统 SHALL 在 Gallery card read model 中包含当前或首选 version id, version tree label, version count 和 tree branch summary.

#### Scenario: 展示多分支 Asset

- **WHEN** 一个 asset 包含 branching versions
- **THEN** Gallery card read model 返回 version count, current version tree label 和 branch count

### Requirement: Lineage Detail 包含 Parent Chain 摘要

系统 SHALL 在 asset detail read model 中返回当前 version 的 parent chain 摘要.

#### Scenario: 查看当前 Version 来源

- **WHEN** 用户在 Inspector 查看当前 version lineage
- **THEN** 系统返回当前 version, parent version 和相关 generation event 摘要

### Requirement: 提供数字 Asset Version 号

系统 SHALL 为每个 asset version 维护 asset 内递增的数字版本号. 版本号 SHALL 从 `1` 开始, 并在同一个 asset 内唯一. 系统 MUST 保留 asset version UUID 作为内部稳定标识, 但用户可见版本名称 MUST 基于数字版本号展示.

#### Scenario: 新 Asset 创建首个数字版本

- **WHEN** 系统创建新的 asset 并写入首个 asset version
- **THEN** 该 asset version 的 version number 为 `1`, 用户可见 version name 为 `v1`

#### Scenario: 同一 Asset 创建下一版本

- **WHEN** 系统在已有 asset 下创建 child version
- **THEN** 新 version 的 version number 为该 asset 当前最大 version number 加 `1`

#### Scenario: 用户可见版本不使用 UUID

- **WHEN** Gallery, Inspector 或 CLI 展示 asset version 摘要
- **THEN** 系统展示 `vN` 形式的 version name, 且默认不把 version UUID 作为版本名称展示

### Requirement: 保持 Version UUID 作为内部标识

系统 MUST 保留 asset version UUID, 并继续将其用于外键, task output target, CLI machine input 和调试信息.

#### Scenario: Machine Input 使用 Version UUID

- **WHEN** CLI 或 Desktop command 需要指定已有 input version
- **THEN** 系统接受内部 version UUID 作为稳定引用, 并在输出中同时返回数字 version number

### Requirement: 区分 Parent Chain 与 Reference Source

系统 SHALL 只使用 `parent_version_id` 表示同一 asset 内的版本父子关系. 当 generation input version 属于不同 asset 时, 系统 SHALL 将其展示为 reference source, 不得把它并入 output asset parent chain.

#### Scenario: 同一 Asset Lineage

- **WHEN** 用户查看由已有 version 生成的新 version lineage
- **THEN** 系统返回同一 asset 内的 parent chain 和对应数字版本号

#### Scenario: 跨 Asset Reference Source

- **WHEN** 用户查看由 uploaded reference 生成的 output asset lineage
- **THEN** 系统返回 output asset 的自身版本号, 并单独返回 reference source version summary
