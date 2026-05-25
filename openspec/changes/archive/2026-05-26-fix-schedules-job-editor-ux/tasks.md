## 1. Editor 状态与操作

- [x] 1.1 为 Schedules editor 增加 editing job state, 区分创建模式和编辑模式.
- [x] 1.2 点击 Edit 时记录 editing job id 并加载 job draft.
- [x] 1.3 增加 New schedule 入口, 重置为默认 draft 并退出编辑模式.
- [x] 1.4 调整 editor 标题和底部 action, 创建模式主操作为 Create schedule, 编辑模式主操作为 Update schedule.

## 2. Job Row Layout

- [x] 2.1 将 Scheduled Jobs row 主文本改为使用左对齐的 task row main content 结构.
- [x] 2.2 增加必要的局部 CSS, 保证 name / metadata 与 status pill 保持稳定间距和 ellipsis.
- [x] 2.3 确认 compact desktop 宽度下 row 不出现文本重叠或横向滚动.

## 3. 验证

- [x] 3.1 运行 desktop frontend tests.
- [x] 3.2 运行 desktop build.
- [x] 3.3 运行 OpenSpec strict validation.
- [x] 3.4 做 Schedules workspace 人工或浏览器检查: job row 对齐正常, Edit 后可点击 New schedule 回到创建新 job.
