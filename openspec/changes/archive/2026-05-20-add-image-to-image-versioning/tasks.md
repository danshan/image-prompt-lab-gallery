## 1. Schema 和 Core Version Model

- [x] 1.1 将 library schema version 递增, 为 `asset_versions` 增加 `version_number`, 并实现旧 library 确定性回填.
- [x] 1.2 为 `(asset_id, version_number)` 增加唯一索引, 并保证 migration 失败时不报告 library 打开成功.
- [x] 1.3 更新 `VersionSummary` 和相关 read model, 暴露 `version_number` 和 `version_name`.
- [x] 1.4 更新 import asset 和 create child version 写入路径, 确保新 asset 从 `v1` 开始, child version 使用同 asset 下一数字版本.
- [x] 1.5 添加 core tests 覆盖 import, child version, migration backfill, future schema 拒绝和稳定引用不被重写.

## 2. Generation Orchestration

- [x] 2.1 更新 generation request preparation, 明确 `input_version_id` 和 `input_file` 的图生图分支.
- [x] 2.2 实现 existing version 图生图输出同 asset child version, 并记录 `parent_version_id`, `input_asset_version_id` 和 `output_version_id`.
- [x] 2.3 实现 uploaded reference 图生图: capability 校验后导入 reference asset `v1`, 输出独立 generated asset `v1`.
- [x] 2.4 实现 uploaded reference 失败语义: capability 或参数失败不导入 reference, provider 失败保留 reference 并记录 failed generation event.
- [x] 2.5 添加 generation tests 覆盖 existing version variation, uploaded reference success, unsupported capability 和 provider failure.

## 3. Read Model, Gallery 和 Lineage

- [x] 3.1 更新 Gallery query 默认排除 `status = reference`, 并保留 source link 可打开 reference asset 的能力.
- [x] 3.2 更新 asset detail 和 Inspector detail, 区分同 asset parent chain 与跨 asset reference source.
- [x] 3.3 更新 Gallery card, version list 和 lineage summary 使用 `vN` 展示, 不默认展示 UUID 派生版本名.
- [x] 3.4 添加 read model tests 覆盖 reference asset 默认不可见, source link 可追溯和数字版本展示.

## 4. CLI, Daemon 和 Desktop Integration

- [x] 4.1 更新 CLI `--input-version` 和 `--input-file` 输出, 包含 output/reference 的 version number 和 version name.
- [x] 4.2 更新 daemon image generation task input/output reconcile, task outputs 链接 output asset/version, generation event, suggestion 和 reference asset/version.
- [x] 4.3 更新 Tauri command DTO mapping, 让 Desktop 收到数字版本和 reference source summary.
- [x] 4.4 更新 React workbench 状态, Gallery, Inspector 和 Generate variation 入口使用数字版本和当前 selected version.
- [x] 4.5 添加 CLI tests, daemon/task tests 和 desktop state/UI tests 覆盖新输出和 reference workflow.

## 5. Verification 和 OpenSpec

- [x] 5.1 运行 `openspec validate add-image-to-image-versioning --strict`, 修复所有 artifact 或 spec 问题.
- [x] 5.2 运行 Rust core 和 CLI 测试, 至少覆盖 `cargo test -p imglab-core` 和 `cargo test -p imglab-cli`.
- [x] 5.3 运行 Desktop 测试, 至少覆盖 `npm test` 或项目现有 workbench state test 命令.
- [x] 5.4 检查 `git diff` 中没有 unrelated refactor, build artifacts 或 generated dependency 目录.

## 6. Follow-up Fixes

- [x] 6.1 修复 Inspector `Generate Version` 入口, 先打开 image-to-image composer 让用户补充 prompt, 再入队当前 version 的新版本生成.
- [x] 6.2 修复 composer 入队失败时仍关闭并跳转 queue 的问题, 只有实际创建 task 后才进入 Tasks Queue.
- [x] 6.3 为 generation composer 增加入队 pending 状态和兜底错误反馈, 避免点击后没有可观察响应.
- [x] 6.4 修复 Desktop dev 模式复用旧 daemon runtime 导致 schema 5/6 mismatch 的问题.
- [x] 6.5 将 core schema version 纳入 daemon health, Desktop 自动拒绝旧 schema daemon 并重启当前版本 sidecar.
- [x] 6.6 修复 Tauri dev/build 前置命令, 确保实际启动的 `target/debug/imglab-daemon` 会随当前代码重建.
- [x] 6.7 修复 Tasks Queue 展示顺序为新到旧, 且不影响 queued task reorder 语义.
- [x] 6.8 修复 Codex CLI provider capability 声明, 让已有 `generate_from_image` 路径可执行 image-to-image.
- [x] 6.9 将 provider capability 纳入 daemon health, Desktop 自动拒绝未声明 `codex-cli image_to_image` 的旧 daemon.
- [x] 6.10 允许 terminal failure 任务手动 retry, 以恢复旧 daemon capability 错误留下的 failed task.
- [x] 6.11 在 asset detail 暴露当前选中 version, 并支持从 Inspector 选择任意 version 查看对应 prompt/file/lineage.
- [x] 6.12 在 version browser 中显示每个 version 的图片缩略图, 并允许从任意 version 发起 Generate variation.
- [x] 6.13 为 Tasks Queue image-to-image draft 增加 reference image picker, 选择文件后写入 `inputFile` 并切换 operation.
- [x] 6.14 在 Inspector Reference Source 中显示 reference 原图缩略图, 并复用 image lightbox 全屏预览.
- [x] 6.15 支持从 Inspector Reference Source 重新打开 image-to-image composer, 以 reference file 作为 `inputFile` 创建新生成任务.
