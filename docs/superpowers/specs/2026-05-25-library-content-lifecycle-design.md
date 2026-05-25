# Library Content Lifecycle Design

## 目标

本设计定义一个统一的 library content lifecycle change, 覆盖 Gallery 图片归档, Prompt Library prompt 归档, Settings 中归档内容管理, 归档内容恢复与物理删除, Gallery / Albums 图墙体验修复, 以及把另一个 resource library copy-merge 到当前 library.

该 change 的核心原则是: 高频删除只进入 archived 状态, 真正不可逆的物理删除只能从 Settings 的归档内容管理区发起; library merge import 必须保持 source library 不变, 并在 target library 中生成新的实体 ID 和 managed file paths.

## 已确认决策

- Gallery asset 删除默认是 soft delete / archive, 不直接删除 managed files 或 lineage.
- Prompt Library prompt 删除默认是 archive, 使用 prompt document 的 archived 语义.
- Settings 增加归档内容列表, 支持 restore 和 permanent delete.
- Permanent delete 采用级联删除语义, 但只能对 archived item 执行, 且必须先 dry-run 展示影响范围.
- Library merge import 采用 copy-merge into current library, source library unchanged.
- Merge 内容范围为创作内容: assets, asset versions, tags, albums, prompt documents, prompt versions, generation events, metadata suggestions 和相关 lineage.
- Merge 不迁移 runtime state: queue tasks, schedules, logs, diagnostics, registry aliases 和 manifest identity.
- Merge 冲突策略是全部复制为新实体, 保留 source-to-target ID map, 不按名称智能合并.
- Gallery 仍是 all-assets browser, Albums 仍拥有 add-to-album workflow ownership.

## 行为契约

### Gallery asset archive

Gallery card 和多选 action bar 提供 archive 操作. Archive 后 asset 从 Gallery 默认查询, Albums detail, add-to-album drawer 和常规 Review 入口中隐藏, 但保留 managed files, asset versions, generation events, album memberships, tags, metadata suggestions 和 prompt lineage.

Archive 成功后, frontend 必须刷新 Gallery query, 清空已归档 selection. 如果当前 Inspector 指向 archived asset, Inspector 应关闭或切换到下一个可见 asset.

### Prompt document archive

Prompt Library row 提供 archive 操作. Archive 后 prompt document 从默认 Prompt Library list 中隐藏, 但保留 prompt versions 和 prompt-to-output history.

当前选中 prompt 被 archive 后, UI 应选择下一条 active prompt. 如果没有 active prompt, Prompt Workspace 进入 empty draft state.

### Settings Archived Content

Settings 增加 `Archived` section, 与 `Libraries`, `Providers`, `Updates`, `Logs` 同级. 该 section 展示 archived assets 和 archived prompts, 推荐使用 `Assets` / `Prompts` segmented control.

每行展示 item title/name, content type, archived time, dependency summary, estimated storage impact 和 source marker. 每行支持 `Restore` 和 `Delete permanently`.

`Restore` 将 item 恢复到 active list. 对 asset, Gallery 和 Albums read models 必须重新可见. 对 prompt, Prompt Library 默认 list 必须重新可见.

`Delete permanently` 必须先执行 dry-run, 展示将删除的 SQLite rows, managed files, album memberships, tags, metadata suggestions, asset versions, generation events, prompt versions 和 output history references. 用户确认后才执行 apply.

### Permanent delete

Permanent delete 只允许 archived item. Active item 必须返回 domain error. Delete planner 必须被 dry-run 和 apply 共享, 避免预览和实际删除影响范围不一致.

Apply 阶段应先在 SQLite transaction 中删除数据库事实. Transaction 成功后再删除 managed files. 如果 file deletion 失败, 系统必须返回可修复状态或 orphan cleanup issue, 不得报告完全成功.

Gallery 和 Prompt Library 不提供 permanent delete 入口. Permanent delete 只位于 Settings Archived Content.

### Gallery masonry

Gallery 图墙应使用 masonry / column layout. Card width 稳定, 图片按真实宽高比展示, 宽度填满卡片. 长图必须限制最大显示高度, 避免单张图撑爆页面. 图片可使用 contained display 或受控裁切, 但不能横向溢出.

Albums detail 图墙复用同一 thumbnail preview 行为. 点击 thumbnail 打开 full image lightbox, card click 仍保留 asset selection / context 行为. Manual album 的 remove action 只删除 album membership, 不 archive asset.

### Library merge import

Settings / Libraries 新增 `Merge Library` action, 与现有 `Import Zip` 分开. `Import Zip` 保持 restore-as-separate-library / clone-on-conflict 语义, 不被 merge import 改写.

Merge target 是当前 library. 用户选择 source library folder. 系统先执行 dry-run:

- 校验 source layout.
- 校验 source manifest 和 schema version.
- 如果 source schema 高于 current schema, 拒绝.
- 如果 source schema 可由当前应用迁移, 在 staging 或只读安全路径完成兼容检查.
- 统计将复制的 assets, versions, tags, albums, prompts, prompt versions, generation events, metadata suggestions, file count 和 file size.
- 报告将跳过的 runtime state: tasks, schedules, logs 和 diagnostics.
- 报告将自动重命名的 album / prompt display names.

Apply 阶段 source library 不变. Target 中所有 source entities 生成新 ID, managed files 复制到 target 标准路径. Source relative paths 不复用. Generation event, prompt version link, asset version parent, album membership, tags 和 metadata suggestions 的内部引用按 ID map 重写.

