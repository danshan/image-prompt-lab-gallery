## Why

当前 Task Detail 在读取 task 基础信息后会继续请求 daemon log tail. 如果该 task 的历史 attempt `log_path` 属于旧 sidecar 或 background daemon runtime, 当前 daemon 会按安全策略拒绝读取, Desktop 因此把整个 task detail 判为失败并显示 `invalid generation parameters: task log path is outside app-owned log root`.

这个错误会阻断用户查看 task 状态, timeline, attempts 和 outputs, 但真正不可用的只有旧日志内容. 需要在不放宽日志读取安全边界的前提下, 让 Task Detail 对 stale log path 做可恢复降级.

## What Changes

- Desktop 获取 task detail 时, SHALL 在 log tail 不可用或 stale 时仍展示 task detail 的结构化信息.
- Desktop SHALL 将 stale/unowned log tail 显示为可读的 unavailable message, 而不是让整个 detail 加载失败.
- Daemon log tail API SHALL 继续拒绝读取非当前 app-owned task log root 下的路径, 不返回任意文件内容.
- 不改变 task, attempt, event, output link 的持久化 schema.
- 不迁移, 删除或移动历史 attempt log 文件.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `task-manager-daemon`: 明确 task detail 的结构化信息不应因历史 attempt log path 越界而不可用.
- `app-logs`: 明确 Task Detail 对不可读取的 app-owned 或 stale task log 使用降级展示, 同时保持任意路径读取拒绝.

## Impact

- Affected code:
  - `apps/desktop/src-tauri/src/commands/daemon.rs`
  - `apps/desktop/src-tauri/src/daemon_client.rs` tests or related command tests
  - `crates/imglab-daemon/src/transport.rs` tests only if daemon contract documentation needs extra coverage
- APIs:
  - No public JSON schema changes.
  - Desktop command `get_daemon_task_detail` behavior changes from fail-fast on stale log tail to best-effort log tail.
- Dependencies:
  - No new runtime dependency.
- Systems:
  - Task Queue detail view becomes resilient to daemon runtime root changes.
  - Log file security boundary remains enforced by daemon and desktop log APIs.
