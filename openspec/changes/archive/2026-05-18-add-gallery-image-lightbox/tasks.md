## 1. Lightbox 状态与交互

- [x] 1.1 在 desktop `App` 中添加 lightbox 状态, 并将打开/关闭回调传给 `Inspector`.
- [x] 1.2 新增 `ImageLightbox` React 组件, 使用现有 `convertImagePath()` 渲染完整图片.
- [x] 1.3 为 lightbox 实现关闭按钮, 背景点击关闭和 `Escape` 关闭, 并确保事件监听正确清理.

## 2. Inspector 入口与缩略图样式

- [x] 2.1 将 Inspector hero 图片包装为仅在存在 `imagePath` 时可点击的预览按钮.
- [x] 2.2 保持 Gallery card 点击行为不变, Gallery card 点击仍只选择 asset.
- [x] 2.3 删除 shared thumbnail 内部正方形描边样式, 确保 Gallery 和 Inspector 都不再显示内框.

## 3. 验证

- [x] 3.1 运行 desktop 前端检查或项目现有相关测试.
- [x] 3.2 手动验证 Inspector 图片打开 lightbox, 完整展示原图, 且三种关闭方式有效.
- [x] 3.3 手动验证 Gallery 图墙和 Inspector 图片均无内部正方形描边.
