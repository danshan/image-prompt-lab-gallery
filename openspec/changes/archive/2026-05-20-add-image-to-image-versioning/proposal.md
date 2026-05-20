## Why

当前项目已经有图生图和版本 lineage 的基础边界, 但用户可见版本仍依赖 UUID 或弱语义 label, 上传参考图也没有明确的 managed lineage 语义. 这会让基于已有图片继续生成, 上传参考图生成, 历史追溯和长期 library 兼容都变得含糊.

## What Changes

- 为每个 asset 增加用户可见的数字版本号, 从 `v1` 开始按 asset 内部递增.
- 保留 UUID 作为内部稳定主键和外键, 不把 UUID 作为默认用户可见版本名称.
- 支持基于已有 asset version 执行图生图, 并在同一 asset 下创建下一数字版本.
- 支持上传本地图片作为 reference asset/version, 再生成独立 output asset/version.
- 明确 reference asset 默认不混入普通 Gallery, 但可通过 generation source link 追溯.
- 为历史 library 增加 schema migration, 回填 `version_number`, 保持旧备份和旧 library 可升级.
- 更新 CLI, Desktop, task output 和 read model, 让输出和展示包含数字版本与 reference source 信息.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `asset-versioning`: 增加 per-asset 数字版本号, 明确 UUID 与用户可见版本名称的边界, 更新 lineage read model.
- `image-generation`: 明确 existing version 图生图与 uploaded reference 图生图的不同输出语义, provider capability 和失败行为.
- `resource-library`: 增加历史 library schema migration 和 reference asset 默认可见性规则.
- `desktop-workbench`: Gallery, Inspector, Generate variation 和 uploaded reference source 展示改为使用数字版本号和 reference source.
- `cli-automation`: CLI 的 `--input-version` 和 `--input-file` 输出需要包含数字版本和 reference summary.

## Impact

- Rust core schema migration, DTO, read model, asset import/create child version, generation orchestration.
- Daemon image generation task input/output reconciliation 和 task output links.
- Desktop Tauri command DTO mapping 和 React Gallery/Inspector/Generate workflow.
- CLI generation output 和 tests.
- OpenSpec specs for asset versioning, image generation, resource library, desktop workbench 和 CLI automation.
