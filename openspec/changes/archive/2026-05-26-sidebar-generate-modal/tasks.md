## 1. Shell navigation

- [x] 1.1 在当前 `AppShell` render path 中新增 responsive sidebar 组件, 替换 `WorkspaceSwitcher` 作为一级导航入口.
- [x] 1.2 接入 `sidebarCollapsed` state, 实现宽屏默认展开, compact desktop 默认折叠, 用户 toggle 后本会话优先.
- [x] 1.3 将 Gallery, Albums, Prompts, Schedules, Review 和 Queue 计数迁移到 sidebar count label, 并保留 icon-only 折叠态的 accessible labels.
- [x] 1.4 在 sidebar 中接入全局 theme 和 language actions, 并使用 theme-aware sidebar tokens.

## 2. Generate modal

- [x] 2.1 将 `GenerationComposer` 从 inline workspace section 迁移为轻量 modal content.
- [x] 2.2 在 modal 中支持 provider, prompt 和 reference images compact strip.
- [x] 2.3 限制 reference strip 和 thumbnail 高度, 确保 reference images 不撑高 modal.
- [x] 2.4 保持现有 generation open/submit path, 包括 text generation, Inspector variation 和 reference generation.

## 3. Workspace density

- [x] 3.1 从 workspace 主内容流移除 `StudioOverviewBand`.
- [x] 3.2 调整 command bar / toolbar compact status, 确保 assets/status, Review 和 Queue 仍可见.
- [x] 3.3 清理或覆盖当前 shell/nav/composer 相关 CSS, 避免与旧 `.workbench` 或旧 sidebar selector 互相干扰.

## 4. Verification

- [x] 4.1 更新或新增 desktop frontend tests, 覆盖 sidebar navigation, Generate modal 和 reference strip.
- [x] 4.2 运行 `npm test --prefix apps/desktop`.
- [x] 4.3 运行 `npm run build --prefix apps/desktop`.
- [x] 4.4 运行 `openspec validate sidebar-generate-modal --strict` 和 `git diff --check`.