同名 album / prompt 不智能合并. UI 可在 display name 上追加来源后缀, 例如 source library name 或 short source id.

## 架构边界

### Core application

新增或扩展 core application use case, 负责 library content lifecycle:

- `archive_asset`
- `restore_asset`
- `archive_prompt_document`
- `restore_prompt_document`
- `list_archived_content`
- `dry_run_permanent_delete`
- `permanent_delete_archived_item`
- `dry_run_merge_library`
- `merge_library`

Runtime adapters 不得直接拼 SQL 或复制 files.

Asset archive 推荐新增明确持久化字段, 例如 `assets.archived_at TEXT`, 让 `status` 继续表达 generated, imported, reference 等业务状态. Gallery query 和 album-scoped query 默认过滤 `archived_at IS NULL`.

Prompt archive 使用现有 prompt document status / archived time 方向, 并补齐 use case, repository 和 command.

### SQLite adapter

SQLite adapter 负责 schema migration, archive status updates, archived content read models, permanent delete dependency planning, merge import ID mapping 和 transaction.

Permanent delete 的 dependency planner 必须可复用, dry-run 不写入, apply 使用同一 planner 输出作为删除计划.

Merge apply 必须以 target database transaction 写入关系事实. Managed files 复制和 database transaction 的边界需要显式处理失败恢复: 数据库不能引用未成功复制的 file, 已复制但未写入成功的 file 应作为 cleanup candidate 处理.

### Tauri commands

新增 commands:

- `archive_asset`
- `restore_archived_asset`
- `archive_prompt_document`
- `restore_archived_prompt_document`
- `list_archived_content`
- `dry_run_permanent_delete_archived_content`
- `permanent_delete_archived_content`
- `dry_run_merge_library`
- `merge_library`

Commands 只做 path normalization, input validation, error mapping 和 view mapping.

### Desktop frontend

Gallery controller 增加 single archive 和 batch archive action. Prompt workflow controller 增加 prompt archive action. Settings workflow 增加 archived content state 和 merge import state.

Albums detail thumbnail 接入 existing lightbox state. Gallery masonry 通过 CSS 和 thumbnail style 修正, 不改变 Gallery read model.

## Implementation slices

### Slice 1: Archive lifecycle foundation

实现 asset archive / restore / prompt archive / restore, 更新 Gallery, Albums 和 Prompt Library 的默认可见性.

该 slice 不做 physical delete, 不复制 files, 不做 merge import.

### Slice 2: Desktop UX operations

实现 Gallery masonry, Gallery archive actions, Albums thumbnail lightbox, Prompt Library archive action, Settings Archived Content list 和 restore.

Permanent delete 在该 slice 中只接入 dry-run preview, apply action 留到 Slice 3 暴露.

### Slice 3: Permanent delete

实现 dry-run summary 和 apply. 只允许 archived item. 覆盖 asset 和 prompt 两类级联路径.

### Slice 4: Library copy-merge import

实现 merge dry-run 和 merge apply. Source library unchanged. 只迁移创作内容, 跳过 runtime state.

## 风险控制

- Asset archive 涉及 schema migration, 必须定义旧 library 兼容路径.
- Permanent delete 是不可逆操作, 必须只在 Settings Archived Content 中暴露, 并要求 dry-run + confirmation.
- Merge import 可能复制大量文件, 必须显示 file count 和 size, 并提供 recoverable failure semantics.
- Source schema 高于当前版本必须拒绝, 不得尝试降级读取.
- Existing `Import Zip` semantics 不得改变.
- Gallery / Albums workflow ownership 不得回退为 shared implicit selected album state.

## 测试与验证

Core tests:

```bash
cargo test -p imglab-core
```

重点覆盖:

- Archive asset 后 Gallery query 默认不返回, restore 后返回.
- Archive prompt 后 Prompt Library 默认不返回, include archived 可返回.
- Permanent delete 拒绝 active item.
- Permanent delete dry-run 和 apply 使用一致 dependency plan.
- Permanent delete 级联删除 asset versions, album memberships, tags, suggestions 和 generation events.
- Prompt permanent delete 级联删除 prompt versions 和 prompt history references.
- Merge dry-run 拒绝 unsupported schema 和 invalid layout.
- Merge apply 为 source entities 生成新 ID, 并重写 generation event, prompt version, asset version 和 album membership references.

Desktop adapter tests:

```bash
cargo test -p imglab-desktop --lib
```

Frontend tests:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
```

Architecture and hygiene:

```bash
scripts/check-architecture.sh
openspec validate <change> --strict
openspec validate --specs --strict
git diff --check
```

Visual checks:

- Gallery masonry at 960px, 1280px and wide desktop.
- Long image card does not create horizontal overflow.
- Albums thumbnail opens lightbox.
- Prompt archive icon does not trigger row selection.
- Settings Archived Content dry-run confirmation is reachable and does not overlap.

## 非目标

- 不做 federated cross-library view.
- 不迁移 source library queue tasks, schedules, logs, diagnostics, registry alias 或 manifest identity.
- 不在 Gallery 或 Prompt Library 直接提供 permanent delete.
- 不按名称智能合并 album 或 prompt.
- 不做 duplicate image detection 或 content hash dedup.
- 不做 cloud sync, multi-user collaboration 或 remote library merge.
