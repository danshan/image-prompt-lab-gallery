## ADDED Requirements

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

## MODIFIED Requirements

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
