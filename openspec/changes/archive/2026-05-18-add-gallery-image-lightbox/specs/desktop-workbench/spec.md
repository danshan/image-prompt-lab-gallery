## ADDED Requirements

### Requirement: Inspector 支持 App 内原图预览

桌面应用 SHALL 在 Inspector 中允许用户点击当前 asset 的图片缩略图, 并在 app 内 lightbox 中查看完整原图. Lightbox MUST 使用完整图片比例展示图片, 不得裁剪图片内容.

#### Scenario: 打开 Inspector 原图预览

- **WHEN** 用户选择一个带有图片路径的 asset 并点击 Inspector 图片缩略图
- **THEN** 桌面应用打开 app 内 lightbox, 并完整展示该 asset 图片

#### Scenario: 关闭 Inspector 原图预览

- **WHEN** lightbox 已打开且用户点击关闭按钮, 点击背景, 或按下 `Escape`
- **THEN** 桌面应用关闭 lightbox 并返回当前 Gallery 和 Inspector 上下文

#### Scenario: Gallery Card 点击语义保持不变

- **WHEN** 用户点击 Gallery 中的 asset card
- **THEN** 桌面应用选择该 asset 并在 Inspector 中展示详情, 不直接打开原图预览

### Requirement: Gallery 和 Inspector 缩略图不显示内部正方形描边

桌面应用 SHALL 在 Gallery 图墙和 Inspector 图片缩略图中展示图片内容, 不得在图片内部叠加额外正方形描边.

#### Scenario: 查看 Gallery 图墙图片

- **WHEN** 用户查看 Gallery asset card 图片
- **THEN** 图片内部不显示额外正方形描边

#### Scenario: 查看 Inspector 图片

- **WHEN** 用户查看 Inspector 中的当前 asset 图片
- **THEN** 图片内部不显示额外正方形描边
