## MODIFIED Requirements

### Requirement: Checksum 展示算法标签
桌面应用 SHALL 在 Inspector File section 中展示 checksum algorithm 和 checksum digest.

#### Scenario: 展示 SHA-256 Checksum

- **WHEN** 当前 file context 的 checksum algorithm 为 `SHA-256`
- **THEN** Inspector 展示 `Checksum    SHA-256: $hash`

### Requirement: 通过 Core Read Model 加载 Gallery 和 Inspector
桌面应用 MUST 通过 Tauri command 调用 Rust core 获取 Gallery card 列表和 Inspector asset detail, 不得在前端或 Tauri command 层用多个低级结果重建业务查询语义. Gallery 和 Inspector MUST 展示 core read model 返回的 provider, model, prompt, tags, albums, lineage 和 file sections. 对通过 generation flow 创建的 asset, 桌面端 MUST 展示其 provider, model 和 prompt, 不得因前端字段映射缺失而回退为占位文案. Tauri command 层 MUST NOT 保留用于 Gallery 的 direct SQLite read command.

#### Scenario: 查询 Gallery 列表

- **WHEN** 用户修改搜索, filter 或 sort 条件
- **THEN** 桌面应用调用 core 定义的 Gallery query command 并使用返回的 card read model 渲染列表

#### Scenario: 加载 Inspector 详情

- **WHEN** 用户选择某个 asset
- **THEN** 桌面应用调用 asset detail command 并展示 prompt, provider/model, tags, albums, lineage 和 file sections

#### Scenario: 展示生成资产 Metadata

- **WHEN** 用户选择一个通过 generation flow 创建的 asset
- **THEN** Gallery card 和 Inspector 展示该 asset 的 provider, model 和 prompt

#### Scenario: 禁止 Gallery Direct SQL Command

- **WHEN** 桌面端需要加载 Gallery 列表
- **THEN** Tauri invoke handler 不提供绕过 core read model 的 `gallery_items` direct SQL command
