## 1. Desktop Command Resilience

- [x] 1.1 在 `get_daemon_task_detail` 中将 log tail 请求改为 best-effort, 保留 task detail 成功路径.
- [x] 1.2 为 log tail 失败生成清晰的 unavailable message, 避免向用户展示 generation parameters 误导文案.

## 2. Verification

- [x] 2.1 添加或更新测试, 覆盖 task detail 成功但 log tail 因 stale/unowned path 失败时仍返回 detail.
- [x] 2.2 运行 OpenSpec strict validate 和相关 Rust 测试.
