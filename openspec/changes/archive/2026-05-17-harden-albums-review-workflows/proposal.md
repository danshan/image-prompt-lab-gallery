## Why

当前 MVP 已具备 albums, metadata review, Gallery 和 Inspector 的核心能力, 但桌面端 Albums 仍是占位式 provider 分组视图, Review Inbox 也缺少可审阅, 可编辑后接受的完整交互. 这会阻碍真实交付验收: 用户可以生成和导入图片, 但不能稳定地完成长期组织和 review-first 元数据确认闭环.

## What Changes

- 将 Albums 视图从占位分组升级为真实 album 工作区, 支持列出 albums, 创建 manual album, 打开 album 并展示 album 内容.
- 在 Inspector 中补齐当前 asset 的 album membership 展示和 `Add to album` 交互, 写入后刷新 Gallery 与 Inspector.
- 将 Review Inbox 升级为真实 pending suggestion 工作区, 支持选择 suggestion, 本地编辑 title, description, tags 和 category 后接受.
- 移除 Review UI 中的 reject 行为, 改为 restore form, 仅恢复本地编辑前状态.
- 支持从 Gallery 对已有 asset 重新发起 review, 将 asset 重新加入 Review Inbox 并显示 Review pending.
- 明确 Generate Image 完成后不强制跳转 Review Inbox, 但生成的 metadata suggestion 必须进入 Review 队列并体现在 badge 和 asset detail pending state 中.
- 保持 Albums 和 Review 为两个独立工作区, 不合并为统一 curation inbox.
- 本次不实现完整 smart album builder, 批量 review, album 删除/重命名/排序或跨视图拖拽.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `albums-search`: 补齐 desktop 可用的 album list, manual album detail, album item count, add asset to album 后按 album query 展示内容的行为要求.
- `metadata-review`: 补齐 Review Inbox 的 suggestion inspect, local edit, restore, regenerate, accept 和 canonical metadata 写入边界要求.
- `desktop-workbench`: 补齐 Albums workspace, Review Inbox workspace, Inspector album membership, review pending state, library switch stale state 清理和刷新行为要求.
- `image-generation`: 明确 generation completion 与 Review Inbox 的关系: 生成结果立即进入 Gallery, metadata suggestion 进入 Review 队列, 不强制导航.

## Impact

- Rust core: 可能需要增加 album list read model, item count 查询, 以及覆盖 album query 与 metadata review 刷新的测试.
- Tauri command: 可能需要增加 `list_albums` command, 并补齐 Albums/Review 相关 DTO mapping.
- Desktop React: 需要替换 Albums 占位视图, 扩展 Review Inbox 交互, 增加 selected album 和 selected suggestion state, 并在写操作后刷新 Gallery 与 Inspector.
- Tests: 需要补充 core tests, desktop state tests, 以及必要的 Tauri mapping 覆盖.
- 不新增外部依赖, 不改变 provider native client 范围.
