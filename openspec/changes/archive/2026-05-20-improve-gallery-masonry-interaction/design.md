## Context

Gallery 当前在桌面端以等高 card grid 展示 assets. 这种布局在横图, 方图, 竖图和超高图混合时会丢失图片真实比例感, 且超高图如果直接完整展示会破坏列表可扫读性. 当前 card 主点击区域也同时承担选择详情和图片预览的含义, 用户无法通过 “点击图片” 直接表达查看原图.

本次变更只处理桌面端 Gallery UI. Rust core 已经在 Gallery read model 中提供 `width`, `height` 和 `imagePath` 等字段, 足以让前端决定 preview ratio. 不需要增加 backend command, 数据库字段或 provider 行为.

## Goals / Non-Goals

**Goals:**

- Gallery 使用固定卡片宽度的瀑布流布局.
- 图片预览在尺寸 metadata 可用时保留原始宽高比.
- 超过 `2:3` 的超高图预览封顶到 `2:3`.
- 超高图封顶时使用 top-aligned crop, 优先保留图片顶部.
- 图片区域点击打开原图 lightbox.
- 卡片非图片区域点击选择 asset 并刷新 Inspector detail.
- Review 和 checkbox 等嵌套控件保持独立行为.

**Non-Goals:**

- 不改变 Gallery query semantics.
- 不改变 Rust core read models, Tauri commands 或 SQLite schema.
- 不引入 masonry 或 virtualization 依赖.
- 不新增拖拽排序, 空间键盘导航或新的 selection model.
- 不重做 Gallery 之外的页面视觉语言.

## Decisions

### 使用 CSS column masonry

Gallery board 使用 CSS columns 实现瀑布流, card 使用固定宽度列和 `break-inside: avoid` 保持完整 item. 这个选择的主要原因是当前需求是窄范围 layout adjustment, 不需要引入图片测量循环, resize observer 或第三方 masonry dependency.

替代方案是 CSS grid row span measurement. 它能保留更接近 row-major 的视觉控制, 但需要在 image load 和 resize 后计算 span, 对当前 Gallery 的收益不够. 第三方 masonry library 也可以解决布局, 但会增加依赖面和未来维护成本.

### 前端计算 preview ratio

`Thumbnail` 或等价的 Gallery preview 单元负责根据 asset `width` 和 `height` 计算 CSS aspect ratio:

- 有效尺寸且不高于 `2:3`: 使用原始比例.
- 有效尺寸但高于 `2:3`: 使用 `2:3`.
- 尺寸缺失或无效: 使用稳定 fallback, 例如当前 `4:3`.

超高图使用 `object-fit: cover` 和 `object-position: top center` 的等价 CSS, 保留顶部并从底部裁切. 这符合用户对长图内容优先级的明确要求.

### 拆分 preview click 和 detail click

Gallery card 继续作为完整 selection visual unit. 但交互目标拆分为:

- 图片按钮: 只负责打开原图 lightbox.
- card body 或 article click: 负责选择 asset 和加载 Inspector detail.
- 嵌套 action 控件: `stopPropagation`, 只执行自身操作.

这个模型让用户意图更明确, 也复用现有 `ImageLightbox` 和 selected asset state.

## Risks / Trade-offs

- CSS columns 的视觉顺序可能不同于严格 row-major grid 顺序 -> 当前 Gallery 是 scan-first image board, 本次不引入空间键盘导航或拖拽排序, 因此可接受.
- 超高图会裁掉底部内容 -> 通过点击图片打开原图 lightbox 保留完整查看路径, preview 只负责扫读.
- 尺寸 metadata 缺失会退回固定比例 -> 不伪造尺寸, 保持 UI 稳定, 后续可通过 import/repair 流程提高 metadata 覆盖率.
- card body click 与嵌套控件可能发生冒泡冲突 -> implementation 必须显式阻止 Review 和 checkbox 冒泡, 并用 smoke test 覆盖.

## Migration Plan

这是纯前端行为变更. 发布时不需要数据迁移或资源库升级. 回滚可以恢复 Gallery grid CSS 和 card 主点击结构, 不影响用户数据.

## Open Questions

无. 用户已确认图片点击打开原图, 卡片空白区域展示详情, 并确认超高图超过 `2:3` 后优先保留顶部.
