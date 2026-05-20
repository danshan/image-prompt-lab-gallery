## 1. OpenSpec 和配置准备

- [x] 1.1 验证 `add-macos-release-updater` OpenSpec artifacts.
- [x] 1.2 生成或接收 Tauri updater public key, 并确认 private key 和 password 只进入 GitHub secrets.

## 2. Tauri updater runtime

- [x] 2.1 在 desktop Tauri crate 增加 updater 和 process plugin dependencies.
- [x] 2.2 在 `tauri.conf.json` 启用 updater artifacts, public key 和 GitHub latest release endpoint.
- [x] 2.3 在 Tauri builder 注册 updater 和 process plugin.
- [x] 2.4 增加 desktop update commands, 覆盖检查更新, 安装 pending update 和重启.

## 3. Settings App Updates UI

- [x] 3.1 增加 frontend update state 和启动静默检查 orchestration.
- [x] 3.2 在 Settings 中增加 App Updates 区域, 展示当前版本, 最近检查时间, 更新状态和错误状态.
- [x] 3.3 接入手动 `Check for Updates`, `Download and Install` 和 `Restart` 操作.
- [x] 3.4 确保更新失败不阻塞主工作流, 且可重试.

## 4. GitHub release workflow 和文档

- [x] 4.1 新增 macOS-only `.github/workflows/release.yml`, 支持 `v*` tag 和 manual dispatch.
- [x] 4.2 配置 release workflow 使用 Tauri updater signing secrets, 不依赖 Apple secrets.
- [x] 4.3 新增 release 文档, 覆盖 key 生成, GitHub secrets, version bump, tag release, assets 检查和 Gatekeeper 边界.

## 5. 验证

- [x] 5.1 运行 `openspec validate add-macos-release-updater`.
- [x] 5.2 运行 desktop frontend build 或相关 TypeScript 检查.
- [x] 5.3 运行 Rust desktop compile check.
- [x] 5.4 在可用环境下运行 macOS Tauri build, 或记录无法本地完成的原因.
- [x] 5.5 更新 tasks completion 状态.
