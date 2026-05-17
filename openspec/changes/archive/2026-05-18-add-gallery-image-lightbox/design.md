## Context

Gallery 和 Inspector 当前共用 `Thumbnail` 组件展示图片. 该组件用正方形容器和 `object-fit: cover` 生成缩略图, 适合图墙浏览, 但不适合查看完整原图. 右侧 Inspector 目前只展示同样的缩略图, 没有原图预览入口.

缩略图内部的正方形描边来自 `.thumbnail::after`, 因为它是 shared CSS, 所以会同时影响 Gallery card 和 Inspector hero.

本次变化只涉及桌面前端展示层. 图片路径继续复用现有 `convertImagePath()` 逻辑, 不新增 Tauri command, 不修改 Rust core 或 library 持久化结构.

## Goals / Non-Goals

**Goals:**

- 在 Inspector 中提供点击图片查看完整原图的 app 内 lightbox.
- Lightbox 中图片完整可见, 不被裁剪, 并保留原始宽高比.
- 支持关闭按钮, 点击背景和 `Escape` 关闭.
- 移除 Gallery 和 Inspector 缩略图内部正方形描边.
- 保持 Gallery card 点击选择 asset 的既有交互.

**Non-Goals:**

- 不提供 zoom, pan, rotate, download 或 open-in-file-manager 控件.
- 不在 lightbox 内增加 metadata side panel.
- 不改变 Gallery card 的点击语义.
- 不改动后端, 数据库, asset lineage 或资源库文件结构.

## Decisions

### Lightbox 只从 Inspector 触发

Gallery card 主点击已经承担选择 asset 的职责. 如果在 Gallery card 图片上引入预览行为, 会让同一张卡片存在相近但不同的点击语义, 也容易影响批量选择和现有 album card 复用.

因此预览入口放在 Inspector hero 图片上. 用户先选择 asset, 再从 Inspector 查看完整原图, 与当前三栏工作台的信息流一致.

### `Thumbnail` 保持展示组件

`Thumbnail` 继续只负责缩略图视觉展示和图片源渲染. Inspector 侧用 button 包装它来提供点击行为, 而不是把 `onClick` 等交互塞进 `Thumbnail`. 这样 Gallery 和 Inspector 可以继续复用同一个视觉组件, 但交互边界由调用方控制.

### Lightbox 复用现有路径转换

Lightbox 使用 asset 的 `imagePath`, 并通过 `convertImagePath()` 转为可加载 URL. 这样 mock 浏览器和 Tauri desktop runtime 的路径处理保持一致, 避免新增后端接口或平台分支.

### 删除 `.thumbnail::after`

内部正方形描边是一个纯视觉 overlay, 不承载状态或交互含义. 删除 shared pseudo-element 是最小修改, 可以同时修复图墙和 Inspector 的内框问题.

## Risks / Trade-offs

- [Risk] Lightbox 图片加载失败时只会显示浏览器默认破图状态. → Mitigation: 本次保持窄范围修复, 不引入额外错误 UI; 后续如扩展 asset integrity UX, 再统一处理.
- [Risk] `Escape` 监听如果未清理会影响其它快捷键. → Mitigation: 只在 lightbox mount 时注册, unmount 或关闭时清理.
- [Risk] Backdrop 点击关闭可能误触. → Mitigation: 只在 overlay 背景接收点击时关闭, 图片容器内部阻止冒泡.
- [Risk] 移除缩略图内框会降低一点视觉层次. → Mitigation: 外层 card 和缩略图圆角仍提供边界, 内框本身不是必要 affordance.
