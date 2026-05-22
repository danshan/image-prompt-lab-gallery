# Gallery Version Tree Design

Date: 2026-05-23

## Summary

本设计为 Gallery 引入 tree-shaped asset version workflow.

核心结论:

- Gallery 主网格继续以 logical asset 为卡片单位, 不把每个 child version 展开成独立 Gallery card.
- 右侧 Inspector 新增 version tree, 支持从任意 version 继续生成 child version.
- 用户可见 version tree name 使用 path-based label, 例如 `v1`, `v1.1`, `v1.1.1`.
- 子版本进入 Gallery 支持两种语义:
  - `Focus in original asset`: 保留在原 asset tree 中, 只让 Inspector 聚焦该 child version.
  - `Promote as new asset`: 创建新的 logical asset, 该图片成为新 asset 的 root `v1`, 同时保留跨 asset promoted-source link.

## Context

现有系统已经有 `Asset`, `AssetVersion`, `parent_version_id`, `generation_event`, `input_version_id`, 以及用户可见 numeric version name.

当前问题是:

- Inspector 和 read model 更接近 version list / parent chain, 不能表达 branching version tree.
- 用户需要从任意 version 继续生成, 但 UI 必须清楚显示新 output 是哪个 parent 的 child.
- 用户需要把某个 child version 放入 Gallery, 但这有两种不同语义:
  - 作为新 Gallery asset 的独立 root.
  - 仍作为原 asset 的 child version, 点击 Gallery asset 时右侧聚焦该 child.

既有约束:

- 内部稳定 identity 继续使用 asset version UUID.
- `parent_version_id` 只表达同一 asset 内的 parent-child version relationship.
- Uploaded reference images 仍作为独立 reference asset/version 管理, 不并入 output asset lineage.
- Gallery 默认展示普通 assets, 不展示 uploaded reference assets.
- Gallery 和 Albums 的职责已经拆分, 本设计不重新混合页面状态 ownership.

## Goals

- 在 asset detail 中展示完整 version tree, 而不是 flat list.
- 支持从任意 tree node 发起 image-to-image variation.
- 支持将任意 child version promote 为新的 Gallery asset root.
- 保持 Gallery 主网格稳定, 避免同一 asset 的大量 child versions 直接刷屏.
- 保留跨 asset 来源可追溯性, 不滥用 `parent_version_id` 表达跨 asset 来源.
- 对历史 library 提供兼容和可升级路径.

## Non-Goals

- 不实现全局 graph-style lineage visualization.
- 不把 Gallery item 抽象成可同时指向 asset 或 version 的 projection model.
- 不改变 album membership, review metadata, canonical metadata 的 asset-level ownership.
- 不改变 uploaded reference asset 默认不进入普通 Gallery 查询的语义.
- 不引入 cloud sync, multi-user collaboration, 或 resource library encryption.

## UX Design

### Gallery Grid

Gallery card 继续代表 logical asset.

卡片展示:

- 当前 focused 或 preferred version thumbnail.
- Asset title, provider, rating, tags.
- Current version tree name, 例如 `v1.2`.
- Tree summary, 例如 `5 versions / 3 branches`.

Gallery 不展示所有 child version card. 只有用户执行 `Promote as new asset` 后, Gallery 才出现新的 asset card.

### Inspector Layout

Inspector 分为四个稳定区域:

1. Asset metadata: title, status, category, rating, tags, albums.
2. Focused version preview: image, file context, provider/model, prompt, parameters, generation event.
3. Version Tree: tree node list, 支持点击和键盘选择.
4. Focused version actions: generate variation, promote as new asset, preview.

Version Tree 使用紧凑 tree control:

- Root 显示 `v1`.
- Child 显示 `v1.1`, `v1.2`.
- Deeper child 显示 `v1.1.1`.
- 当前 focused node 高亮.
- 每个 node 显示 minimal metadata: thumbnail sliver 或 icon, tree name, created time, provider shorthand.
- 长路径需要稳定宽度处理, 例如 middle truncation, tooltip 展示完整 path.

### Focus Flow

点击 Gallery card:

- 选中 `selectedAssetId`.
- 加载 asset detail.
- 默认 focus preferred version. Preferred version 可以是最近创建 version, 或后续保存的 last-focused version.

点击 version tree node:

- 更新 Inspector local `focusedVersionId`.
- Preview, file context, generation event, lineage 和 action target 全部切换到该 version.
- Gallery card identity 不变, 仍是同一个 asset.

