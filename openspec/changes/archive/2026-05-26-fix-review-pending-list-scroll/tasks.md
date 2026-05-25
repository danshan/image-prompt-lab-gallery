## 1. 滚动容器修复

- [x] 1.1 确认 Review Inbox 左侧 panel 的 DOM 分段和现有 CSS 滚动约束.
- [x] 1.2 将 desktop workflow surface 下的 Review list panel 改为显式 grid row layout.
- [x] 1.3 确保 `.review-list` 是最后一行可滚动区域, 且列表项不被拉伸.
- [x] 1.4 确认 narrow viewport fallback 不受影响.

## 2. 验证

- [x] 2.1 运行 desktop frontend tests.
- [x] 2.2 运行 desktop build.
- [x] 2.3 运行 OpenSpec strict validation.
- [x] 2.4 做 Review Inbox pending list 长列表场景的人工或浏览器检查.
