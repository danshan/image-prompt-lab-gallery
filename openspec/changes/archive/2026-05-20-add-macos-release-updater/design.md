## Context

桌面端当前是 Tauri 2 + React + Rust workspace. 现有 `.github/workflows/ci.yml` 只验证 Rust 和 desktop frontend build, 没有 release workflow. `apps/desktop/src-tauri/tauri.conf.json` 当前只启用基础 bundle 配置, 没有 updater artifact, updater endpoint 或 updater public key.

用户明确选择 macOS-only, tag + manual dispatch, ad-hoc signing, 启动时静默检查加 Settings 手动检查. 该选择的核心约束是: 不依赖 Apple Developer certificate 完成打包, 但也接受 ad-hoc signing 不能提供 notarization 级别的 Gatekeeper 保证.

本变更基于已确认的设计文档 `docs/superpowers/specs/2026-05-20-macos-release-updater-design.md`.

## Goals / Non-Goals

**Goals:**

- 通过 GitHub Actions 产出 macOS release assets 和 Tauri updater artifacts.
- 已安装 app 能通过 GitHub latest release endpoint 检查新版本.
- 更新包必须通过 Tauri updater signature 验证.
- Settings 提供用户可控的检查, 下载, 安装和重启入口.
- 发布流程文档化, 包括 updater key 生成, GitHub secrets 和版本发布纪律.

**Non-Goals:**

- 不实现 Apple Developer ID signing.
- 不实现 notarization.
- 不承诺首次打开完全免 Gatekeeper 限制.
- 不支持 Windows 或 Linux release.
- 不把 updater 状态写入 resource library 或 core domain.
- 不实现后台自动下载和安装.

## Decisions

### 1. 使用 ad-hoc signing 作为 macOS 第一阶段发布策略

选择 ad-hoc signing 是为了让 release workflow 在没有 Apple Developer certificate 的情况下可运行. 它让 bundle 内部签名结构更完整, 但不等价于 Developer ID signing 或 notarization.

替代方案是正式 Developer ID signing + notarization. 该方案能提供更好的首次打开体验, 但需要 Apple Developer Program, certificate, notarization credentials 和 GitHub secrets. 这与当前约束不匹配, 留作未来 release hardening.

### 2. 使用 Tauri updater signing key 保护更新链路

Tauri updater 要求更新包签名验证, 该要求不能关闭. 本变更使用 Tauri CLI 生成 updater private/public key pair. Public key 写入 `tauri.conf.json`; private key 和 password 只放在 GitHub secrets 和开发者密码管理器中.

该 key 只保护 updater artifacts, 不替代 macOS code signing. 这两个签名概念必须在文档和错误处理里分开表达.

### 3. Updater 属于 desktop shell concern

Updater 状态不进入 `imglab-core`, 不改变 resource library schema, 不改变 generation, metadata 或 albums 领域模型. Desktop Tauri layer 负责调用 updater plugin, 前端 Settings 负责展示状态和触发动作.

React 可以直接使用 Tauri JavaScript updater plugin, 但当前应用已经以 Tauri command boundary 为主. 因此实现优先封装 Rust commands, 以复用现有错误映射和 app logs 习惯.

### 4. GitHub latest release 是唯一生产更新源

Updater endpoint 使用:

```text
https://github.com/danshan/image-prompt-lab-gallery/releases/latest/download/latest.json
```

这要求生产更新必须发布为非 draft GitHub Release. Manual dispatch 可以用于测试, 但 draft release 不作为已安装 app 的更新源. 如果未来迁移发布仓库, 需要单独兼容性设计, 因为已安装 app 会继续读取内置 endpoint.

### 5. 启动静默检查, 安装由用户触发

启动时只检查更新并记录 UI 状态, 不弹 blocking modal, 不自动下载或安装. Settings 提供显式 `Check for Updates`, `Download and Install`, `Restart` 流程. 该策略对本地生产工具更可控, 失败也不会阻塞主工作流.

## Risks / Trade-offs

- [Risk] 用户仍可能遇到 macOS Gatekeeper 警告. → Mitigation: 文档明确 ad-hoc signing 边界, 后续需要正式分发体验时单独引入 Developer ID signing 和 notarization.
- [Risk] Tauri updater private key 丢失后, 已安装 app 无法继续信任新更新. → Mitigation: private key 存入 GitHub secrets 和密码管理器备份, 不提交仓库.
- [Risk] Public key 变更会破坏已安装 app 的更新兼容性. → Mitigation: key rotation 视为专门迁移, 不作为常规配置调整.
- [Risk] version, tag 和 Tauri config 不一致会导致 release 资产混乱. → Mitigation: 初期用文档化发布纪律约束, 后续可增加脚本校验.
- [Risk] GitHub latest release endpoint 无法覆盖 draft 测试 release. → Mitigation: 测试 release 用 manual dispatch 和资产检查, 生产更新只依赖非 draft release.

## Migration Plan

1. 生成 Tauri updater key pair, 将 public key 写入 Tauri config, 将 private key 和 password 写入 GitHub secrets.
2. 合入 updater plugin, process plugin, Tauri config 和 Settings UI.
3. 合入 release workflow 和 release 文档.
4. 本地验证 desktop build 和 Rust compile.
5. 手动触发一次 release workflow, 检查 assets 和 updater metadata.
6. 发布两个连续版本, 用已安装旧版本验证检查, 下载, 安装和重启.

Rollback strategy:

- 如果 updater runtime 有问题, 可以从 Settings UI 隐藏 update section, 保留 release workflow.
- 如果 release workflow 有问题, 可以禁用 tag trigger 或删除 workflow, 不影响本地 app 功能.
- 已发布 app 的 updater public key 不应随意更换; 如果必须更换, 需要先发布兼容迁移版本.

## Open Questions

- 初始 release 是否同时构建 Intel target `x86_64-apple-darwin`. 当前设计允许后续在 release workflow matrix 中添加, 初始实现优先 Apple Silicon.
- Updater check 状态最终放在 Settings Logs 页还是独立 Settings section. 当前实现优先放在 Settings 中的 `App Updates` section, 后续可随 Settings IA 调整.
