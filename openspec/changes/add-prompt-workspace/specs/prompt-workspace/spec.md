## ADDED Requirements

### Requirement: Prompt Library

系统 SHALL 支持在 managed resource library 内创建, 更新, 搜索和归档 prompt documents.

#### Scenario: 创建 Prompt Document

- **WHEN** 用户创建 prompt document 并提供 name 和 draft body
- **THEN** 系统保存 active prompt document
- **AND** draft body 尚不创建 prompt version

#### Scenario: 更新 Prompt Draft

- **WHEN** 用户修改 prompt document 的 body, negative prompt, style prompt, variables, preset 或 notes
- **THEN** 系统只更新 prompt document draft
- **AND** 不修改任何已保存 prompt version

#### Scenario: 搜索 Prompt Library

- **WHEN** 用户按文本搜索 Prompt Library
- **THEN** 系统按 prompt name, draft body 和 notes 返回匹配 active prompts

#### Scenario: 归档 Prompt Document

- **WHEN** 用户归档 prompt document
- **THEN** 系统将 status 设置为 archived
- **AND** 默认 Prompt Library list 不再显示该 prompt

### Requirement: Prompt Versioning

系统 SHALL 支持为 prompt document 保存 immutable prompt versions.

#### Scenario: 保存初始 Version

- **WHEN** 用户对没有 version 的 prompt document 执行 Save version
- **THEN** 系统创建 version number 为 1 的 prompt version

#### Scenario: 保存新 Version

- **WHEN** 用户对已有 version 的 prompt document 执行 Save version
- **THEN** 系统使用该 prompt document 内的下一个 version number 创建 prompt version

#### Scenario: Prompt Version 不可修改

- **WHEN** prompt version 已经创建
- **THEN** 系统不得通过 draft 更新修改该 version snapshot

#### Scenario: Restore Version To Draft

- **WHEN** 用户从旧 prompt version restore to draft
- **THEN** 系统用该 version snapshot 覆盖 prompt document draft
- **AND** 不修改旧 prompt version

### Requirement: Template Variables

系统 SHALL 支持 `{{variable}}` template variables, default values 和 required validation.

#### Scenario: Render Prompt 成功

- **WHEN** prompt version body 引用声明过的 variables 且 required values 均可解析
- **THEN** 系统生成 rendered prompt snapshot

#### Scenario: Required Variable 缺失

- **WHEN** required variable 没有 run-time value 且没有 default value
- **THEN** 系统返回 validation error
- **AND** 不创建 generation task

#### Scenario: Body 引用未声明 Variable

- **WHEN** prompt body 包含未在 schema 中声明的 variable
- **THEN** 系统返回 validation error
- **AND** 不创建 generation task

### Requirement: Generation From Prompt

系统 SHALL 支持从 prompt version 发起 image generation task.

#### Scenario: 从 Prompt Version Enqueue Task

- **WHEN** 用户从 prompt version 发起 generation
- **THEN** task input 包含 prompt version id, rendered prompt snapshot, variables, provider, model 和 parameters

#### Scenario: Generation Event 保存 Prompt Link

- **WHEN** prompt-sourced image generation task completed
- **THEN** generation event 保存 prompt snapshot
- **AND** generation event 保存 prompt version link
- **AND** output asset version 通过 generation event 可反查 prompt version

### Requirement: Prompt-To-Output History

系统 SHALL 支持查询 prompt version 的 output history.

#### Scenario: Prompt Detail 展示 Output History

- **WHEN** 用户查看 prompt version
- **THEN** 系统展示该 version 生成过的 asset, version, task 和 generation event

#### Scenario: Legacy Events 不进入 Prompt History

- **WHEN** generation event 没有 prompt version link
- **THEN** prompt output history 不包含该 event

### Requirement: Asset Prompt Lineage

系统 SHALL 支持从 asset detail 反查 prompt lineage.

#### Scenario: Linked Prompt Lineage

- **WHEN** asset version 的 generation event 有 prompt version link
- **THEN** asset detail 展示 prompt document 和 prompt version link

#### Scenario: Legacy Prompt Snapshot

- **WHEN** asset version 的 generation event 没有 prompt version link
- **THEN** asset detail 仍展示 raw prompt snapshot
- **AND** 用户可以执行 Save as Prompt

#### Scenario: Prompt Metadata 不进入 Asset Metadata

- **WHEN** 用户从 prompt version 发起 generation
- **THEN** prompt notes, template variables, style prompt 和 preset 不写入 canonical asset metadata

### Requirement: Library Compatibility

系统 SHALL 在 schema migration 后保持旧 library 可读.

#### Scenario: 旧 Library Migration

- **WHEN** 系统打开旧 schema library
- **THEN** migration 创建 prompt tables 和 generation event prompt link column
- **AND** 不批量创建 prompt documents

#### Scenario: Existing Prompt Search Still Works

- **WHEN** 用户在 Gallery/Search 搜索 legacy generation prompt 文本
- **THEN** 系统仍返回匹配 asset
