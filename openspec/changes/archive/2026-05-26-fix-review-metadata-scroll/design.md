## Context

Desktop Studio Console 的主 workspace 使用固定高度 grid, 多数 workflow surface 通过 `height: 100%`, `min-height: 0` 和内部 `overflow: auto` 控制滚动. Review workspace 左侧 list panel 已经采用 flex column 和内部 list scroll, 但右侧 Review metadata detail panel 没有同等约束. 当 schema prompt, generated field status 或 suggestion history 较长时, detail 内容会超过 workspace 高度, 外层 `.workflow-surface .review-workspace` 又会隐藏溢出, 导致底部内容不可达.

## Goals / Non-Goals

**Goals:**

- 让 Review metadata detail panel 在 desktop workflow surface 内可垂直滚动.
- 保持左侧 Review Inbox list 的现有滚动行为.
- 保持 Review 表单字段, actions 和 history 的视觉顺序与交互语义.
- 在 compact desktop 宽度下保持内容可达, 不引入横向滚动或文本重叠.

**Non-Goals:**

- 不重做 Review workspace 信息架构.
- 不改变 Review suggestion, metadata generation task 或 canonical metadata 写入逻辑.
- 不改变 Tauri commands, Rust core 或 SQLite schema.

## Decisions

1. 使用 CSS 修复优先, 只在必要时修改 TSX 结构.

   依据: 当前 DOM 已经按 list panel 和 detail panel 分离, bug 来自滚动容器层级缺失. CSS 可以最小化行为风险, 并与 existing workflow surface pattern 保持一致.

2. 将 `.workflow-surface .review-detail-panel` 设为 bounded vertical scroll container.

   依据: detail panel 包含完整 Review metadata flow, 包括 header, confidence, task mirror, form, actions 和 history. 让整个 panel 滚动可以保持 actions 与 history 都可达, 不需要拆分 sticky header 或内部嵌套 scroll.

3. 保留 mobile / narrow viewport 的 visible overflow fallback.

   依据: 小屏 media rules 已经将 workflow surface 切到 page-level scroll. 修复不应破坏该 fallback, 只针对固定高度 desktop surface 生效.

## Risks / Trade-offs

- [Risk] 整个 detail panel 滚动时 header 不固定, 长表单中用户需要向上滚动查看状态. → Mitigation: 本次目标是恢复可达性, 不引入 sticky header 以避免遮挡和 stacking 复杂度.
- [Risk] 如果后续 Review detail 内部新增独立长列表, 单一 panel scroll 可能不够精细. → Mitigation: 当前改动保持最小, 后续可按具体 ownership 拆出局部 scroll region.
