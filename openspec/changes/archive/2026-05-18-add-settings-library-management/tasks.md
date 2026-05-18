## 1. Core Registry Semantics

- [x] 1.1 在 `imglab-core` service trait 和 DTO 中增加 registry alias rename 与 unregister 所需的 request/summary 类型.
- [x] 1.2 实现 `rename_library_alias`, 只更新 app-level registry display name, 并拒绝空白 alias.
- [x] 1.3 实现 `unregister_library`, 从 registry 中移除 entry, 不触碰 Library root 文件.
- [x] 1.4 调整或补充 `list_libraries` 兼容旧 hidden registry rows, 但新 close path 不写 hidden.
- [x] 1.5 添加 core tests 覆盖 alias rename 不修改 manifest, unregister 不删除目录, unknown id 返回 `LibraryNotFound`.

## 2. Core Backup Zip Workflow

- [x] 2.1 选择不引入外部 zip 依赖, 在 core backup workflow 内实现窄边界 stored-method ZIP codec.
- [x] 2.2 实现 `export_library_backup_zip`, 导出前校验 Library layout, 并包含 manifest, database, required dirs 和 managed files.
- [x] 2.3 为 export 使用临时 zip path 和最终 rename, 避免失败时返回半成品成功结果.
- [x] 2.4 实现 `import_library_backup_zip` staging 解压, backup layout 校验和失败清理.
- [x] 2.5 实现导入 id 冲突时生成新 library id, 重写导入目录 manifest, 并作为 clone 注册.
- [x] 2.6 添加 core tests 覆盖有效 zip 导入导出, 无效 zip, 非安全目标目录, id 冲突 clone, 失败不注册半成品.

## 3. Desktop Tauri Commands

- [x] 3.1 增加 `rename_library_alias`, `unregister_library`, `export_library_backup_zip`, `import_library_backup_zip` commands.
- [x] 3.2 增加 Reveal in Finder command, 对 missing path 返回 recoverable error.
- [x] 3.3 增加或接入 Tauri dialog 能力, 支持选择 Library folder, zip input, zip save path 和 import destination.
- [x] 3.4 确保 command input/output 使用现有 camelCase mapping 和 `CommandError` 约定.
- [x] 3.5 视当前 harness 可行性补充 Tauri command tests 或 smoke checklist.

## 4. Settings Frontend

- [x] 4.1 将现有 `SettingsView` 拆分为 `SettingsWorkspace`, `SettingsLibrariesView`, `SettingsLogsView`.
- [x] 4.2 增加 `settingsSection` state, 默认 `libraries`, 并保证切换 `libraries` / `logs` 不改变当前 Library.
- [x] 4.3 在 `Settings / Libraries` 实现 toolbar: Create Library, Open Existing Library, Import Zip.
- [x] 4.4 在 `Settings / Libraries` 实现 registered libraries table, 展示 name, path, schema, status 和 row actions.
- [x] 4.5 实现 Switch, Rename, Close, Export Zip, Reveal in Finder row actions 和 per-action pending state.
- [x] 4.6 Close 当前 Library 后清空 current library, gallery, inspector detail, albums, suggestions, tasks 和 selected ids, 进入 no-library state.
- [x] 4.7 Import Zip 成功后刷新 registry 并默认切换到导入后的 Library.
- [x] 4.8 保留并迁移现有 Logs browser 到 `Settings / Logs`.
- [x] 4.9 补充 missing path row 状态: 标记 missing on disk, Close 可用, Export Zip 和 Reveal in Finder 禁用或显示 recoverable error.

## 5. Frontend State Tests And Styling

- [x] 5.1 在 `workbench-state` 中提取或补充 close-current-library cleanup helper.
- [x] 5.2 添加 frontend state tests 覆盖 Settings 默认子页, 子页切换不改变 current Library, close current 清理上下文.
- [x] 5.3 添加 frontend state tests 覆盖 rename current alias 后展示更新, missing path action availability.
- [x] 5.4 更新 CSS, 保证 Settings tabs, Libraries table, row actions, long path truncation 和 Logs layout 在窄宽度下不重叠.

## 6. Verification

- [x] 6.1 运行 `cargo test -p imglab-core`.
- [x] 6.2 运行 `cargo test -p imglab-desktop` 或记录当前不可行原因.
- [x] 6.3 运行 `npm test --prefix apps/desktop`.
- [x] 6.4 运行 `npm run build --prefix apps/desktop`.
- [ ] 6.5 手工 smoke test Create, Open Existing, Rename, Close, Export Zip, Import Zip, Reveal in Finder, Settings Logs 子页切换.
- [x] 6.6 运行 OpenSpec status/validation, 确认 change apply-ready.
