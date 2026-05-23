## Overview

本变更采用 `Asset-first Gallery + Inspector version tree + promoted-root asset` 方案.

Gallery 主网格继续以 logical asset 为卡片单位. Version tree 是当前 asset 的局部导航, 只在 Inspector 中展开. 用户从任意 tree node 生成 variation 时, output 作为该 node 的 child version. 用户将 child version 放入 Gallery 时, 通过 `Promote as new asset` 创建新的 logical asset, 新 asset 的 root version 显示为 `v1`, 并通过 promoted-source relation 保留来源.

## Data Model

核心模型继续保持两层:

- `Asset`: Gallery card, canonical metadata, albums 和 review 的 owner.
- `AssetVersion`: concrete image file, 通过 `parent_version_id` 表达同一 asset 内的 tree.

`parent_version_id` 的边界保持严格:

- 只允许指向同一 asset 下的 version.
- 不表达 uploaded reference source.
- 不表达 promoted-from source.
- 跨 asset source 使用独立 relation.

为 promote workflow 增加 source relation:

```text
asset_version_sources
  id
  target_version_id
  source_asset_id
  source_version_id
  source_kind
  created_at
```

初期 `source_kind` 只需要支持 `promoted_from`. `target_version_id` 指向新 asset 的 root version, `source_version_id` 指向原 asset 中被 promote 的 version.

`version_number` 继续保留为 asset 内递增数字序列, 用于 migration-friendly ordering 和 fallback display. 用户在 tree UI 看到的 `version_tree_name` 是 read-model-only path label, 不作为稳定 identity.

## Version Tree Read Model

Asset detail read model 扩展为:

- `focused_version_id`
- `focused_version`
- `focused_version_tree_name`
- `version_tree`
- `focused_lineage`
- `source_reference`
- `promoted_from`

Version tree node 包含:

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

Tree name 计算规则:

- Root version 显示为 `v1`.
- 同一 parent 下 children 按 `created_at ASC, id ASC` 排序.
- 第一个 child 显示为 `parent_tree_name + ".1"`.
- 第二个 child 显示为 `parent_tree_name + ".2"`.

示例:

```text
v1
+-- v1.1
|   +-- v1.1.1
+-- v1.2
```

如果历史 asset 有多个 root versions, read model 按 `created_at ASC, id ASC` 显示为 `v1`, `v2`, etc. 后续新 variation 从 focused node 继续派生, 不主动重写历史 parent links.

## Gallery And Inspector UX

Gallery card 继续代表 logical asset. Card 展示当前 focused 或 preferred version thumbnail, asset title, provider, rating, tags, 当前 tree name 和 tree summary, 例如 `5 versions / 3 branches`.

Inspector 新增 `Version Tree` 区块:

- Tree node 点击后更新 `focusedVersionId`.
- Preview, file context, generation event, lineage 和 action target 全部切换到 focused version.
- Gallery item identity 不变.
- 长路径使用稳定宽度处理, 不挤压 Inspector.

Focused version actions:

- `Generate variation`: 使用 focused version id 打开 composer.
- `Promote as new asset`: 调用 promote command, 成功后刷新 Gallery 并选中新 asset.
- `Preview`: 打开 lightbox.

Tree control 必须可键盘操作:

- Arrow up/down 移动 visible node focus.
- Enter 选择 node.
- Right 展开 collapsed node.
- Left 收起 expanded node 或移动到 parent.

## Promote Workflow

Promote workflow:

1. 校验 source version 存在, 属于 ordinary generated/imported asset, 且 source file 可读.
2. 复制 source managed file 到新的 managed version path.
3. 创建新 asset.
4. 创建新 root version, `parent_version_id = null`, `version_number = 1`.
5. 创建 `asset_version_sources` relation, `source_kind = promoted_from`.
6. 返回新 asset / version summary.

如果文件复制失败, 不能留下普通 Gallery 可见的坏 asset. 如果存储层无法和 SQLite transaction 原子提交, 使用 temporary path staging, DB commit 成功后 finalize managed path.

## Error Handling

Generate variation:

- Focused version 不存在时返回 recoverable domain error.
- Provider 不支持 image-to-image 时返回 existing unsupported capability error.
- UI 保留当前 selection 和 composer draft.

Promote:

- Source file 缺失或 checksum mismatch 时阻止 promote.
- 默认不允许直接 promote uploaded reference asset version, 除非后续产品语义显式放开.
- Promote 失败不应刷新出半成品 Gallery card.

Tree degradation:

- Parent 缺失: 显示 orphan group 和 repair hint.
- Parent 指向不同 asset: 不合并跨 asset tree, 显示 invalid parent degraded state.
- Cycle: 截断 traversal, 标记 degraded tree, Gallery query 保持可用.

## Migration

Schema version 增加 promoted-source relation table.

Migration 必须:

- 对旧 library 幂等创建 `asset_version_sources`.
- 不重写现有 `asset_versions.parent_version_id`.
- 保留现有 `version_number` 和 `version_name`.
- 允许没有 promoted-source rows 的 library 正常打开.

## Testing

Core tests:

- Asset detail 从 parent links 构造 tree.
- Tree names 按 path 和 sibling order 计算.
- Existing version 图生图从 focused version 创建 child.
- Promote 创建新 asset root 和 promoted-source relation.
- Legacy multi-root, missing parent, cross-asset parent 和 cycle 都 degrade predictably.

Desktop tests:

- 点击 tree node 更新 preview 和 action target.
- Generate variation 发送 focused version id.
- Promote 成功后刷新 Gallery 并选中新 asset.
- Tree keyboard navigation 和 focus ring 可用.
- 长 label 不溢出 Inspector.

Verification:

- `openspec validate add-gallery-version-tree --strict`
- Focused core tests.
- Desktop tests / type check.
- `scripts/check-architecture.sh`
