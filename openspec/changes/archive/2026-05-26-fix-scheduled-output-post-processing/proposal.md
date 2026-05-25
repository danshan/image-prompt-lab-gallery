## Why

通过 scheduled generation 生成的图片在任务完成后没有稳定遵循 job 配置: output asset 没有自动加入目标 album, job tags 也没有写入 canonical asset tags. 这会破坏定时生成的核心整理能力, 也会让 run history 中的 output counters 与实际 library 状态不一致.

## What Changes

- 修复 scheduled generation task 完成后的 output 后处理链路, 确保所有 output assets 都加入 job 的 target manual album.
- 修复 job tags 应用链路, 确保 tags 写入 canonical asset tags, 而不是只停留在 task input 或 run metadata.
- 保持后处理幂等: 重复 reconcile 同一个 completed task 不应创建重复 album membership 或重复 tag relation.
- 增加覆盖真实 schedule runner completion reconcile 的测试, 验证 album membership, canonical tags, run output records 和 counters.
- 不改变 scheduled generation API, resource library schema 或 provider contract.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `scheduled-image-generation`: 加固 scheduled run output 后处理要求, 明确 run-now 和后台 schedule runner 的 completed task reconcile 都必须应用 target album 和 tags, 且结果需要反映在 run outputs 与 counters 中.

## Impact

- 影响 `crates/imglab-daemon` 的 schedule runner / task completion reconcile 实现和测试.
- 可能涉及 `crates/imglab-core` 中 task output link 或 scheduled run output persistence 的最小修正.
- 不涉及 desktop UI layout, Tauri command 形状, SQLite schema migration 或 provider crate 行为变更.
