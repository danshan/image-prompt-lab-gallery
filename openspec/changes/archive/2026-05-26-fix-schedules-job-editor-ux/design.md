## Context

Schedules workspace 当前由三列组成: job list, create/edit form, run detail. Job list row 使用 `.task-row` 的两列 grid, 但 row 内主文本是裸 `span`, 没有 `.task-row-main` 的 `text-align: left`, `min-width: 0` 和 ellipsis 约束, 因而在 button 默认居中或内容较长时会破坏扫描性. Editor 当前只有一个 draft state, `Edit` 会把 selected job 复制进 draft, 但 UI 仍显示 Create Schedule, 并同时提供 Create / Update 两个按钮, 缺少回到空白新建 draft 的显式动作.

## Goals / Non-Goals

**Goals:**

- Scheduled Jobs row 的名称和 metadata 左对齐, status pill 右对齐, 中间有稳定间距.
- Editor 明确区分创建模式和编辑模式.
- 用户点击 Edit 后, 可以通过 New schedule 回到默认新建 draft.
- 保持现有表单字段和业务调用不变.

**Non-Goals:**

- 不重构 Schedules workflow 的三列布局.
- 不改变 schedule validation, daemon request 或 persistence.
- 不新增 modal, wizard 或全新 navigation pattern.

## Decisions

1. 在 Schedules workspace 内引入局部 editor mode state.

   选择 `editingJobId: string | null` 表示当前 draft 是否来自某个 job. 这样可以明确禁用或隐藏不合适的 action, 并让 New schedule reset draft. 替代方案是根据 draft 与 selected job 是否相等推断模式, 但会因为用户编辑字段后难以判断意图, 可维护性较差.

2. 使用现有 `.task-row-main` 结构修复 job row alignment.

   Scheduled job row 应与 Queue task row 一样使用主内容 column, 避免为同类 row 创建独立视觉系统. 必要时增加 `.schedule-job-row` 的局部 class 来补充 gap, text alignment 和 status pill placement.

3. 保留 Create 和 Update 能力, 但 action 呈现随模式变化.

   创建模式显示主要 Create schedule. 编辑模式显示主要 Update schedule, 并显示 New schedule 作为回到创建的入口. 这比同时常驻两个 full-width action 更清晰.

## Risks / Trade-offs

- [Risk] 用户可能习惯在编辑 draft 后点击 Create 来复制 job. → Mitigation: Duplicate action 已存在且语义更明确, 本次将 create/update 模式分开以降低误操作.
- [Risk] 默认 album 可能在 albums 异步刷新后变化. → Mitigation: reset 使用当前 `manualAlbums[0]?.id` 和 `defaultProvider`, 与初始 draft 逻辑保持一致.
