## 1. Core Read Models

- [x] 1.1 增加 Gallery query, sort, card view 和 asset detail view DTO.
- [x] 1.2 实现 core Gallery query service, 支持 text, provider, rating, review status, tags, album 和 sort.
- [x] 1.3 实现 asset detail 聚合, 返回 prompt, provider/model, tags, albums, versions, lineage 和 file context.
- [x] 1.4 增加 provider capability 检查, 对不支持的 image-to-image 请求返回 `UnsupportedProviderCapability`.
- [x] 1.5 为 Gallery query, asset detail 和 provider capability 添加 Rust 测试.

## 2. Tauri Commands

- [x] 2.1 增加 `query_gallery` command 和 DTO mapping.
- [x] 2.2 增加 `get_asset_detail` command 和 DTO mapping.
- [x] 2.3 扩展 `start_generation` 输入处理, 支持 `input_version_id` 和 recoverable capability error.
- [x] 2.4 为 command mapping 和错误映射添加测试或可验证覆盖.

## 3. Frontend State and Commands

- [x] 3.1 拆分 Gallery query state, selected asset state, detail loading state 和 recoverable error state.
- [x] 3.2 接入 `query_gallery` 和 `get_asset_detail`, 替换 Gallery 主流程中的前端拼装逻辑.
- [x] 3.3 接入 variation action, 在 provider 不支持时展示 inline recoverable error.
- [x] 3.4 更新或新增 frontend state 测试.

## 4. Productized Workbench UI

- [x] 4.1 重构 sidebar, library switcher, navigation badges 和 library status panel.
- [x] 4.2 重构 workspace search bar, Generate split button, filters, sort 和 item count.
- [x] 4.3 重构 Gallery cards, 包含 thumbnail, provider/model, rating, review badge, tags 和 version 摘要.
- [x] 4.4 重构 Inspector sections, 包含 header preview, prompt, provider/model, tags, albums, lineage 和 file.
- [x] 4.5 实现 desktop, medium 和 narrow viewport 的 responsive behavior.

## 5. Verification and Commits

- [x] 5.1 运行 Rust 测试并修复失败.
- [x] 5.2 运行 desktop/frontend 测试并修复失败.
- [x] 5.3 启动本地桌面前端并用浏览器检查宽屏和窄屏布局.
- [x] 5.4 按 spec, core, Tauri, frontend UI, verification fixes 的稳定边界提交代码.
