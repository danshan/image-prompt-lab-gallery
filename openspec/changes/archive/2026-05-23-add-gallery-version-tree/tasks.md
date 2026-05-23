## 1. OpenSpec Artifacts

- [x] 1.1 完成 proposal, design, specs 和 tasks.
- [x] 1.2 运行 OpenSpec change validation.

## 2. Core Version Tree Read Model

- [x] 2.1 扩展 core DTO, 增加 version tree node, focused version fields, promoted source summary 和 Gallery tree summary.
- [x] 2.2 实现 asset detail version tree builder, 覆盖 path-based tree label, sibling ordering 和 legacy multi-root.
- [x] 2.3 实现 degraded tree handling, 覆盖 missing parent, cross-asset parent 和 cycle.
- [x] 2.4 更新 Gallery / asset detail SQLite read path, 返回 version tree 和 card tree summary.
- [x] 2.5 增加 core tests 覆盖 version tree read model 和 degraded cases.

## 3. Promote As New Asset

- [x] 3.1 增加 `asset_version_sources` schema migration 和 idempotent migration test.
- [x] 3.2 增加 source relation persistence helpers 和 read model mapping.
- [x] 3.3 实现 promote use case, 创建新 asset root version 并记录 `promoted_from`.
- [x] 3.4 增加 promote workflow tests, 覆盖成功, source missing, checksum mismatch 和 promoted source detail.
- [x] 3.5 暴露 Tauri command / core facade, 供 desktop 调用 promote workflow.

## 4. Image Generation Focused Version Flow

- [x] 4.1 确认 generation use case 使用 focused `input_version_id` 创建 child version.
- [x] 4.2 更新生成成功后的 desktop refresh / focus flow, 聚焦新 child version.
- [x] 4.3 增加 focused generation tests 或更新现有 tests.

## 5. Desktop Inspector UI

- [x] 5.1 扩展 desktop TypeScript types 和 DTO adapter.
- [x] 5.2 新增 Inspector version tree component, 支持 click 和 keyboard navigation.
- [x] 5.3 将 Inspector preview, file context, generation event, lineage 和 actions 绑定到 focused version.
- [x] 5.4 在 Gallery card 展示 focused tree label 和 tree summary.
- [x] 5.5 实现 `Promote as new asset` action, 成功后刷新 Gallery 并选中新 asset.
- [x] 5.6 补充 UI state tests 或 focused component tests.

## 6. Verification

- [x] 6.1 运行 `openspec validate add-gallery-version-tree --strict`.
- [x] 6.2 运行 focused Rust tests.
- [x] 6.3 运行 desktop tests 或 type check.
- [x] 6.4 运行 `scripts/check-architecture.sh`.
- [x] 6.5 执行 focused manual UI smoke 或记录无法执行的风险. 记录: Vite dev server 可启动, 但当前环境缺少 Playwright module, 未能完成 browser automation screenshot.
