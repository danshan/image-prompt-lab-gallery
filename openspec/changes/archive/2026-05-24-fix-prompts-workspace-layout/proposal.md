## Why

Prompts workspace 在笔记本宽度下出现布局错乱: 三栏区域没有稳定撑满中间 workspace, draft name 输入框会被保存按钮挤压, parameter preset JSON 编辑区过小. 这会直接影响 Prompt Library, draft editing 和 run panel 的高频编辑体验.

## What Changes

- 调整 Prompts workspace 的 desktop 三栏 grid, 使 Library, Editor 和 Run 三栏在笔记本宽度下仍按比例填满可用 workspace.
- 调整 prompt editor header, 允许 name 输入区和 Save draft / Save version 按钮在空间不足时自然换行, 避免重叠.
- 提升 Parameter preset JSON 编辑区的默认可用高度, 便于编辑 provider, operation, model 和参数 preset.
- 不改变 prompt document, prompt version, generation run 或 persistence 语义.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `prompt-workspace`: 补充 Prompts workspace 在 compact desktop / 笔记本宽度下的布局可用性要求.

## Impact

- 影响 `apps/desktop/src/app/screens/workflows/prompts.tsx` 和 `apps/desktop/src/styles.css`.
- 不涉及 Rust core, SQLite schema, Tauri command, daemon API 或 provider contract.
- 验证范围以 desktop frontend build 和 Prompts workspace 视觉检查为主.
