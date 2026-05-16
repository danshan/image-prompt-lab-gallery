## ADDED Requirements

### Requirement: 提供 File Context Read Model
系统 SHALL 在 asset detail read model 中提供当前 version 的文件上下文, 包括 filename, relative location, MIME type, checksum 和 integrity status.

#### Scenario: 查看文件上下文
- **WHEN** 用户在 Inspector 查看 File section
- **THEN** 系统返回当前 version 的文件名, 相对位置, MIME type, checksum 和 integrity 状态

### Requirement: File Context 允许缺失的派生字段为空
系统 SHALL 在无法可靠获得 file size, dimensions 或 generation duration 时返回空值, 不得伪造真实文件元数据.

#### Scenario: 文件尺寸信息不可用
- **WHEN** 当前资源库没有记录某个 version 的尺寸信息
- **THEN** 系统返回空尺寸字段, UI 展示 unavailable 状态

### Requirement: 支持 Inspector 触发完整性复查
系统 SHALL 提供从桌面端触发资源库或当前 asset version 完整性复查的能力.

#### Scenario: 重新校验当前文件
- **WHEN** 用户在 Inspector File section 点击 re-verify
- **THEN** 系统校验当前 version 文件存在性和 checksum, 并返回更新后的 integrity 状态
