## Context

Task attempt logs 是 runtime-owned 文件, 但 task attempts 持久化在 resource library SQLite 中. 当用户在不同 daemon runtime 之间切换, 或旧 sidecar / background daemon 写入的 log root 不再是当前 daemon 的 `log_root` 时, attempt 中保存的 `log_path` 可能指向当前 daemon app-owned root 之外.

当前 daemon 的 `/v1/tasks/{id}/logs/tail` 会 canonicalize 当前 log root 和 attempt log path, 并拒绝读取 root 之外的路径. 这是正确的安全边界. 问题出在 Desktop `get_daemon_task_detail`: 它将 task detail 和 log tail 作为一个不可分割操作处理, 导致 log tail 失败时整个 detail 失败.

## Goals / Non-Goals

**Goals:**

- 保留 daemon 对非 app-owned log path 的拒绝语义.
- 让 Desktop Task Detail 在 log tail stale, missing 或不可读时仍能展示 task identity, status, attempts, timeline 和 outputs.
- 给用户展示可恢复的日志不可用信息, 避免把日志问题误报为 generation 参数无效.
- 覆盖 Tauri command 层行为, 防止回归.

**Non-Goals:**

- 不迁移历史 attempt log path.
- 不改变 daemon HTTP API response schema.
- 不改变 task persistence schema.
- 不扩大 Settings Logs 或 daemon log tail 的可读路径范围.

## Decisions

1. 在 Desktop command 层做 best-effort log tail.

   `get_daemon_task_detail` 继续先请求 `client.get_task`, 确保结构化 task detail 可用. 随后请求 `client.tail_task_log`; 如果失败属于可恢复 log-tail 错误, command 返回 detail 并填入 unavailable log message. 这样不会改变 daemon API 的安全契约, 也避免 task detail UI 因日志不可用整体失败.

   Alternative considered: 在 daemon `tail_task_log` 中吞掉 unowned path 并返回 empty content. 这会改变 daemon API contract, 也削弱现有 “拒绝非 task-owned arbitrary paths” 的测试语义, 因此不采用.

2. 降级范围限定为 log tail 获取失败.

   `client.get_task` 失败仍应返回错误, 因为这表示 task 本身无法定位或 daemon 不可用. 只有 `tail_task_log` 失败会被转换为 detail 内的 log unavailable message.

3. 不解析具体错误字符串作为业务核心契约.

   实现可以优先基于 `CommandError.recoverable` 或 daemon API error code 识别可恢复失败. 如当前 error mapping 不足以区分, 应新增小型 helper 将 tail failure 转为日志消息, 但不得影响非 tail API.

## Risks / Trade-offs

- [Risk] 用户可能看不到旧 attempt raw log 内容. → Mitigation: detail 中保留 attempts, timeline, last error 和 outputs; log 区域显示明确不可用原因.
- [Risk] 过宽地吞掉 daemon 错误会掩盖真正故障. → Mitigation: 只在已经成功取得 task detail 后, 对 log tail 单独降级; task detail 读取失败不降级.
- [Risk] 错误消息依赖 daemon 文案可能脆弱. → Mitigation: 测试覆盖 command 行为, 并保持 daemon 拒绝 unowned path 的既有测试.

## Migration Plan

无需数据迁移. 发布后, 已存在的历史 tasks 在打开 detail 时会继续显示结构化信息; 旧日志仍不会被当前 daemon 读取.

Rollback 方式是恢复 Desktop command 的 fail-fast 行为; 不涉及 schema rollback.

## Open Questions

无.
