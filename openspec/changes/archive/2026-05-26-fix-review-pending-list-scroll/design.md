## Context

Review workspace 的左侧 pending review panel 当前使用 flex column 让 `.review-list` 占据剩余空间并滚动. 但该 panel 还受 workspace grid, sticky positioning, generic `.review-list` grid styles 和 responsive overrides 共同影响. 当列表内容超过可用高度时, 滚动区域不够明确, 容易退化成 panel 内容溢出但不可滚动.

## Goals / Non-Goals

**Goals:**

- Pending Review 面板中只有 suggestion list 区域滚动.
- Header, batch actions 和 add-to-album controls 保持在列表上方.
- Desktop workflow surface 中 list 可以填满剩余高度并滚动.
- Narrow viewport fallback 继续使用 page-level scroll 和 bounded list max-height.

**Non-Goals:**

- 不调整 Review suggestion card 内容或选择逻辑.
- 不改变右侧 Review metadata detail 的滚动策略.
- 不改变 Review data model 或 persistence.

## Decisions

1. 将 desktop workflow surface 下的 `.review-list-panel` 从 flex column 改为显式 grid rows.

   使用 `grid-template-rows: auto auto auto minmax(0, 1fr)` 描述 header, actions, add-to-album controls 和 list 四段结构. 这比依赖 flex remaining space 更贴合现有 DOM, 也更不容易被 `.review-list` 的通用 grid display 干扰.

2. 继续让 `.review-list` 自身承担滚动.

   `.review-list` 保持 `overflow: auto`, `min-height: 0`, 并使用 `align-content: start`, 从而列表项不会被拉伸, 新增项只增加 scroll height.

3. 保持 mobile/narrow viewport overrides.

   现有 `max-width: 959px` fallback 会将 workflow surface 改成 page-level scroll, 并给 list 设置 max-height. 本次修复不改变该策略.

## Risks / Trade-offs

- [Risk] 如果未来左侧 panel 增加新的固定区域, grid row 数需要同步调整. → Mitigation: 当前结构稳定且只服务 Review Inbox, 比隐式 flex 更容易审阅.
- [Risk] Sticky panel 和 internal scroll 同时存在时可能有滚动嵌套感. → Mitigation: desktop surface 本身是固定高度 workspace, 内部 list scroll 是现有 dense console pattern.
