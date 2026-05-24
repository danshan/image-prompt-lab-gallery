## ADDED Requirements

### Requirement: Compact Desktop Prompt Workspace Layout

系统 SHALL 在 compact desktop / 笔记本宽度下保持 Prompts workspace 的 Library, Editor 和 Run 三个工作区可见, 并让三栏填满中间 workspace 的可用宽度.

#### Scenario: 笔记本宽度展示三栏

- **WHEN** 用户在 desktop shell 的笔记本宽度查看 Prompts workspace
- **THEN** Prompt Library, Prompt Editor 和 Run 面板 SHALL 作为三栏展示
- **AND** 三栏 SHALL 使用可用 workspace 宽度而不是收缩成未撑满的窄列

#### Scenario: 小屏宽度降级为单栏

- **WHEN** 可用宽度不足以可用地展示三栏
- **THEN** Prompts workspace SHALL 降级为单栏布局
- **AND** 每个面板 SHALL 保持可达, 不发生横向覆盖

### Requirement: Prompt Draft Header Does Not Overlap

系统 SHALL 保证 prompt draft header 中的 name 输入框和保存 action 在 compact desktop 宽度下不会重叠.

#### Scenario: Name 输入框与保存按钮空间不足

- **WHEN** Prompt Editor header 的可用宽度不足以同时容纳 name 输入框和保存按钮
- **THEN** 保存 action SHALL 自然换行或收缩到安全位置
- **AND** name 输入框 SHALL 保持可编辑且不被 Save draft 或 Save version 覆盖

### Requirement: Parameter Preset Editing Space

系统 SHALL 为 Parameter preset JSON 提供比普通短字段更大的默认编辑空间.

#### Scenario: 编辑 Parameter preset JSON

- **WHEN** 用户编辑 prompt draft 的 Parameter preset JSON
- **THEN** 编辑区 SHALL 提供足够高度展示多行 provider, model, operation 和 parameters preset
- **AND** 该高度调整 SHALL 不改变其他 prompt draft 字段的业务语义
