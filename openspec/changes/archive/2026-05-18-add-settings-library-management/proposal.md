## Why

当前 Settings 只能处理最基础的 Library path/name 输入和 create/open 操作, 无法承担多个 Library 的长期维护. 随着项目已经具备 registry, managed library, Gallery, Review, Queue 和 Logs 等能力, Settings 需要把 Library 生命周期维护和诊断日志明确拆开, 避免用户把 close, rename, backup 和 asset export 语义混淆.

## What Changes

- 在 Settings 内新增 `Libraries` 和 `Logs` 两个子页, 默认进入 `Libraries`.
- `Settings / Libraries` 提供多个 Library 的维护入口: 创建, 打开已有文件夹, 本机别名重命名, 取消注册, 完整 zip 备份导出, 完整 zip 备份导入, Reveal in Finder.
- `Close` 语义改为从 app-level registry 取消注册, 不删除磁盘上的 Library 文件.
- `Rename` 语义限定为本机 registry display name, 不修改 Library `manifest.json`.
- 新增完整 Library backup zip workflow. Settings 导出的 zip 用于恢复一个可打开的 Library, 不作为 Gallery/Album asset-level export.
- 导入 backup zip 时解压到新的本地 Library 目录并注册. 如果导入包内的 `manifest.id` 已存在于 registry, 系统自动生成新 id 并作为 clone 注册.
- `Settings / Logs` 保留当前 app logs 浏览能力, 但不再和 Library lifecycle actions 混在同一个 Settings grid 中.

## Capabilities

### New Capabilities

- 无.

### Modified Capabilities

- `resource-library`: 扩展 app-level registry 维护语义, 增加本机 alias rename, unregister close, 完整 Library backup zip export/import, 以及导入 id 冲突时 clone 的行为约束.
- `desktop-workbench`: 扩展 Settings workspace 信息架构, 增加 `Libraries` 和 `Logs` 子页, 并在 `Libraries` 中提供多个 Library 的维护表格和操作状态.
- `app-logs`: 明确 Logs 作为 `Settings / Logs` 子页保留, 与 Library lifecycle actions 分离.

## Impact

- `crates/imglab-core`: 增加 registry alias rename, unregister, backup zip export/import 相关 service API 和测试.
- `crates/imglab-core/src/library/*`: 增加 backup zip 打包, staging 解压, layout 校验, manifest id rewrite 和 registry 注册逻辑.
- `apps/desktop/src-tauri`: 增加 Tauri commands, native dialog 或文件选择集成, Reveal in Finder OS integration.
- `apps/desktop/src`: 拆分 Settings UI 为 Libraries 和 Logs 子页, 增加 Library maintenance table, operation state 和 current-library cleanup flow.
- `openspec/specs/resource-library`, `openspec/specs/desktop-workbench`, `openspec/specs/app-logs`: 更新行为要求.
- 可能新增 zip 处理依赖. 该依赖仅用于完整 Library backup workflow, 不改变现有 managed library layout.
