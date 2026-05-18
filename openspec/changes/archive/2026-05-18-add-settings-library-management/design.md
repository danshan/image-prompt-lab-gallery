## Context

项目已经具备 managed resource library, app-level registry, Sidebar Library selector, Settings Logs, 以及 Rust core service boundary. 当前 Settings 页面仍是原型阶段结构: Library path/name 输入, create/open 按钮和 Logs browser 混在同一个 grid 中. 这导致多个 Library 的维护动作缺少明确入口, 也容易把本机 registry 操作, portable library identity, 完整备份和 asset-level export 混淆.

本变更跨越 core, Tauri command 和 React Settings UI. 备份包实现选择窄边界 ZIP stored-method codec, 避免为本地备份 workflow 引入新的远程依赖, 因此需要先明确边界和失败语义.

## Goals / Non-Goals

**Goals:**

- 将 Settings 拆为 `Libraries` 和 `Logs` 两个子页.
- 在 `Settings / Libraries` 中提供多个 Library 的维护表格和操作入口.
- 将 `Rename` 定义为本机 registry alias 修改, 不修改 `manifest.json`.
- 将 `Close` 定义为 unregister, 不删除磁盘文件.
- 增加完整 Library backup zip export/import, 与现有 asset/sidecar export 分离.
- 导入 backup zip 时, 如果 `manifest.id` 已存在, 自动生成新 id 并作为 clone 注册.
- 保持 Rust core 是 Library registry 和 backup 语义的事实边界.

**Non-Goals:**

- 不实现删除 Library 磁盘目录.
- 不把 backup zip 合并进当前 Library.
- 不改变 managed library 目录布局.
- 不引入云同步, 多用户协作, 加密或 daemon 化 Library 管理.
- 不重写 Gallery/Album asset-level export.

## Decisions

### 1. Settings 使用两个子页, 而不是继续扩展单页 grid

`Settings / Libraries` 承载 Library 生命周期维护, `Settings / Logs` 承载诊断日志. 顶层 Sidebar 仍保留一个 Settings 入口, Settings workspace 内使用本地 tab 或 segmented control.

备选方案是继续在单页 Settings grid 中追加 Library 表格. 该方案改动更小, 但 Library maintenance table 和 Logs browser 都属于高信息密度模块, 放在同页会使状态和视觉层级快速混乱. 两个子页能保持 Settings 可扩展, 也符合用户明确要求.

### 2. Rename 只更新 registry alias

Library `manifest.json` 表示 portable library identity, 不应因为某台机器上的显示偏好被修改. 因此 `rename_library_alias(library_id, alias)` 只更新 app-level registry display name. Export backup zip 仍保留 Library manifest 内的原始 name.

备选方案是同步修改 manifest name. 该方案更直观, 但会把本机 UI 标签变成 portable metadata mutation, 导致 backup/restore 后含义不稳定.

### 3. Close 执行 unregister, 不复用 hide 作为用户语义

现有 registry 有 hidden 字段, 但新 UI 不暴露 hide. `Close` 应删除 registry entry 或等价地让该 Library 不再出现在 registry 列表中, 且不触碰磁盘文件. 如果关闭当前 Library, 前端必须清空当前 Library 相关上下文.

备选方案是继续写 hidden. 该方案可以复用已有实现, 但会留下用户不可见状态, 长期会让 close/unhide/open existing 的行为难以解释. 旧 hidden 行为可保留兼容读取, 新 close 语义应是 unregister.

### 4. 完整 backup zip 使用独立 API 和窄边界 ZIP codec, 不复用现有 directory export

现有 `export_library` 更接近 asset/sidecar export, 目标是内容交换或 Album/Library 内容导出. Settings 的 `Export Zip` 是完整 Library snapshot, 目标是 restore 一个可打开 Library. 两者需要分离 API:

- `export_library_backup_zip(library_path, output_zip_path)`.
- `import_library_backup_zip(zip_path, destination_dir)`.

实现上使用本项目内的 stored-method ZIP codec, 只覆盖本应用完整备份导出和导入所需的文件场景. 这样可以避免在同一个 API 中混入不同的数据完整性和恢复语义, 也避免新增外部依赖对离线构建的影响.

### 5. Import 使用 staging 目录, 注册是最后一步

导入 zip 时先解压到 staging directory, 校验 manifest, database, schema 和 required dirs. 校验通过后再移动到目标目录并注册. 这能避免失败时留下已注册但不可打开的半成品.

如果导入包内 `manifest.id` 已存在于 registry, core 自动生成新 id 并重写导入目录的 manifest, 作为 clone 注册. 这比替换原 registry entry 更安全, 也符合用户选择.

### 6. Reveal in Finder 留在 Tauri 层

Reveal 是 OS integration, 不属于 core domain. Core 只处理 Library registry, layout, manifest, SQLite 和 backup 语义. Tauri command 负责打开 Finder 或返回 recoverable error.

## Risks / Trade-offs

- [Risk] 自写 stored-method ZIP codec 覆盖范围比通用 zip 库窄. → Mitigation: 该 codec 只作为本应用 backup zip 的读写实现, 拒绝 unsupported compression, 并用 core tests 覆盖文件列表, staging 和失败清理.
- [Risk] 导出过程中复制正在变化的 SQLite 可能得到不一致 snapshot. → Mitigation: MVP 要求导出前校验 layout, 使用临时 zip 文件和最终 rename. 更严格的 SQLite backup API 可以作为后续增强, 当前单用户 local-first 场景先避免隐式 repair 或复杂锁协议.
- [Risk] unregister 当前 Library 后前端残留旧 selection 或 task context. → Mitigation: 抽出或复用 library switch cleanup helper, 为 close-current 增加 frontend state test.
- [Risk] registry alias 和 manifest name 并存后用户可能困惑. → Mitigation: UI 主要展示 alias, backup/import 相关状态文案明确 alias 是本机显示名.
- [Risk] missing path row 的 action 可用性不一致. → Mitigation: `Close` 始终可用, `Export Zip` 和 `Reveal in Finder` 对 missing path 禁用或返回 recoverable error.

## Migration Plan

1. 保留既有 registry schema 和 hidden 字段兼容读取.
2. 增加 unregister 行为, 新 UI 不再暴露 hide.
3. 增加 alias rename 和 backup zip API, 不改变现有 Library layout.
4. 前端 Settings 拆分为 Libraries 和 Logs, 默认进入 Libraries.
5. 实现完成后通过 OpenSpec archive 将 delta specs 合并进当前 specs.

Rollback strategy: 本变更不自动删除用户数据. 如果 backup zip 或 Settings UI 出现问题, 可以回滚代码. 已 unregister 的 Library 仍可通过 Open Existing Library 重新注册; 已导入 clone 是独立 Library 目录, 可由用户在文件系统中保留或手工删除.

## Open Questions

无阻塞问题. 实现时可以根据 Tauri v2 当前依赖选择前端 dialog API 或 thin Tauri commands, 但这不改变核心行为契约.