### Generate Variation

`Generate variation` 从当前 `focusedVersionId` 发起:

- Composer 使用 `focusedVersionId` 作为 `input_version_id`.
- Core 在同一 asset 下创建 new child version.
- 新 child 的 `parent_version_id` 指向 focused version.
- 成功后 Inspector 展开 parent branch 并 focus 新 node.

### Promote As New Asset

`Promote as new asset` 从当前 focused version 创建新 logical asset:

- 新 asset 进入普通 Gallery.
- 新 asset 创建 root version, `parent_version_id = null`, `version_number = 1`, tree name = `v1`.
- 新 asset Inspector 展示 `Promoted from` summary, 指向 source asset 和 source version.
- 原 asset tree 不变.

用户层语义:

- Promote 后的图片是独立 Gallery item.
- 新 asset 可以独立编辑 metadata, 加入 albums, 进入 review, 继续生成自己的 child tree.
- 系统仍保留它来自哪个 source version.

## Domain Model

继续保持两层核心模型:

- `Asset`: logical image work, 是 Gallery card, canonical metadata, albums, review 的 owner.
- `AssetVersion`: concrete image file, 通过 `parent_version_id` 表达同一 asset 内 tree.

`parent_version_id` 的边界:

- 只允许引用同一 asset 的 version.
- 不用于 uploaded reference source.
- 不用于 promoted-from source.
- 跨 asset 来源通过独立 source relation 表达.

`version_number` 继续保留:

- 用于 migration-friendly numeric sequence.
- 用于 stable sorting fallback.
- 不直接等同于 tree path label.

新增 read-model-only field:

- `version_tree_name`: path-based user-facing name, 例如 `v1.1`.

`version_tree_name` 不作为外键, 不保证在 reorder policy 改变后仍是永久 identity. Machine input 继续使用 version UUID.

## Version Tree Naming

命名规则:

- Root version: `v1`.
- 同一 parent 下 children 按 `created_at ASC, id ASC` 排序.
- 第一个 child: `parent_tree_name + ".1"`.
- 第二个 child: `parent_tree_name + ".2"`.

示例:

```text
v1
+-- v1.1
|   +-- v1.1.1
+-- v1.2
```

Legacy multi-root case:

- 如果一个 asset 下存在多个 root versions, read model 按 `created_at ASC, id ASC` 显示为 `v1`, `v2`, etc.
- 后续 generate variation 从 focused node 继续派生, 不主动重写历史 parent links.

## Read Models

### Asset Detail

Asset detail read model 扩展:

- `focused_version_id`
- `focused_version`
- `focused_version_tree_name`
- `version_tree`
- `focused_lineage`
- `source_reference`
- `promoted_from`

`version_tree` 节点建议包含:

- `version_id`
- `parent_version_id`
- `tree_name`
- `version_number`
- `image_path`
- `created_at`
- `provider`
- `model_label`
- `generation_status`
- `children`

### Gallery Asset Card

Gallery card read model 增加 tree summary:

- `focused_version_id`
- `focused_version_tree_name`
- `version_tree_node_count`
- `version_tree_branch_count`
- `has_promoted_source`

Gallery card 不需要完整 tree. 完整 tree 只在 asset detail / Inspector 查询中返回.

## Persistence

Version tree 本身可以由现有 `asset_versions.parent_version_id` 构造, 不要求新增字段.

为 `Promote as new asset` 建议新增 source relation 表:

```text
asset_version_sources
  id
  target_version_id
  source_asset_id
  source_version_id
  source_kind
  created_at
```

约束:

- `source_kind` 初期支持 `promoted_from`.
- `target_version_id` 指向新 asset 的 root version.
- `source_version_id` 指向原 asset 的 selected child version.
- 同一个 target version 对同一 source kind 只允许一条 source relation.

Storage policy:

- 初期优先复制 managed file 到新 version path, 保持 library repair 和 export 简单.
- 如果未来引入 dedup / hard-link, 应隐藏在 storage layer 内, 不改变 read model.

事务边界:

- Promote 必须在业务事务中创建 asset, root version, source relation.
- 文件复制失败时不能留下普通 Gallery 可见的坏 asset.
- 如果存储层无法纳入同一个 SQLite transaction, 应采用先复制到 temporary managed path, 再提交 DB, 最后 finalize path 的 staged flow.

