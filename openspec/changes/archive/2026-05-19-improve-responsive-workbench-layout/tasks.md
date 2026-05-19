## 1. Shell Layout

- [x] 1.1 为 Workbench 增加 compact shell state, 覆盖 Sidebar 折叠, Inspector drawer 打开状态和 compact detail 可达性.
- [x] 1.2 调整顶层 Workbench markup 和 CSS, 在 `1280px` 以上保留三栏, 在 `960px` 到 `1279px` 进入 Workspace 优先的折叠 shell.
- [x] 1.3 增加 Sidebar compact rail 表达, 保留 library 入口, Gallery, Albums, Review, Queue 和 Settings 导航可达.
- [x] 1.4 增加 Inspector compact drawer 或等价 detail surface, 保持 Gallery asset 详情在 compact desktop 下可打开和关闭.

## 2. Shared Responsive Patterns

- [x] 2.1 增加 shared responsive CSS tokens, 包括 compact breakpoint, wide breakpoint, sidebar widths, inspector widths, panel minimums 和 card minimums.
- [x] 2.2 增加 toolbar wrapping, split workspace, panel stack 和 table-to-card 的 shared class pattern 或 layout-only component.
- [x] 2.3 为长文本, path, prompt, JSON, log preview, checksum 和 task id 补齐 `min-width: 0`, wrapping, truncation 或 internal scroll 规则.

## 3. Page Layouts

- [x] 3.1 调整 Gallery toolbar, filter strip, card grid 和 selected asset detail 触发行为, 满足 compact desktop layout.
- [x] 3.2 调整 Albums list-detail layout 和 create album UI, 防止 compact desktop 下 popover 覆盖关键操作.
- [x] 3.3 调整 Review list-detail layout, batch controls, add-to-album controls, editable form 和 regenerate field actions.
- [x] 3.4 调整 Queue workspace, 宽屏保留三栏, compact desktop 下提供 `Compose`, `Queue`, `Detail` panel switching.
- [x] 3.5 调整 Settings Libraries 和 Logs, Libraries 在 compact desktop 下使用 row cards, Logs 使用 stacked list/preview 和 bounded preview scroll.

## 4. Verification

- [x] 4.1 运行 `npm test` 验证现有 desktop state tests.
- [x] 4.2 运行 `npm run build` 验证 TypeScript 和 Vite build.
- [x] 4.3 在 `1440px`, `1180px`, `960px`, `900px` 宽度下检查 Gallery, Albums, Review, Queue 和 Settings 没有关键控件覆盖或主操作不可达.
- [x] 4.4 检查长 library path, prompt, schema prompt JSON, checksum, log path 和 task id 不撑破整体布局.

## 5. Studio Workbench Refinement

- [x] 5.1 收敛全局颜色 tokens 为中性专业 light workbench palette, 保留 restrained accent 和明确 status colors.
- [x] 5.2 将 Workspace toolbar 改为 view-aware command surface, 展示当前 view title, status 和主操作层级.
- [x] 5.3 将 Gallery cards 改为 image-first asset tile, 降低 review, tags, version 和 selection 的视觉噪声.
- [x] 5.4 用一致的 inline SVG icon component 替换 `DB`, `#`, `=`, `+`, `X` 等临时文本图标.
- [x] 5.5 调整 Queue 中屏布局, 在 `1280px` 到 `1439px` 优先保留 `Queue | Detail` 并行, compact desktop 再使用本地 panel switching.
- [x] 5.6 运行 `npm test` 和 `npm run build`, 并完成多视口布局检查.
