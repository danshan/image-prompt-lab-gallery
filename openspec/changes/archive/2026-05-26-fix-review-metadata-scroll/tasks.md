## 1. 滚动行为修复

- [x] 1.1 检查 Review workspace 现有 CSS 层级, 确认右侧 detail panel 在 workflow surface 中缺少 bounded scroll 约束.
- [x] 1.2 更新 desktop workflow surface 下的 Review detail panel CSS, 使其在固定高度 workspace 中 `min-height: 0` 且 `overflow: auto`.
- [x] 1.3 确认 narrow viewport fallback 仍使用 page-level scroll, 不被 desktop-only detail scroll 规则破坏.

## 2. 验证

- [x] 2.1 运行 desktop frontend tests.
- [x] 2.2 运行 desktop build.
- [x] 2.3 对 Review metadata 长内容场景做人工或浏览器 viewport 检查, 确认底部 actions 和 suggestion history 可达且无横向滚动.
- [x] 2.4 运行 OpenSpec strict validation.
