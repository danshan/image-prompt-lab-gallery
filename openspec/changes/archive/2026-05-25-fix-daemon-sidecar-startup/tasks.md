## 1. Daemon Binary Discovery

- [x] 1.1 在 desktop daemon binary discovery 中验证 explicit 和 fallback path 必须存在.
- [x] 1.2 为 packaged macOS app 增加 bundle resources daemon binary lookup.
- [x] 1.3 更新 release build hook 和 Tauri external sidecar 配置, 确保 daemon binary 在 packaging 前以 release profile 构建并放入 bundle.

## 2. Automation Daemon Service Control

- [x] 2.1 将 `launchctl` 执行改为 bounded wait, timeout 后返回 recoverable diagnostic.
- [x] 2.2 保持 start, stop, restart, repair 的现有 LaunchAgent label, registry 和 runtime path 语义不变.

## 3. Verification

- [x] 3.1 运行 `openspec validate fix-daemon-sidecar-startup --strict`.
- [x] 3.2 运行 `cargo fmt --all --check`.
- [x] 3.3 运行 `cargo test -p imglab-desktop`.
- [x] 3.4 运行 `git diff --check`.
