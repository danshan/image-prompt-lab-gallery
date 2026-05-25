## Why

当前 Task Queue 的调度配置只在 daemon 进程内使用默认值, Settings 里没有用户可控入口. 对于批量 image generation, metadata regeneration 或 schedule run, 单任务串行执行会让本地队列吞吐受限, 也无法根据机器资源和 provider 限制调整执行强度.

## What Changes

- 在 Settings 中新增 `Task Queue` 配置 section, 展示并允许修改 daemon task queue 的最大并发执行数.
- 将最大并发数作为 daemon scheduler runtime policy 管理, 默认保持现有保守行为, 用户修改后影响后续 scheduler tick.
- 让 daemon scheduler 在全局并发 slot 可用时允许多个 queued tasks 同时进入 running, 并继续保留 provider-level concurrency limit, queue priority 和 queue position 规则.
- 为无效配置值提供 validation 和可恢复错误, 避免设置为 `0`, 非数字或超过安全上限的值.
- 不改变 resource library SQLite schema, asset lineage, task persistence model 或 provider output persistence contract.

## Capabilities

### New Capabilities
- 无.

### Modified Capabilities
- `task-manager-daemon`: task scheduler 必须支持可配置的全局最大并发数, 并在配置允许时启动多个可执行 task.
- `desktop-workbench`: Settings workflow 必须提供 `Task Queue` section, 用于查看和修改最大并发执行数.

## Impact

- Rust core: `TaskSchedulerConfig` evaluation tests may need expansion, but business task status ownership remains unchanged.
- Daemon: runtime config storage, local HTTP routes, scheduler loop iteration, validation, tests.
- Tauri desktop backend: daemon client command/view mapping for reading and updating queue settings.
- Desktop frontend: Settings section navigation, compact settings form, locale text, controller state/actions.
- OpenSpec specs: `task-manager-daemon` and `desktop-workbench` delta specs.
