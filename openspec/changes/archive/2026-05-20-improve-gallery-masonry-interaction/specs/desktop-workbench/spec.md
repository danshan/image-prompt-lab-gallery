## ADDED Requirements

### Requirement: Gallery Asset Board 使用固定宽度瀑布流和明确点击语义

桌面应用 SHALL 在 Gallery asset board 中使用固定卡片宽度的瀑布流布局展示 asset cards. Gallery 图片预览在可用尺寸 metadata 下 MUST 保留原始宽高比, 但当图片高于 `2:3` 宽高比上限时, 预览 MUST 封顶为 `2:3` 并优先保留图片顶部内容. Gallery card MUST 将原图预览和详情选择拆分为不同点击目标: 图片区域打开原图 lightbox, 卡片非图片区域选择 asset 并更新 Inspector detail. 嵌套操作控件 MUST 保持自身语义, 不得误触原图预览或详情选择.

#### Scenario: 混合比例图片以瀑布流展示

- **WHEN** 用户打开 Gallery 且结果包含横图, 方图和竖图 asset
- **THEN** Workspace 以固定卡片宽度瀑布流展示这些 assets, 并在可用尺寸 metadata 下按各自原始宽高比展示图片预览

#### Scenario: 超高图预览封顶并保留顶部

- **WHEN** Gallery asset 的图片比例高于 `2:3`
- **THEN** 图片预览高度封顶到 `2:3`, 并以顶部对齐方式裁切, 优先保留源图顶部内容

#### Scenario: 图片点击打开原图

- **WHEN** 用户点击 Gallery card 的图片区域
- **THEN** 桌面应用打开该 asset 当前图片的原图 lightbox, 且不因该点击改变 Inspector detail selection

#### Scenario: 卡片非图片区域展示详情

- **WHEN** 用户点击 Gallery card 的 title, metadata, footer 或空白区域
- **THEN** 桌面应用选择该 asset, 并在 Inspector 中展示该 asset detail

#### Scenario: 嵌套控件不误触卡片点击

- **WHEN** 用户点击 Gallery card 内的 Review 操作或批量选择 checkbox
- **THEN** 桌面应用只执行对应控件操作, 不同时打开原图 lightbox 或触发 card detail selection
