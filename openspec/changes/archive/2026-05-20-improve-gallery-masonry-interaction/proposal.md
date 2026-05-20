## Why

当前 Gallery 使用等高网格展示图片, 对混合横图, 方图, 竖图和超长图的可扫读性不够好. 同时 Gallery card 的点击语义把图片预览和 Inspector 详情选择耦合在同一个主按钮上, 用户无法直接点击图片查看原图, 也难以把 “看原图” 和 “看详情” 明确区分.

## What Changes

- 将 Gallery asset board 调整为固定卡片宽度的瀑布流布局.
- Gallery 图片预览默认保留原始宽高比.
- 对超过 `2:3` 宽高比上限的超高图片, 预览高度封顶到 `2:3`, 并优先保留图片顶部内容.
- 将 Gallery card 点击语义拆分为:
  - 图片区域点击打开原图 lightbox.
  - 卡片非图片区域点击选择 asset 并刷新 Inspector detail.
- 保持 Review 和批量选择 checkbox 的独立操作语义, 不误触详情选择或原图预览.
- 不变更 Gallery query, core read model, 数据库 schema 或资源库文件格式.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `desktop-workbench`: Gallery asset board 的布局规则和卡片点击语义发生变化.

## Impact

- 影响 `apps/desktop` 中 Gallery workspace, thumbnail rendering 和相关 CSS.
- 复用现有 `ImageLightbox` 和 asset selection 状态.
- 不新增外部依赖.
- 不影响 Rust core, Tauri commands, SQLite schema, CLI 或 provider 行为.
