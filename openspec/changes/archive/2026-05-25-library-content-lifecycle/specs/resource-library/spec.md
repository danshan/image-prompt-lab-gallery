## ADDED Requirements

### Requirement: 归档 Asset Lifecycle

系统 SHALL 支持对 asset 执行 archive 和 restore. Archive MUST 是可逆的 soft delete, MUST 保留 asset rows, asset versions, managed files, album memberships, tags, metadata suggestions, generation events 和 prompt lineage. 归档状态 MUST 由 resource library 持久化, 不得只在 desktop frontend 中过滤.

#### Scenario: 归档 Asset 后默认不可见

- **WHEN** 用户归档一个 active asset
- **THEN** 系统记录该 asset 的 archived time
- **AND** 默认 Gallery query, Albums detail query 和 add-to-album source query 不再返回该 asset
- **AND** asset versions, managed files, generation events, tags, album memberships 和 metadata suggestions 保持可恢复

#### Scenario: 恢复归档 Asset

- **WHEN** 用户恢复一个 archived asset
- **THEN** 系统清除该 asset 的 archived state
- **AND** 后续默认 Gallery query 和相关 album-scoped query 可以再次返回该 asset

#### Scenario: 拒绝归档不存在的 Asset

- **WHEN** 用户请求归档不存在的 asset id
- **THEN** 系统返回明确 domain error
- **AND** 不修改 resource library

### Requirement: Archived Content Read Model

系统 SHALL 提供 archived content read model, 用于 Settings 展示 archived assets 和 archived prompt documents. Read model MUST 至少返回 item id, item type, display title, archived time, dependency summary 和 estimated file impact.

#### Scenario: 列出归档内容

- **WHEN** 用户打开 Settings Archived Content
- **THEN** 系统返回当前 library 中 archived assets 和 archived prompt documents
- **AND** 每个 item 包含可用于 restore 和 permanent delete dry-run 的 stable id

#### Scenario: 空归档列表

- **WHEN** 当前 library 没有 archived assets 或 archived prompt documents
- **THEN** 系统返回空列表
- **AND** 不把 active content 混入 archived list

### Requirement: Permanent Delete Archived Content

系统 SHALL 支持对 archived asset 或 archived prompt document 执行 permanent delete. Permanent delete MUST 先提供 dry-run summary, MUST 只允许 archived item, MUST 采用级联删除语义删除相关 SQLite facts 和 managed files. Active item MUST NOT 被 permanent delete.

#### Scenario: Permanent Delete Dry Run

- **WHEN** 用户对 archived item 请求 permanent delete dry-run
- **THEN** 系统返回 deletion summary
- **AND** summary 包含将删除的 SQLite facts, managed files, relationship rows 和 lineage/history references count
- **AND** dry-run 不修改 SQLite 或文件系统

#### Scenario: 拒绝 Permanent Delete Active Item

- **WHEN** 用户对 active asset 或 active prompt document 请求 permanent delete
- **THEN** 系统返回 domain error
- **AND** 不删除 SQLite rows
- **AND** 不删除 managed files

#### Scenario: Permanent Delete Archived Asset

- **WHEN** 用户确认 permanent delete 一个 archived asset
- **THEN** 系统级联删除该 asset 的 versions, managed files, album memberships, tags, metadata suggestions, generation events 和相关 task output references
- **AND** 后续 archived content list 和 Gallery query 均不再返回该 asset

#### Scenario: Permanent Delete Archived Prompt

- **WHEN** 用户确认 permanent delete 一个 archived prompt document
- **THEN** 系统级联删除该 prompt document, prompt versions 和相关 prompt output history references
- **AND** 后续 archived content list 和 Prompt Library list 均不再返回该 prompt document

#### Scenario: Managed File 删除失败可恢复

- **WHEN** permanent delete 的 SQLite transaction 已成功但 managed file 删除失败
- **THEN** 系统返回可恢复 issue
- **AND** 不得报告完全成功
- **AND** issue 包含无法删除的 file path 或等价定位信息

### Requirement: Merge Library Into Current Library

系统 SHALL 支持将另一个 managed source library 的创作内容 copy-merge 到当前 target library. Source library MUST 保持不变. Merge MUST 先执行 dry-run compatibility check, 再由用户确认执行 apply. Merge MUST NOT 改变 existing `Import Zip` restore-as-separate-library 语义.

#### Scenario: Merge Dry Run 成功

- **WHEN** 用户选择一个有效 source library folder 并请求 merge dry-run
- **THEN** 系统校验 source layout, manifest 和 schema version
- **AND** 系统返回将迁移的 assets, versions, tags, albums, prompts, prompt versions, generation events, metadata suggestions, file count 和 file size
- **AND** 系统报告将跳过的 runtime state
- **AND** dry-run 不修改 source library 或 target library

#### Scenario: 拒绝 Unsupported Source Schema

- **WHEN** source library schema version 高于当前应用支持版本
- **THEN** 系统拒绝 merge dry-run 和 merge apply
- **AND** 不修改 source library
- **AND** 不修改 target library

#### Scenario: 拒绝 Invalid Source Layout

- **WHEN** source library 缺少 manifest, database 或 required managed directories
- **THEN** 系统拒绝 merge dry-run 和 merge apply
- **AND** 返回可恢复错误

#### Scenario: Merge Apply 复制创作内容

- **WHEN** 用户确认 merge apply
- **THEN** 系统将 source assets, asset versions, tags, albums, prompt documents, prompt versions, generation events, metadata suggestions 和相关 lineage 复制到 target library
- **AND** target 中每个迁入实体使用新的 target-local id
- **AND** managed files 被复制到 target library 标准路径
- **AND** source library 保持不变

#### Scenario: Merge Apply 重写内部引用

- **WHEN** source generation event, asset version, album membership, tag relation 或 prompt lineage 引用 source entity id
- **THEN** merge apply 按 source-to-target ID map 写入 target-local references
- **AND** target library 不保存指向 source library entity id 的 dangling references

#### Scenario: Merge Apply 跳过 Runtime State

- **WHEN** source library 包含 queue tasks, schedules, logs, diagnostics, registry alias 或 manifest identity
- **THEN** merge apply 不迁移这些 runtime state
- **AND** dry-run summary 报告这些 records 被跳过

#### Scenario: Merge 不按名称智能合并

- **WHEN** source album 或 prompt document 与 target 中已有 item 同名
- **THEN** merge apply 创建新的 target item
- **AND** 系统可以为 display name 添加 source marker 或后缀
- **AND** 不把同名 item 的 contents 或 versions 合并到 existing item

#### Scenario: Existing Import Zip 语义不变

- **WHEN** 用户执行现有 Import Zip workflow
- **THEN** 系统仍将 backup restore 成独立 library
- **AND** manifest id 冲突时仍按 clone-on-conflict 语义处理
- **AND** 不把 backup 内容 merge 到当前 library
