## 1. Spec Artifacts

- [x] 1.1 创建 `redesign-desktop-interface-system` change.
- [x] 1.2 编写 proposal, design 和 `desktop-workbench` delta spec.
- [x] 1.3 运行 `openspec validate redesign-desktop-interface-system --strict`.
- [x] 1.4 运行 `openspec validate --specs --strict`.

## 2. Shell and Design System

- [x] 2.1 新增或替换 desktop shell 为 Command Bar, Workspace Switcher, Workflow Surface 和 Context Drawer.
- [x] 2.2 移除旧 sidebar / library context / 常驻 inspector 布局对新界面的结构约束.
- [x] 2.3 引入现代 app-native visual language, 屏弃传统 website / web admin dashboard 风格.
- [x] 2.4 引入 semantic theme tokens, 覆盖 light/dark surfaces, text, border, focus, actions, status 和 overlay.
- [x] 2.5 审查 surface, card, button 和 decoration 使用, 移除 hero, marketing-style section, oversized cards, decorative gradients/blobs 和过度复杂按钮.
- [x] 2.6 实现 theme toggle 和持久化 preference, 不影响 resource library data.
- [x] 2.7 建立统一 z-index scale 和 drawer overlay/dock 行为.

## 3. Locale Boundary

- [x] 3.1 新增 locale dictionary boundary, 初始支持 `en` 和 `zh-CN`.
- [x] 3.2 实现 language switch 和 locale preference.
- [x] 3.3 将主要 shell, navigation, workflow labels, actions, empty/error states 迁移到 translation keys.
- [x] 3.4 增加 formatter helpers, 覆盖 count, date, bytes, status 和 action labels.

## 4. Workflow Redesign

- [x] 4.1 Gallery 改为 image-first board + compact filters + selection action bar + asset drawer.
- [x] 4.2 Albums 改为 album manager + selected album surface + add-to-album drawer, 保持 Albums workflow ownership.
- [x] 4.3 Prompts 改为 prompt library / editor / run preview 的 responsive split layout.
- [x] 4.4 Review 改为 inbox / metadata draft / history-task context drawer 的 staged workbench.
- [x] 4.5 Queue 改为 compose / queue / detail 的 operations console, compact 下 panel switching.
- [x] 4.6 Settings 改为 Libraries, Providers, Updates, Logs page-local sections, 使用紧凑表格和行内 actions.
- [x] 4.7 保留现有业务能力和 controller wiring, 不改变 Rust core 业务语义.

## 5. Responsive and Accessibility

- [x] 5.1 定义 `wide`, `compact`, `narrow` responsive layout policies.
- [x] 5.2 确保 `>=1280px`, `960px - 1279px`, `<960px` 下主要导航, 主操作, detail 和 recovery action 可达.
- [x] 5.3 为 long path, prompt, JSON, checksum, log, task id 增加 wrap, truncation 或 bounded scroll 策略.
- [x] 5.4 确保 icon-only controls 有 accessible labels 和 visible focus.
- [x] 5.5 覆盖 drawer keyboard close, backdrop close 和 focus recovery.

## 6. Tests and Verification

- [x] 6.1 增加 frontend tests 覆盖 theme, locale, drawer state 和 responsive shell state.
- [x] 6.2 更新 workflow state tests, 覆盖 Gallery, Albums, Prompts, Review, Queue, Settings 的关键交互入口.
- [x] 6.3 运行 `npm test --prefix apps/desktop`.
- [x] 6.4 运行 `npm run build --prefix apps/desktop`.
- [x] 6.5 启动 desktop dev server, 用 browser QA 检查 `1440px`, `1280px`, `960px`, `768px`, `390px`.
- [x] 6.6 检查 Gallery, Albums, Prompts, Review, Queue, Settings 和 drawer states 无组件重叠, 无关键操作不可达, 无页面横向溢出.
- [x] 6.7 检查视觉结果是否保持 app-native, 有设计感但不复杂臃肿, 且没有退回传统 website / admin template.
- [x] 6.8 运行 `git diff --check`.

## 7. Canvas Deck Demo Alignment

- [x] 7.1 将 production shell 对齐 Canvas Deck demo: 顶部 Command Bar, 水平 Workspace Switcher tabs, Workflow Surface 和 Context Drawer.
- [x] 7.2 将 Command Bar 操作收敛为 compact icon / indicator controls, 确保 stats, badges 和 locale 切换在小屏不裁切.
- [x] 7.3 移除或覆盖旧 sidebar / inspector / decorative gradient 视觉遗留, 保证 production CSS 不再退回旧页面结构.
- [x] 7.4 对 Gallery, Albums, Prompts, Review, Queue, Settings 运行生产应用视觉检查, 覆盖 `1440px`, `1280px`, `960px`, `768px`, `390px`.
- [x] 7.5 重新运行 `openspec validate redesign-desktop-interface-system --strict`, `npm test --prefix apps/desktop`, `npm run build --prefix apps/desktop` 和 `git diff --check`.
