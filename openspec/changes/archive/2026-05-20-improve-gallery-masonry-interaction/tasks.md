## 1. Gallery Layout

- [x] 1.1 将 Gallery board CSS 从等高 grid 调整为固定宽度瀑布流布局.
- [x] 1.2 为 Gallery image preview 增加基于 asset 尺寸的 aspect ratio 计算, 并在尺寸缺失时使用稳定 fallback.
- [x] 1.3 对高于 `2:3` 的图片预览应用高度封顶和 top-aligned crop.

## 2. Gallery Interaction

- [x] 2.1 将 Gallery card 图片区域改为打开原图 lightbox 的独立点击目标.
- [x] 2.2 将 Gallery card 非图片区域改为选择 asset 并刷新 Inspector detail.
- [x] 2.3 确保 Review 操作和批量选择 checkbox 不冒泡触发 lightbox 或 detail selection.

## 3. Verification

- [x] 3.1 运行 OpenSpec validation.
- [x] 3.2 运行 frontend build.
- [x] 3.3 使用浏览器 smoke test 验证瀑布流, `2:3` top crop, image lightbox click, card detail click 和嵌套控件行为.
