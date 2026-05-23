## Why

当前 Gallery 已支持 asset version 和 image-to-image parent lineage, 但 Inspector 仍以 list / parent chain 为主, 无法清楚表达一个 asset 下的 branching version tree. 用户需要从任意 version 继续 AI 生成, 并能把某个 child version 提升为新的 Gallery asset root, 同时保留来源可追溯性.

本变更将 Gallery 保持为 asset-first browser, 在 Inspector 内引入 version tree 和 focused version workflow, 避免主 Gallery 被大量 child versions 刷屏.

## What Changes

- Asset detail read model 增加 version tree, focused version, focused lineage 和 tree path label, 例如 `v1`, `v1.1`, `v1.1.1`.
- Gallery card 保持以 logical asset 为单位, 增加当前 focused / preferred version 和 tree summary.
- Inspector 支持点击任意 version tree node, 并让 preview, file context, generation event, lineage 和 action target 聚焦该 version.
- `Generate variation` 必须以当前 focused version 作为 `input_version_id`, 并在同一 asset 下创建 child version.
- 新增 `Promote as new asset` workflow, 将任意 ordinary asset version 创建为新的 Gallery asset root `v1`.
- Promote workflow 保留跨 asset `promoted_from` source relation, 不使用 `parent_version_id` 表达跨 asset 来源.
- Resource library schema 增加 promoted-source relation migration, 并对历史 library 保持兼容.
- Desktop Inspector 新增紧凑 version tree control, 支持鼠标和键盘选择.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `asset-versioning`: 增加 asset version tree read model, path-based tree label, focused version detail, degraded tree handling, 以及 promoted-source source relation 语义.
- `image-generation`: 修改 existing version 图生图 workflow, 明确 output 必须作为 focused input version 的 child version, 并在成功后聚焦新 child.
- `resource-library`: 增加 promoted-source relation 的 schema migration 和历史 library 兼容要求.
- `desktop-workbench`: 增加 Inspector version tree, focused version action target, promote action, keyboard accessibility 和 Gallery card tree summary.

## Impact

- 影响 Rust core DTO, asset detail read model, Gallery read model, SQLite schema migration, asset / generation use cases 和 tests.
- 影响 desktop TypeScript types, Inspector UI, composer variation wiring, Gallery card summary 和 Tauri command wiring.
- 影响 OpenSpec specs: `asset-versioning`, `image-generation`, `resource-library`, `desktop-workbench`.
- 不改变 uploaded reference assets 默认不进入普通 Gallery 查询的语义.
- 不改变 albums, review 和 canonical metadata 的 asset-level ownership.
