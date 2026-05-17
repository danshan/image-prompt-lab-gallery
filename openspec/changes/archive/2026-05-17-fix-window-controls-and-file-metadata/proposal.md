## Why

当前桌面应用使用应用内标题栏后, 窗口拖动, 最小化, 最大化和关闭等基本窗口操作不可用, 这会阻断日常使用和人工验收. 该应用是生产工具, 应优先采用 macOS 和宿主系统的标准窗口行为, 减少自定义 chrome 的跨平台维护成本. 同时, 图片文件元数据的信息层级需要调整: Gallery 左侧不应承载分辨率细节, Inspector 作为详情区应在已有分辨率基础上补充长宽比.

## What Changes

- 恢复系统原生标题栏和窗口控制, 确保拖动, 最小化, 最大化或还原, 关闭使用宿主系统标准行为.
- 移除应用内自定义窗口控制, 避免与系统标题栏重复.
- 从左侧 Gallery card 或 Sidebar asset summary 中移除文件分辨率展示, 避免在浏览列表中暴露过细文件信息.
- 在右侧 Inspector 的 File section 中继续展示分辨率, 并在宽高都可用时展示计算出的长宽比.
- 当宽度或高度缺失时, Inspector 不展示伪造长宽比, 而是展示明确的 unavailable 状态或隐藏该字段.
- 调整右侧 Inspector 信息层级, 将 Rating card 显示在 Prompt card 之下.
- 不改变 Rust core 的资源库写路径, 不改变现有 asset/version 数据模型.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `desktop-workbench`: 修正窗口 chrome 要求为使用系统原生标题栏, 调整 Gallery 与 Inspector 对文件分辨率, 长宽比和 Rating card 位置的展示职责.

## Impact

- `apps/desktop/src-tauri/tauri.conf.json`: 恢复系统窗口 decoration.
- `apps/desktop/src/main.tsx`: 移除应用内自定义标题栏, 调整 Gallery 和 Inspector 文件元数据渲染, 并移动 Inspector Rating card.
- `apps/desktop/src/styles.css`: 移除应用内标题栏和窗口控制样式, 调整 workbench grid.
- `apps/desktop/src/workbench-state.ts` 与 `apps/desktop/tests/workbench-state.test.mjs`: 如长宽比需要状态层辅助计算或格式化, 补充对应测试.
- `openspec/specs/desktop-workbench/spec.md`: 完成变更后需要同步新的窗口控制和文件元数据展示要求.
