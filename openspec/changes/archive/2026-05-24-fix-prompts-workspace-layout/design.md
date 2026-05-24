## Context

Prompts workspace 当前由 `PromptWorkspace` 组件和全局 `styles.css` 驱动, 主体包含 Prompt Library, Prompt Editor 和 Run 三个并列区域. 现有 CSS 在 `1599px` 以下提前把 Run 面板折到下一行, 同时 editor header 使用两列 grid, 在笔记本可用宽度不足时会压缩 name 输入框并导致保存按钮视觉重叠.

本次变更是 desktop UI 布局修复, 不改变 prompt document, version snapshot, render 或 generation run 的业务流程.

## Goals / Non-Goals

**Goals:**

- 让 Prompts workspace 在 compact desktop / 笔记本宽度下维持三栏并填满中间 workspace.
- 让 prompt draft name 输入框和保存按钮在窄宽度下自然换行, 不发生重叠.
- 增大 Parameter preset JSON 编辑区默认高度, 降低编辑 structured preset 的摩擦.
- 保持现有 component ownership, 不引入新的 state 或 layout framework.

**Non-Goals:**

- 不重做 Prompts workflow 信息架构.
- 不改变 Prompt Library, draft 保存, version 保存或 prompt run 行为.
- 不修改 resource library schema, daemon API, Tauri commands 或 provider contract.

## Decisions

- 使用 CSS grid 的 compact 三栏比例替代 `1599px` 以下的两栏降级. 这样可以让 Prompts workspace 继续表达 Library / Editor / Run 三个并列工作区, 并避免 Run 面板被推到下一行造成空间浪费.
- 降低三栏的 minimum track width, 用 editor 占优的 `fr` 比例分配剩余空间. 这比固定像素宽度更适合 Studio shell 中间 workspace, 因为可用宽度会受到 navigator, inspector 和 activity rail 影响.
- 将 prompt editor header 从两列 grid 改为可换行 flex. Name 字段保留可增长空间, action buttons 作为右侧 action group; 当宽度不足时 action group 可以换到下一行, 而不是覆盖输入框.
- 为 Parameter preset JSON 字段增加专用 class, 只提升该字段 textarea 的 `min-height`. 这避免影响 Variables schema, Default values 或 Run values 等其他 JSON 字段的密度.

## Risks / Trade-offs

- [Risk] 三栏在极窄 desktop 宽度下每栏内容会更紧凑. → Mitigation: 继续保留 `1279px` 以下的单栏断点, 避免在真正小屏上强制三栏.
- [Risk] Header 换行会增加 editor 顶部高度. → Mitigation: 只在空间不足时换行, 正常宽度仍保持一行布局.
- [Risk] 只靠 CSS build 难以捕捉视觉回归. → Mitigation: 至少运行 frontend build, 并在浏览器中对 Prompts workspace 做笔记本宽度人工检查.
