## Why

当前资源库文件布局, Library 切换入口和桌面工作台视觉细节已经影响 MVP 的长期可维护性和日常使用体验. 图片文件需要避免单目录堆积和文件名冲突, 同时桌面端需要去掉重复标题栏, 明确支持多个 Library 切换, 并修正 Sidebar 与 Inspector 中若干不准确或不一致的展示.

## What Changes

- 资源库导入和生成图片写入 managed files 时, 文件名使用 UUID, 并按 `$year/$month/$filename` 拆分相对路径, 避免单目录存储和命名冲突.
- asset version checksum 统一使用 MD5, file context 展示格式为 `Checksum    MD5: $hash`.
- Library Status 中 storage 只展示当前 Library 实际大小, 不展示固定容量上限或进度上限.
- 桌面端支持多个已注册 Library, 并在工作台最上方提供 Library 切换入口.
- 桌面 Tauri window 去掉系统默认标题栏, 保留应用内自定义标题栏, 避免最大化和关闭等窗口控制重复出现.
- Sidebar 不出现内部滚动条, 应通过响应式高度, 密度和内容折叠适配窗口大小.
- 所有 select / selector 控件使用与 app 一致的视觉样式, 不再混用浏览器默认样式.
- Sidebar 中分辨率字段必须来自真实 asset detail 或 card read model, 缺失时展示 unavailable 状态, 不展示错误或占位尺寸.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `resource-library`: 调整 managed image file layout, filename uniqueness, checksum algorithm 和 storage usage read model.
- `desktop-workbench`: 调整 desktop shell chrome, Library switcher, Sidebar layout, selector styling 和 file metadata presentation.

## Impact

- Rust core resource library import, generation asset persistence, integrity check, file context read model 和相关测试.
- SQLite schema 或 migration 可能需要新增 checksum algorithm 字段, 或以兼容方式明确当前 checksum 含义.
- Tauri command DTO 和 frontend type mapping 需要暴露 Library 列表, 当前 Library 切换状态, storage size, dimensions 和 checksum label.
- Tauri window config 需要设置无系统装饰窗口, 并确认 macOS / Windows / Linux 下 resize, drag region 和 window controls 行为可用.
- React workbench layout, Library Sidebar, top app bar, selector component styling 和 mock data 需要同步更新.
