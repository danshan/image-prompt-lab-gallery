## Context

当前 task queue 已经有 `TaskSchedulerConfig`, 其中包含全局并发上限和 provider-level 并发上限. 但 daemon 启动时只使用默认配置, Settings 也没有读取或修改该配置的入口. 更关键的是, 现有 scheduler loop 一次 tick 只选择并执行一个 task, 且执行过程同步占用 scheduler worker, 因此即使全局配置大于 `1`, 实际吞吐也不会形成真正的并发执行.

这个配置属于本机 daemon runtime policy, 不属于 resource library 的业务事实. 因此它不应写入 `library.sqlite`, 也不应影响 asset lineage, task history, generation output persistence 或 metadata review canonical write 语义.

## Goals / Non-Goals

**Goals:**

- 在 Settings 中提供 `Task Queue` section, 允许用户查看和修改最大并发执行数.
- 将最大并发数作为 app-level daemon setting 持久化, daemon 重启后仍能恢复.
- 让 scheduler 在同一轮调度中启动多个 eligible queued tasks, 直到达到全局并发上限或 provider safety limit.
- 保持 task state transition, retry classification, output link 和 attempt/event persistence 的现有 ownership.
- 保持 daemon health, capabilities 和 task read API 在长任务期间可响应.

**Non-Goals:**

- 不引入 per-library 并发配置.
- 不提供 provider-specific 并发 UI.
- 不改变 resource library schema, registry schema 或 backup/restore 格式.
- 不改变 queue priority, queue position, retry policy 或 cancellation marker 语义.
- 不承诺所有 provider 都能按全局上限并发执行; provider-level safety limit 仍可限制实际并发.

## Decisions

### 1. 使用 app-level daemon settings, 不写入 resource library

最大并发数表达的是本机执行能力和用户对当前机器资源的偏好, 与 library 可迁移内容无关. 实现时应使用 app-level settings 文件或等价本机配置存储, 由 Tauri desktop 启动 sidecar daemon 时传入或由 daemon 根据 app config path 读取. Daemon HTTP API 返回 effective value 和 validation bounds.

备选方案是写入每个 resource library. 这会让同一个 library 在不同机器上携带本机资源偏好, 也会引入 schema migration 和 backup compatibility 成本, 不符合该配置的 ownership.

### 2. Settings 只暴露全局最大并发数

UI 提供一个紧凑 numeric input 或 stepper: `Max parallel tasks`, 默认值沿用现有全局默认值. 文案必须说明 provider safety limits may reduce actual parallelism, 避免用户把全局上限理解为强制所有 provider 同时执行.

备选方案是同时暴露 provider-specific limits. 这会增加设置复杂度, 且当前 provider 能力模型还没有稳定的用户可见 concurrency contract, 本次先不做.

### 3. Scheduler 改为批量领取和 worker 并发执行

Scheduler loop 应将 "选择可运行 task" 与 "执行 provider request" 分离. 每轮 loop 读取 opened libraries 的 task snapshot, 根据当前 running count, configured global limit, provider limits, priority 和 queue position 选出最多 `available_global_slots` 个 task. 每个被领取的 task 必须先通过 task owner 原子地从 queued 转为 running 并写入 attempt started event, 然后在独立 worker 中执行 provider body 和 completion/failure/cancel finalization.

备选方案是保持单 tick 单任务, 依赖更短 tick interval. 这无法解决长任务同步占用 scheduler 的问题, 也无法真正支持多个 running tasks.

### 4. 保持 provider-level safety limit

全局最大并发数控制 daemon 同时运行的 task 总数. Provider-level limit 继续作为下限约束, 例如某个 provider 默认只允许一个 active worker 时, 即使全局上限更高, 同 provider queued tasks 也会继续获得 provider slot wait reason. 这样保留当前 spec 中的 provider concurrency 语义, 并降低外部 CLI 或 provider rate limit 风险.

### 5. 配置更新只影响后续调度, 不抢占 running tasks

当用户降低最大并发数时, daemon 不取消已经 running 的 task. 新值从下一轮 scheduler decision 开始生效; 如果当前 running count 已超过新上限, scheduler 不再启动新 task, 直到 running count 低于上限. 这样避免设置操作变成破坏性任务控制.

## Risks / Trade-offs

- 并发 worker 同时写同一 resource library SQLite 可能增加 write contention -> 复用现有 task owner 和 SQLite transaction 边界, 增加 daemon scheduler tests 覆盖多任务并发完成和 wait reason.
- 用户把全局上限理解为所有任务都会并发 -> Settings UI 展示 effective running/limit 说明, 并在 provider slot wait reason 中保留 provider 限制原因.
- 降低并发数后 running count 短时间高于新值 -> 明确不抢占 running tasks, 只阻止新任务启动.
- app-level settings 文件损坏或缺失 -> daemon fallback 到默认配置, Settings 展示可恢复错误并允许重新保存合法值.
- 多 scheduler tick 重复领取同一 queued task -> task claiming 必须在持久化层通过 status 条件或 task owner 原子校验完成, 失败时跳过该 task.

## Migration Plan

1. 新增 app-level daemon settings 读写逻辑, 缺省时使用 `TaskSchedulerConfig::default()`.
2. 新增 daemon settings read/update route, 并在 route 层做 numeric bounds validation.
3. 改造 scheduler loop 为批量领取和 worker 并发执行, 保持 task owner 决策入口.
4. 新增 Tauri command 和 frontend state/action, 在 Settings / Task Queue 中读取和保存配置.
5. 增加 Rust scheduler/config route tests 和 frontend settings state/render tests.

Rollback 时删除 Settings 入口并让 daemon 回到默认 `TaskSchedulerConfig`; 已经持久化的 app-level setting 可以忽略, 不影响 library 数据.

## Open Questions

- 最大并发数的 hard upper bound 建议先设为 `8`, 以控制本地 provider process 和 SQLite write contention 风险; 如果后续 provider capability 暴露更精细的限制, 再扩展 provider-specific setting.
