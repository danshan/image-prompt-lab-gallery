## Why

Review Inbox 的 Review metadata 右侧详情区域包含表单, 重新生成状态, actions 和 suggestion history. 当前 detail panel 在 Studio Console 的固定高度 workspace 内不能滚动, 用户在内容超过视口时无法访问底部字段和操作.

## What Changes

- 修复 Review metadata detail panel 的滚动行为, 让右侧详情内容在 workspace 高度受限时可垂直滚动.
- 保持左侧 Review Inbox list 现有内部滚动行为不变.
- 保持 desktop compact 最小宽度目标和现有 Review 表单布局不变, 只调整滚动容器和高度约束.
- 增加针对 Review workspace 的前端回归测试或等价验证, 覆盖长 schema prompt / history 场景下底部操作可达.
- 不改变 Review metadata 的业务状态, API, persistence 或 canonical metadata 写入语义.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `metadata-review`: 加固 Review workspace 可达性要求, 明确 Review metadata detail 内容在桌面固定高度 workspace 中必须可滚动并保持底部操作可达.

## Impact

- 影响 `apps/desktop/src/styles.css` 中 Review workspace / panel / form 的滚动与高度约束.
- 可能影响 `apps/desktop/src/app/screens/workflows/review.tsx` 的 detail 内容包裹结构, 仅在 CSS 难以准确限定滚动区域时修改.
- 验证范围为 desktop frontend tests 和必要的 build / manual viewport check.
