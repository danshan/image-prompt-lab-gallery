## ADDED Requirements

### Requirement: Gallery 提供 Asset Archive 操作

桌面应用 SHALL 在 Gallery 中提供 archive asset 操作. 单张 archive 和 batch archive MUST 调用真实 Tauri command, 不得只更新 frontend mock state. Archive 成功后 Gallery MUST 刷新 read model 并清除已归档 selection.

#### Scenario: 单张 Archive Asset

- **WHEN** 用户在 Gallery card 上执行 archive
- **THEN** 桌面应用调用 core-backed archive command
- **AND** 成功后该 asset 从 Gallery 默认图墙移除
- **AND** 当前 selection 和 inspector state 不再指向已归档 asset

#### Scenario: Batch Archive Selected Assets

- **WHEN** 用户选择多个 Gallery assets 并执行 archive selected
- **THEN** 桌面应用对 selected assets 执行真实 archive workflow
- **AND** 成功后刷新 Gallery
- **AND** selected ids 中不再包含已归档 assets

### Requirement: Gallery 使用 Masonry 图墙

桌面应用 SHALL 在 Gallery 使用 masonry / 瀑布流图墙展示图片. Card width MUST 稳定, 图片 MUST 按真实宽高比显示, 图片宽度 MUST 填满 card, 超长图 MUST 限制最大显示高度且不得造成横向溢出.

#### Scenario: Gallery Masonry 展示

- **WHEN** 用户打开 Gallery 且存在不同比例图片
- **THEN** 图墙按稳定 card width 形成 masonry layout
- **AND** 图片按真实宽高比展示
- **AND** 长图不会撑出横向滚动

#### Scenario: Compact Desktop Masonry

- **WHEN** 桌面窗口宽度为 960px
- **THEN** Gallery masonry 仍保持主要 action 可达
- **AND** 图墙和 card text 不发生不可读重叠

### Requirement: Settings 管理 Archived Content

桌面应用 SHALL 在 Settings 中提供 Archived Content section, 展示 archived assets 和 archived prompts, 并支持 restore 和 permanent delete. Permanent delete MUST 先展示 dry-run summary 并要求确认.

#### Scenario: 查看 Archived Assets

- **WHEN** 用户打开 Settings Archived Content 的 Assets tab
- **THEN** 桌面应用通过真实 command 加载 archived assets
- **AND** 每行展示 title, archived time, dependency summary 和 restore / delete permanently actions

#### Scenario: 查看 Archived Prompts

- **WHEN** 用户打开 Settings Archived Content 的 Prompts tab
- **THEN** 桌面应用通过真实 command 加载 archived prompt documents
- **AND** 每行展示 name, archived time, dependency summary 和 restore / delete permanently actions

#### Scenario: Restore Archived Item

- **WHEN** 用户对 archived item 执行 restore
- **THEN** 桌面应用调用真实 restore command
- **AND** 成功后刷新 Archived Content list 和相关 workspace read model

#### Scenario: Permanent Delete Requires Dry Run Confirmation

- **WHEN** 用户对 archived item 点击 Delete permanently
- **THEN** 桌面应用先调用 dry-run command
- **AND** 展示级联影响摘要
- **AND** 只有用户确认后才调用 apply command

### Requirement: Settings 提供 Merge Library 操作

桌面应用 SHALL 在 Settings / Libraries 中提供 `Merge Library` 操作, 与现有 `Import Zip` 分开. Merge Library MUST 选择 source library folder, 对当前 library 执行 dry-run preview, 并在用户确认后调用真实 merge apply command.

#### Scenario: Merge Library Dry Run

- **WHEN** 用户在 Settings / Libraries 选择 Merge Library 并选择 source library folder
- **THEN** 桌面应用调用真实 dry-run command
- **AND** 展示 schema/layout status, will-copy counts, file size, renamed album/prompt count 和 skipped runtime records

#### Scenario: Merge Library Apply

- **WHEN** 用户确认 dry-run summary
- **THEN** 桌面应用调用真实 merge apply command
- **AND** 成功后刷新 Gallery, Albums, Prompt Library, Settings status 和 library storage summary

#### Scenario: No Current Library

- **WHEN** 当前没有打开 target library
- **THEN** Merge Library action 不可执行或返回可恢复错误
- **AND** 不调用 merge apply command

### Requirement: Destructive Action 不使用 Mock 或 Fake 完成

桌面 production workflow SHALL NOT 使用 mock 或 fake 逻辑替代 archive, restore, permanent delete 或 merge import. Preview fixtures MAY exist only for non-Tauri preview mode and MUST NOT be used as completion evidence for production functionality.

#### Scenario: Production Archive Uses Command

- **WHEN** app running in Tauri 执行 archive
- **THEN** desktop frontend 调用 Tauri command
- **AND** 不只修改 local mock array

#### Scenario: Production Merge Uses Command

- **WHEN** app running in Tauri 执行 merge library
- **THEN** desktop frontend 调用 Tauri command
- **AND** 不通过 frontend mock data 伪造 merge result
