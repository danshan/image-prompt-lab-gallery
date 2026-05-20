## ADDED Requirements

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
