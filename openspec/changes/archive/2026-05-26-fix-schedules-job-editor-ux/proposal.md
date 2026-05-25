## Why

Schedules workflow 中的 Scheduled Jobs list row 没有稳定使用左对齐的主内容列, job name 和右侧状态之间缺少清晰空隙. 同时用户点击现有 job 的 Edit 后, draft 被切换到编辑内容, 但没有明确入口重置为创建新 job, 容易让创建和更新路径混淆.

## What Changes

- 修复 Scheduled Jobs row layout, 让 job name / metadata 左对齐, 并与右侧 status pill 保持稳定间距.
- 在 schedule editor 中增加明确的 New schedule 入口, 可从编辑现有 job 的 draft 回到创建新 job 的默认 draft.
- 让 editor 标题和 primary action 区分 create / edit 状态, 降低误点 update 或 create copy 的风险.
- 保持现有 create, update, duplicate, delete, run now 和 enable / pause backend 行为不变.
- 不改变 scheduled generation persistence, daemon API 或 schedule runner 行为.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `scheduled-image-generation`: 加固 Schedules workflow 管理 job 的 UX 要求, 明确 job list row 需要可扫描的左对齐布局, 且从编辑现有 job 返回创建新 job 必须有明确入口.

## Impact

- 影响 `apps/desktop/src/app/screens/workflows/schedules.tsx` 的 editor 状态和按钮呈现.
- 影响 `apps/desktop/src/styles.css` 中 scheduled job row 的局部布局样式.
- 验证范围为 desktop frontend tests, build, 以及 960px 级别 viewport 下的人工或浏览器检查.
