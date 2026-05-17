## MODIFIED Requirements

### Requirement: 通过 Core Read Model 加载 Gallery 和 Inspector

桌面应用 MUST 通过 Tauri command 调用 Rust core 获取 Gallery card 列表和 Inspector asset detail, 不得在前端用多个低级结果重建业务查询语义. Gallery 和 Inspector MUST 展示 core read model 返回的 provider, model, prompt, tags, albums, lineage 和 file sections. 对通过 generation flow 创建的 asset, 桌面端 MUST 展示其 provider, model 和 prompt, 不得因前端字段映射缺失而回退为占位文案.

#### Scenario: 查询 Gallery 列表

- **WHEN** 用户修改搜索, filter 或 sort 条件
- **THEN** 桌面应用调用 core 定义的 Gallery query command 并使用返回的 card read model 渲染列表

#### Scenario: 加载 Inspector 详情

- **WHEN** 用户选择某个 asset
- **THEN** 桌面应用调用 asset detail command 并展示 prompt, provider/model, tags, albums, lineage 和 file sections

#### Scenario: 展示生成资产 Metadata

- **WHEN** 用户选择一个通过 generation flow 创建的 asset
- **THEN** Gallery card 和 Inspector 展示该 asset 的 provider, model 和 prompt

## ADDED Requirements

### Requirement: Inspector 支持添加 Tag

桌面应用 SHALL 允许用户从 Inspector 给当前 asset 添加手动 tag. 写入 MUST 通过 Tauri command 调用 Rust core, 成功后刷新当前 Gallery query 和 Inspector detail.

#### Scenario: 添加 Tag 成功

- **WHEN** 用户在 Inspector 输入非空 tag 并确认添加
- **THEN** 桌面应用调用 core 写入 tag, 并在 Gallery card tag 列表与 Inspector tag 列表中展示该 tag

#### Scenario: 添加空 Tag

- **WHEN** 用户提交空白 tag
- **THEN** 桌面应用不调用写入 command, 并保持当前 Gallery 和 Inspector state 不变

### Requirement: Search 输入驱动 Core Gallery Query

桌面应用 SHALL 使用 Workspace toolbar 的 search 输入驱动 core Gallery query text 字段. 在真实 Tauri 模式下, 搜索结果 MUST 来自 `query_gallery` command, 并匹配 title, prompt 和 tags.

#### Scenario: 搜索 Prompt

- **WHEN** 用户在搜索框输入某个生成 prompt 的文本片段
- **THEN** Gallery 只展示 core 返回的匹配 asset

#### Scenario: 搜索 Tag

- **WHEN** 用户在搜索框输入某个 tag 名称
- **THEN** Gallery 展示 tags 中包含该名称片段的 asset

### Requirement: Inspector 支持编辑 Title 和星级 Rating

桌面应用 SHALL 允许用户在 Inspector 中双击当前 title 后编辑 canonical title. 桌面应用 SHALL 使用星级控件展示和修改 rating, 不得使用 `*` 文本串作为主要 rating 表达.

#### Scenario: 编辑 Title

- **WHEN** 用户在 Inspector 中双击 title, 输入新标题并确认
- **THEN** 桌面应用通过 Rust core 更新 canonical title, 并刷新 Gallery card 和 Inspector title

#### Scenario: 修改 Rating

- **WHEN** 用户在 Inspector 中点击某个星级
- **THEN** 桌面应用通过 Rust core 更新 rating, 并使用星级显示更新后的评分
