## Why

当前桌面端只有 CI 构建, 没有可复用的 GitHub release 打包流程, 也没有已安装 app 从 GitHub Release 自动检查和安装新版本的能力. 这使 macOS 分发依赖手工步骤, 且每次升级都需要用户重新下载和替换 app.

本变更以 macOS-only, ad-hoc signing 为第一阶段目标: 在没有 Apple Developer certificate 的情况下完成打包和发布, 并通过 Tauri updater signing 保障自动更新包来源可信.

## What Changes

- 新增 macOS-only GitHub Actions release workflow, 支持 `v*` tag 自动发布和 manual dispatch.
- Desktop Tauri 配置启用 updater artifacts, GitHub latest release endpoint 和 updater public key.
- Desktop runtime 注册 Tauri updater 和 process plugin.
- Settings 增加 App Updates 区域, 支持启动静默检查, 手动检查, 下载, 安装和重启.
- 新增 release 文档, 覆盖 updater key 生成, GitHub secrets, version bump, tag release 和验证步骤.
- 明确 ad-hoc signing 不是 Developer ID signing 或 notarization, 不承诺首次打开完全免 Gatekeeper 限制.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `desktop-workbench`: Settings 和 desktop shell 增加 app update 检查, 安装和可恢复错误状态要求.

## Impact

- `apps/desktop/src-tauri/tauri.conf.json`: 增加 updater artifact 和 plugin 配置.
- `apps/desktop/src-tauri/Cargo.toml`: 增加 Tauri updater 和 process plugin dependencies.
- `apps/desktop/src-tauri/src/lib.rs`: 注册 plugins, 增加 updater commands 或等价 desktop command boundary.
- `apps/desktop/src`: Settings UI, update state 和启动检查 orchestration.
- `.github/workflows/release.yml`: 新增 macOS release workflow.
- `docs/`: 新增 release 和 updater 操作文档.
- GitHub repository secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
