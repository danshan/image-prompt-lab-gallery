## Context

当前 resource library 是 local-first managed directory, SQLite 是权威索引, managed image files 位于 library root 下的标准文件布局. 现有 `Import Zip` workflow 用于恢复完整 library backup, 并在 manifest id 冲突时 clone 成新的独立 library. 该语义不能被改成 merge, 否则会破坏已有备份恢复契约.

Desktop 侧已经有 Gallery, Albums, Prompt Library 和 Settings workflows. Gallery 是 all-assets browser, Albums 拥有 add-to-album workflow ownership, Prompt Library 已有 prompt document / version 模型, Settings 已有 Libraries / Providers / Updates / Logs sections. 目前缺口是内容 lifecycle 不完整: 高频删除没有 archive 入口, archived 内容没有集中 restore / permanent delete 管理区, Albums 图墙 thumbnail 没有 full image preview, Gallery 图墙展示被固定 grid 覆盖, 另一个 managed library 也不能 copy-merge 到当前 library.

该 change 涉及持久化格式, read model visibility, destructive operation, file IO 和 cross-library import, 因此必须由 core application / SQLite adapter 拥有语义. Tauri commands 只做 adapter glue, desktop frontend 只做交互和状态刷新. 不允许用 mock 或 fake 逻辑替代真实 core / command 实现.

## Goals / Non-Goals

**Goals:**

- 为 Gallery asset 和 Prompt document 提供真实 archive / restore lifecycle.
- 在 Settings 中提供 archived assets / prompts 列表, 支持 restore 和 permanent delete.
- Permanent delete 只允许 archived item, 必须 dry-run + confirmation, 并执行级联删除.
- Gallery 使用 masonry / 瀑布流展示, 图片按真实比例展示并限制超长图.
- Albums 图墙 thumbnail 支持打开 full image lightbox.
- Settings / Libraries 支持把另一个 managed library 的创作内容 copy-merge 到当前 library.
- Merge import 必须检查 source layout 和 schema compatibility, source library unchanged, target entities 使用新 ID, internal references 按 ID map 重写.
- Existing `Import Zip` backup restore semantics 保持不变.

**Non-Goals:**

- 不做 federated cross-library view.
- 不迁移 source library queue tasks, schedules, logs, diagnostics, registry alias 或 manifest identity.
- 不在 Gallery 或 Prompt Library 直接暴露 permanent delete.
- 不按名称智能合并 album 或 prompt.
- 不做 duplicate image detection 或 content hash dedup.
- 不做 cloud sync, multi-user collaboration 或 remote library merge.

## Decisions

### Decision 1: Asset archive 使用显式持久化字段

Asset archive SHOULD NOT 复用 `assets.status`, 因为 `status` 已经表达 generated, imported, reference 等业务状态. 该 change 增加明确 archive metadata, 例如 `assets.archived_at TEXT`, 并由 Gallery / Albums / Review / add-to-album read models 默认过滤 archived assets.

替代方案是把 `status` 设置成 archived. 该方案实现更短, 但会把 lifecycle state 和业务状态混在一起, 后续 restore 时也难以恢复原状态.

### Decision 2: Prompt archive 使用现有 prompt document archive 语义

Prompt document 已经有 `status` 和 `archived_at`, Prompt Library spec 也已描述归档语义. 该 change 补齐真实 core use case, Tauri command 和 frontend action, 并增加 restore / permanent delete 管理.

### Decision 3: Permanent delete 只能从 Settings Archived Content 发起

Gallery 和 Prompt Library 只提供 archive. Permanent delete 是不可逆维护动作, 必须集中在 Settings Archived Content 中执行, 并要求 dry-run summary + explicit confirmation. Dry-run 和 apply 使用同一个 dependency planner, 避免预览与实际删除范围不一致.

替代方案是在 Gallery card 上直接提供 hard delete. 该方案误触风险高, 且难以在图墙主路径中展示完整级联影响.

### Decision 4: Permanent delete 采用级联删除, 但只接受 archived item

用户明确选择级联删除. 因此 apply 会删除相关 SQLite facts 和 managed files. Active item 必须拒绝 permanent delete. 对 asset, 级联范围包括 versions, generation events, album memberships, tags, metadata suggestions 和 task output references. 对 prompt, 级联范围包括 prompt versions, prompt-linked generation event references 或相关 history references.

