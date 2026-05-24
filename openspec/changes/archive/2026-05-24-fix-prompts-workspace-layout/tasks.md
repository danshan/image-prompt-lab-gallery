## 1. OpenSpec Artifacts

- [x] 1.1 创建 Prompts workspace layout 修复 proposal.
- [x] 1.2 创建 layout design, 明确只修改 desktop CSS / component class.
- [x] 1.3 创建 `prompt-workspace` delta spec, 覆盖 compact desktop 三栏, header 不重叠和 Parameter preset 编辑空间.

## 2. Frontend Implementation

- [x] 2.1 调整 Prompts workspace grid, 在笔记本宽度保持三栏并撑满中间 workspace.
- [x] 2.2 调整 prompt editor header, 防止 name 输入框和保存按钮重叠.
- [x] 2.3 为 Parameter preset JSON 字段添加专用样式并增大默认高度.

## 3. Verification

- [x] 3.1 运行 `openspec validate fix-prompts-workspace-layout --strict`.
- [x] 3.2 运行 `npm run build --prefix apps/desktop`.
- [x] 3.3 运行 `git diff --check`.
