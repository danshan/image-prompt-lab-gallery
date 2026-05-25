## MODIFIED Requirements

### Requirement: Output Album And Tags
系统 SHALL 在 linked image generation task completed 后, 将所有 output assets 加入 job 的 target manual album, 并将 job 中用户指定 tags 写入 canonical asset tags. Post-processing MUST be idempotent, MUST update run output rows, and MUST keep run output counters consistent with persisted post-processing state.

#### Scenario: Completed Task Post Processing
- **WHEN** linked image generation task completed 且包含 output assets
- **THEN** schedule runner 将每个 output asset 加入 target manual album
- **AND** schedule runner 为每个 output asset 应用 job tags as canonical asset tags
- **AND** run outputs 记录 output asset, asset version, generation event, album added state 和 applied tags
- **AND** run output counters reflect output assets, album-added assets 和 tagged assets

#### Scenario: Run Now Uses Same Post Processing
- **WHEN** 用户通过 run-now 创建 scheduled run 且 linked image generation task completed
- **THEN** run-now reconciliation 使用与后台 schedule runner 相同的 output album 和 tag post-processing

#### Scenario: Post Processing Restart
- **WHEN** daemon 在 post-processing 中断后重启并再次 reconcile 同一个 completed linked task
- **THEN** schedule runner 根据 task output links 和 run outputs 继续未完成 output 处理
- **AND** 系统不得创建重复 album membership, duplicate canonical tag relation 或重复 run output row

#### Scenario: Target Album Deleted
- **WHEN** job 的 target manual album 已不存在
- **THEN** 系统将 run 标记为 failed
- **AND** 系统 pause 该 job
- **AND** 系统不得继续创建 image generation task