为降低不一致风险, database transaction 先完成 SQLite facts 删除. Managed file deletion 在 transaction 成功后执行; file deletion failure 必须返回可修复状态或 orphan cleanup issue, 不得报告完全成功.

### Decision 5: Library merge import 是 copy-merge, 不改变 source

Merge target 是当前 library. Source library 只读参与 compatibility check 和内容读取. Apply 时所有 source entities 在 target 中生成新 ID, managed files 复制到 target 标准路径, source relative paths 不复用, internal references 按 source-to-target ID map 重写.

替代方案是 move-merge 或 attach / federated view. Move-merge 会改变 source library, 恢复难度高. Federated view 会破坏当前 single-library truth source, 并扩大 read model 与 file context 复杂度.

### Decision 6: Merge 只迁移创作内容

Merge 迁移 assets, asset versions, tags, albums, prompt documents, prompt versions, generation events, metadata suggestions 和相关 lineage. 不迁移 queue tasks, schedules, logs, diagnostics, registry alias 和 manifest identity.

Runtime state 与当前 daemon / scheduler / local app 环境绑定, 原样迁移可能恢复旧队列或错误执行旧 schedule. 创作内容则是用户期望 merge 的主要资产.

### Decision 7: 同名对象不智能合并

Merge import 不按 album name 或 prompt name 智能合并. Source 中所有迁移实体复制为 target 新实体. UI 可给 display name 添加 source marker 或后缀. 这保证 semantic safety, 避免同名但含义不同的 album / prompt 被误合并.

## Risks / Trade-offs

- [Risk] Asset archive schema migration 影响所有旧 library 打开路径. → Mitigation: 只做 additive migration, 旧 rows 默认 `archived_at IS NULL`, schema 高于当前版本仍按既有规则拒绝.
- [Risk] Permanent delete 级联删除可能破坏 lineage/history. → Mitigation: 只允许 archived item, dry-run 展示完整影响范围, apply 由同一 dependency planner 驱动, 并覆盖 core tests.
- [Risk] SQLite transaction 和 managed file deletion 无法形成真正原子操作. → Mitigation: 先完成 DB transaction, 后删 files; file deletion failure 返回 repairable issue, 后续 repair / cleanup 可处理 orphan files.
- [Risk] Merge import 大文件复制失败后留下 partial files. → Mitigation: 使用 staging / cleanup candidate 策略, database 不引用未成功复制的 file, 失败时返回 recoverable summary.
- [Risk] Source library migration check 可能修改 source. → Mitigation: source compatibility check 不能在用户 source 上做不可逆写入; 需要使用 staging copy 或明确只读检查路径.
- [Risk] Frontend preview mode 的 mock data 掩盖真实 command 未接通. → Mitigation: production Tauri path 必须通过 real commands; tests 覆盖 command invocation 和 state refresh. Preview fixtures 只能用于非 Tauri local preview, 不能作为 feature completion evidence.
- [Risk] Gallery masonry 改动导致 compact desktop overflow. → Mitigation: 以 960px 为一等验证视口, long image 设置最大展示高度, visual checks 覆盖 960px / 1280px / wide.

## Migration Plan

1. 添加 asset archive metadata 的 schema migration, 保证旧 library 打开后所有既有 assets 默认为 active.
2. 补齐 prompt archive / restore repository operations, 不改变 prompt versions snapshot.
3. 增加 archived content read models 和 lifecycle commands.
4. 增加 permanent delete dry-run planner, 再接 apply.
5. 增加 merge dry-run, 再接 merge apply. Merge apply 在 source compatibility 通过后才允许执行.
6. 更新 desktop UI, 所有 destructive apply action 都通过真实 Tauri command.

Rollback 策略:

- Archive metadata 是 additive schema change, 旧应用若不支持更高 schema 将按 schema mismatch 拒绝打开.
- Archive / restore 可逆.
- Permanent delete 不可逆, 必须依靠 dry-run + confirmation 降低误删风险.
- Merge import 是 copy-merge, source library unchanged, 因此 merge 后若 target 结果不满意, 用户可从 target 中归档或删除迁入内容, source 仍可重新导入.

## Open Questions

- 无. 用户已确认 copy-merge, 创作内容范围, 冲突策略和 permanent delete 级联语义.
