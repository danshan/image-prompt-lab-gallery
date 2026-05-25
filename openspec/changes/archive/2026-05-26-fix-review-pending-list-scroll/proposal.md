## Why

Review Inbox 左侧 Pending Review 面板包含 header, batch actions, add-to-album controls 和 pending suggestion list. 当前 pending list 在内容超过面板高度时不能稳定滚动, 用户无法访问列表下方的 suggestions.

## What Changes

- 修复 Review Inbox 左侧 pending review list 的滚动容器, 让列表在固定高度 desktop workspace 中可垂直滚动.
- 保持 header, batch actions 和 add-to-album controls 固定在列表上方, 只让 suggestion list 区域滚动.
- 保持右侧 Review metadata detail 滚动行为和业务逻辑不变.
- 不改变 Review suggestion API, metadata persistence 或 canonical metadata 写入语义.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `metadata-review`: 加固 Review Inbox pending list 可达性要求, 明确 pending suggestions 列表在内容超过可用高度时必须可滚动.

## Impact

- 影响 `apps/desktop/src/styles.css` 中 Review list panel 的布局和滚动约束.
- 不涉及 Rust core, Tauri commands, SQLite schema 或 daemon task 行为.
