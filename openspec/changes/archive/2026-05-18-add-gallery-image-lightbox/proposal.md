## Why

Gallery 右侧 Inspector 当前无法点击图片查看完整原图, 用户只能看到被缩略图容器裁剪后的预览. 同时 Gallery 图墙和 Inspector 缩略图内部存在一个额外正方形描边, 干扰图片内容查看.

## What Changes

- Inspector 中有图片的 asset SHALL 支持点击缩略图打开 app 内 lightbox.
- Lightbox SHALL 以完整比例展示原图, 不裁剪图片内容.
- Lightbox SHALL 支持关闭按钮, 点击背景和 `Escape` 关闭.
- Gallery card 点击语义保持不变, 仍用于选择 asset.
- Gallery 图墙和 Inspector 图片缩略图 SHALL 移除内部正方形描边.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: Gallery/Inspector 图片展示和 Inspector 原图预览行为发生变化.

## Impact

- 影响 `apps/desktop/src/main.tsx` 的 Inspector 图片交互和 lightbox 渲染.
- 影响 `apps/desktop/src/styles.css` 的缩略图和 lightbox 样式.
- 不涉及 Rust core, Tauri command, SQLite schema, resource library 文件结构或公共 API 变更.
