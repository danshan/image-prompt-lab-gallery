## Why

当前桌面端的 tags 入口不可用, 搜索无法可靠覆盖 title, prompt 和 tags, 且图墙和 Inspector 在真实生成数据上经常显示 `Untitled`, `Unknown provider` 或缺失 prompt. 这些问题会直接破坏 MVP 的核心浏览, 筛选和复用 prompt 工作流, 需要在实现层补齐数据写入链路和 read model 映射.

## What Changes

- 让桌面端支持给 asset 添加手动 tag, 成功后刷新图墙和 Inspector.
- 修复 gallery search, 使搜索文本覆盖 asset title, prompt, provider/model 和 tags, 并保持 tag filter 与其他 filter 的 AND 语义.
- 修复生成图片的 metadata 关联, 确保通过 generation flow 创建的新 asset/version 能关联 generation event, 从而让 provider, model, prompt 和 parameters 可被图墙与 Inspector 读取.
- 为生成资产创建基于 prompt 的默认 title, 并允许用户在 Inspector 中双击 title 后编辑.
- 将 rating 的显示和输入改为星级控件, 不再使用文本 `*` 占位.
- 明确 imported-only asset 没有 generation metadata 时仍可显示为 `Untitled` / `Unknown provider`, 但生成资产不得因关联缺失而退化为这些占位文案.
- 补齐 core, Tauri command 和 desktop state/UI 的回归测试.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `albums-search`: 明确 gallery 文本搜索必须覆盖 title, prompt 和 tags, 并要求生成资产的 gallery read model 返回 provider/model.
- `desktop-workbench`: 明确 Inspector 可添加 tag, 并要求图墙和详情页展示生成资产的 provider, model, prompt 和 tags.
- `image-generation`: 明确 generation flow 写入 library 时必须把 generation event 绑定到生成 asset/version.

## Impact

- Rust core: `LocalGenerationService`, `GalleryReadService`, search helpers, tag 写入和相关 tests.
- Tauri commands: gallery/detail/tag command 映射, 错误处理和返回字段.
- Desktop React: search 输入, tag 添加 UI, gallery/detail 刷新与状态同步.
- SQLite schema 不需要升级, 但需要修复现有写入流程中 event 与 generated asset/version 的关联方式.