## Migration And Compatibility

新增 promoted-source 表需要 schema migration.

Migration 规则:

- Existing libraries without source relation table can open and migrate idempotently.
- Existing versions use current `parent_version_id` to build tree.
- Existing numeric `version_number` remains valid.
- Existing `version_name` such as `v1`, `v2` remains available as legacy / fallback display, but Inspector tree uses `version_tree_name`.

Degraded cases:

- Parent points to missing version: show orphan node group and repair hint.
- Parent points to different asset: treat as invalid parent link, do not merge cross asset tree.
- Cycle detected: break traversal, show degraded tree error, keep Gallery query alive.

## Error Handling

Generate variation:

- Missing focused version returns recoverable domain error.
- Provider lacks image-to-image capability returns existing unsupported capability error.
- UI keeps current selection and composer draft.

Promote as new asset:

- Missing source file returns recoverable error and creates no visible new asset.
- Checksum mismatch blocks promote unless future repair flow explicitly allows it.
- Source version from reference asset is allowed only if product decides reference promotion is valid. Default should require ordinary generated/imported asset versions.

Version tree:

- Invalid tree data should degrade Inspector only.
- Gallery query should not fail because one asset has corrupt parent links.
- UI should make degraded state visible without replacing the whole Inspector with an error page.

## Accessibility And Interaction Quality

This design follows dense operational dashboard guidance:

- Use visible focus rings on tree nodes and icon buttons.
- Tree control must be keyboard reachable.
- Arrow up/down moves through visible nodes.
- Enter selects focused node.
- Right expands collapsed node.
- Left collapses expanded node or moves to parent.
- Primary actions must be click/tap based, not hover-only.
- Hover state should use color, border, or background transitions, not layout-shifting scale transforms.
- Respect reduced-motion preferences.
- Image alt text should describe the asset title or fallback to version tree name.

Visual style:

- Keep current restrained desktop application tone.
- Use neutral background, high contrast text, and one functional accent.
- Avoid decorative blobs, excessive gradients, and nested cards.
- Use icons from the existing app icon system or a consistent SVG icon set.
- Keep card radius at or below existing system radius.

## Testing Plan

Core tests:

- `asset_detail_builds_version_tree_from_parent_links`
- `version_tree_names_are_path_based_and_sibling_ordered`
- `generate_variation_creates_child_under_focused_version`
- `promote_version_creates_new_asset_root_with_promoted_source`
- `legacy_multi_root_versions_degrade_predictably`
- `invalid_cross_asset_parent_does_not_merge_trees`
- `cycle_in_version_parent_links_is_reported_as_degraded_tree`

Migration tests:

- Old library without source relation table opens after migration.
- Source relation table migration is idempotent.
- Existing asset versions keep numeric `version_number`.

UI tests:

- Clicking a version node updates preview and action target.
- `Generate variation` sends focused version id.
- `Promote as new asset` refreshes Gallery and selects new asset.
- Tree keyboard navigation works.
- Long labels such as `v1.2.3.4` do not overflow Inspector.

Manual visual QA:

- Default desktop width.
- Narrow laptop width.
- Asset with many siblings.
- Asset with deep branch.
- Empty / missing file degraded state.
- Promoted asset with source summary.

## Implementation Notes

Likely affected areas:

- `openspec/specs/asset-versioning/spec.md`
- `openspec/specs/image-generation/spec.md`
- `crates/imglab-core/src/domain/asset`
- `crates/imglab-core/src/application/use_cases/assets.rs`
- `crates/imglab-core/src/application/use_cases/generation.rs`
- `crates/imglab-core/src/library/assets.rs`
- `crates/imglab-core/src/library/gallery.rs`
- `crates/imglab-core/src/infrastructure/sqlite/schema.rs`
- `apps/desktop/src/app/types.ts`
- `apps/desktop/src/app/StudioAppController.tsx`
- `apps/desktop/src/app/screens/workflows/*`
- `apps/desktop/src/app/screens/gallery/GalleryWorkspace.tsx`

Recommended implementation order:

1. Add OpenSpec change for tree version read model and promote workflow.
2. Add core read-model tree builder with tests.
3. Add source relation persistence and migration.
4. Add promote use case.
5. Wire Tauri DTOs and desktop types.
6. Add Inspector version tree UI.
7. Wire generate variation to focused version.
8. Add promote action and refresh flow.
9. Run core, desktop, and architecture checks.
