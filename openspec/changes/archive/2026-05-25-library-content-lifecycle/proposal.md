## Why

当前 Gallery, Albums, Prompt Library 和 Settings 已经具备基础内容管理能力, 但缺少统一的内容生命周期: 用户不能从 Gallery 归档图片, 不能从 Prompt Library 归档 prompt, 归档内容没有集中恢复或物理删除入口, Albums 图墙无法打开全图, Gallery 图墙也没有按图片比例展示的瀑布流体验.

同时, 现有 `Import Zip` 语义是把备份恢复成独立 library, 不能满足用户把另一个 managed library 的创作内容合并进当前 library 的需求. 该需求涉及 manifest, schema, managed file layout, ID 冲突和 lineage 引用重写, 必须通过 OpenSpec 明确定义.

## What Changes

- Gallery 支持对单张或多选 asset 执行 archive, 从默认 Gallery / Albums / add-to-album / Review 入口隐藏, 但保留 managed files 和 lineage.
- Prompt Library 支持 archive prompt document, 从默认 prompt list 隐藏, 但保留 prompt versions 和 output history.
- Settings 增加 Archived Content 管理区, 展示 archived assets 和 archived prompts, 支持 restore 和经过 dry-run + confirmation 的 permanent delete.
- Permanent delete 只允许 archived item, 并按用户确认采用级联删除语义, 删除相关 SQLite facts 和 managed files.
- Gallery 图墙改为 masonry / 瀑布流展示, card 宽度稳定, 图片按真实宽高比展示并限制超长图最大高度.
- Albums 图墙 thumbnail 支持打开全图 lightbox, manual album remove 仍只移除 album membership.
- Settings / Libraries 增加 `Merge Library` workflow, 将另一个 source library 的创作内容 copy-merge 到当前 target library, source library 保持不变.
- Merge import 只迁移创作内容: assets, asset versions, tags, albums, prompt documents, prompt versions, generation events, metadata suggestions 和相关 lineage.
- Merge import 不迁移 runtime state: queue tasks, schedules, logs, diagnostics, registry aliases 和 manifest identity.
- Merge import 对所有 source entities 生成 target 新 ID, 复制 managed files 到 target 标准路径, 并按 ID map 重写内部引用.
- Existing `Import Zip` restore-as-separate-library / clone-on-conflict 语义保持不变.
- 不引入 mock 或 fake 逻辑作为真实功能替代; desktop actions 必须接入 core / Tauri commands.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `resource-library`: 增加 archived asset 持久化语义, Archived Content restore / permanent delete, 以及 library copy-merge import 的兼容检查, dry-run 和 apply 语义.
- `desktop-workbench`: 增加 Gallery archive action, masonry 图墙, Settings Archived Content UI, Settings Merge Library UI, 以及 destructive action 的确认路径.
- `albums-search`: 调整 Gallery / Albums 默认查询语义以排除 archived assets, 并要求 Albums 图墙 thumbnail 可打开 full image preview.
- `prompt-workspace`: 增加 prompt document archive / restore / permanent delete 生命周期, 并确保默认 Prompt Library list 排除 archived prompts.

## Impact

- Rust core:
  - schema migration for asset archive metadata.
  - library content lifecycle use cases and repository operations.
  - permanent delete dependency planning and apply.
  - library merge dry-run and copy-merge apply.
- SQLite:
  - asset archive metadata.
  - archived content read models.
  - cascade delete plans for archived assets and prompts.
  - source-to-target ID remapping during merge import.
- Tauri:
  - new commands for archive, restore, archived content listing, permanent delete dry-run/apply, merge dry-run/apply.
  - view mappers for archived content and merge summaries.
- Desktop frontend:
  - Gallery masonry and archive actions.
  - Albums thumbnail lightbox integration.
  - Prompt Library archive actions.
  - Settings Archived Content and Merge Library sections.
- Tests:
  - core lifecycle and merge tests.
  - desktop command mapping tests.
  - frontend interaction tests.
  - visual verification for masonry and Settings flows.
