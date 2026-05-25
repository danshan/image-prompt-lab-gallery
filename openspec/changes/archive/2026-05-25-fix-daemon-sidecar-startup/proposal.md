## Why

当前 desktop 在 daemon binary 不存在时会尝试启动一个不存在的 sidecar, 最终只暴露 `os error 2`, 用户无法判断是构建缺失还是 bundle 缺失. Settings 中的 Automation daemon `Restart` / `Repair` 还会同步等待 `launchctl`, 一旦系统服务命令卡住, 整个 Tauri command 会长期阻塞.

该问题直接影响 Generate Queue, Review regeneration 和 Settings Automation 的可用性, 需要在 daemon lifecycle boundary 内修复.

## What Changes

- 在解析 daemon binary 时验证候选路径必须存在, 缺失时返回 recoverable 且可操作的错误信息.
- 支持 packaged app 从 Tauri external sidecar 资源位置定位 `imglab-daemon`, 并确保 release build 将 daemon binary 放入 bundle.
- 为 Automation daemon 的 `launchctl` 操作增加 bounded timeout, 避免 `Restart` / `Repair` 无限阻塞 UI.
- 保持已有 background daemon 优先, app-owned sidecar fallback, runtime file discovery 和 token-authenticated loopback API 语义不变.

## Capabilities

### New Capabilities

无.

### Modified Capabilities

- `task-manager-daemon`: 明确 desktop daemon startup 在 daemon binary 缺失和 service command 卡住时的 recoverable failure 行为.

## Impact

- `apps/desktop/src-tauri/src/paths.rs`: daemon binary discovery 和 missing binary error.
- `apps/desktop/src-tauri/src/automation_daemon.rs`: `launchctl` command execution timeout.
- `apps/desktop/src-tauri/tauri.conf.json`: release build daemon binary profile 和 external sidecar 配置.
- 验证范围: Tauri backend unit tests, desktop build config validation, daemon sidecar smoke path.
