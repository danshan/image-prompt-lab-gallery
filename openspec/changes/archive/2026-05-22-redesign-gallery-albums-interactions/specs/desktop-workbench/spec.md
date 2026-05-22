## ADDED Requirements

### Requirement: Sidebar 支持 Active View 二级导航

桌面应用 SHALL 将 Sidebar 表达为稳定主 rail 和 active-view second-level context panel. 主 rail SHALL 保持 Gallery, Albums, Review, Queue 和 Settings 作为一级导航. Second-level context panel SHALL 根据 active view 展示对应 workflow 的二级导航或 library context. Album items MUST 只在 Albums active 时作为二级项展示. Settings sections MUST 在 Settings active 时作为二级项展示.

#### Scenario: Gallery 使用 Library Context

- **WHEN** 用户打开 Gallery
- **THEN** 主 rail 高亮 Gallery
- **AND** second-level context panel 展示当前 library context, status 和 library-level 操作
- **AND** panel 不展示 album item tree

#### Scenario: Albums 展示 Album 二级列表

- **WHEN** 用户打开 Albums
- **THEN** 主 rail 高亮 Albums
- **AND** second-level context panel 展示 album search, create album, `All albums` 和当前 library 的 album items

#### Scenario: 点击 Album 二级项

- **WHEN** 用户在 Albums second-level context panel 点击某个 album item
- **THEN** 桌面应用保持 active view 为 Albums
- **AND** Albums workspace 选中该 album
- **AND** Gallery query 不因该点击而改变

#### Scenario: Settings 展示 Section 二级列表

- **WHEN** 用户打开 Settings
- **THEN** 主 rail 高亮 Settings
- **AND** second-level context panel 展示 `Libraries`, `Providers`, `Updates` 和 `Logs`
- **AND** Settings workspace 不再需要重复展示同层级 tabs

### Requirement: Gallery 作为全库图片浏览器

桌面应用 SHALL 将 Gallery 定义为 all-assets browser. Gallery 默认查询 MUST 展示当前 library 中所有普通 image assets, 且 MUST NOT 被 Albums selected album 影响. Gallery album selection MUST 作为 Gallery filter 表达, 不作为导航作用域.

#### Scenario: Gallery 默认展示全库 Assets

- **WHEN** 用户打开 Gallery 且未设置 Gallery album filter
- **THEN** Workspace 展示当前 library 中所有普通 Gallery assets
- **AND** 不按 Albums workspace 的 selected album 过滤

#### Scenario: Albums Selection 不改变 Gallery Query

- **WHEN** 用户在 Albums 中选择某个 album
- **AND** 用户返回 Gallery
- **THEN** Gallery 保持自己的 query state
- **AND** Gallery 不自动切换为该 album 的 asset board

### Requirement: Gallery Filter Surface 支持 Provider 和 Album Selector

桌面应用 SHALL 在 Gallery filter surface 中提供 search, provider selector, rating filter, review filter, album selector 和 sort control. Provider selector MUST 基于当前 library available providers. Album selector MUST 支持单选和多选 album filter, 并支持 `Not in any album` 特殊选项. Active filters SHALL 以 chips 或等价方式展示, 并支持移除单项 filter 和 clear all.

#### Scenario: Provider Selector 使用 Available Providers

- **WHEN** 当前 library 中存在 provider metadata
- **THEN** Gallery provider selector 展示当前 library 可用 providers
- **AND** 用户选择 provider 后 Gallery 只展示匹配 provider 的 assets

#### Scenario: Album Multi-Select Filter

- **WHEN** 用户在 Gallery album selector 中选择多个 albums
- **THEN** Gallery 展示属于任意 selected album 的 assets
- **AND** active filter surface 展示 selected album filter summary

#### Scenario: Not In Any Album Filter

- **WHEN** 用户在 Gallery album selector 中选择 `Not in any album`
- **THEN** 具体 album selections 被清空或禁用
- **AND** Gallery 只展示不属于任何 album 的 assets

#### Scenario: Clear Gallery Filters

- **WHEN** 用户点击 Gallery clear all filters
- **THEN** search, provider, rating, review 和 album filters 回到默认状态
- **AND** Gallery 回到 all-assets query

### Requirement: Albums Workspace 提供 Add Images Drawer

桌面应用 SHALL 在 Albums workspace 中提供 manual album 的 `Add images` 主流程. `Add images` SHALL 打开 drawer 或等价临时面板, 从 all-assets source query 中选择 assets 并批量添加到当前 manual album. Add drawer source query MUST 独立于 Gallery query.

#### Scenario: Manual Album 打开 Add Images Drawer

- **WHEN** 用户打开 manual album detail
- **THEN** Albums workspace 展示 `Add images` 操作
- **WHEN** 用户点击 `Add images`
- **THEN** 桌面应用打开 add images drawer, 并展示 all-assets source query

#### Scenario: Drawer Source Query 独立于 Gallery Query

- **WHEN** 用户在 Gallery 设置 provider, rating 或 album filters
- **AND** 用户切换到 Albums 并打开 add images drawer
- **THEN** add images drawer 使用自己的 source query state
- **AND** 不自动继承 Gallery query

#### Scenario: Drawer 默认排除已在当前 Album 中的 Assets

- **WHEN** 用户为 manual album 打开 add images drawer
- **THEN** source list 默认不展示已经属于当前 manual album 的 assets

#### Scenario: Add Selected Assets To Manual Album

- **WHEN** 用户在 add images drawer 中选择一个或多个 eligible assets 并提交
- **THEN** 桌面应用调用 core batch add 操作
- **AND** 成功后刷新 selected album contents 和 album item count
- **AND** 清空 drawer selection

#### Scenario: Smart Album 不显示 Add Images

- **WHEN** 用户打开 smart album detail
- **THEN** Albums workspace 不展示 `Add images`, remove 或 reorder 操作
- **AND** Workspace 展示 smart rule context 或 edit rule 入口

### Requirement: Gallery 和 Albums State Ownership 分离

桌面应用 MUST 分离 Gallery query state 和 Albums workspace state. Gallery query, Gallery selection 和 Gallery result MUST 属于 Gallery workflow. Albums selected album, album search, album contents query, add drawer state, add source query 和 add drawer selection MUST 属于 Albums workflow.

#### Scenario: Gallery Album Filter 不改变 Albums Selection

- **WHEN** 用户在 Gallery 中选择一个或多个 album filters
- **THEN** Gallery result 按 filter 更新
- **AND** Albums workspace 的 selected album 不因此改变

#### Scenario: Albums Add Drawer 不改变 Gallery Filters

- **WHEN** 用户在 Albums add drawer 中修改 source filters
- **THEN** add drawer source result 按 filter 更新
- **AND** Gallery query 不因此改变
