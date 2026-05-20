# macOS Release And Updater Design

## Purpose

本设计为 Image Prompt Lab desktop 增加 macOS-only release 和 auto update 能力. 目标是让没有 Apple Developer certificate 的情况下也能完成 GitHub release 打包, 并让已安装 app 可以通过 Tauri updater 从 GitHub Release 检查, 下载和安装新版本.

本设计选择 ad-hoc signing, 不做 Apple Developer ID signing, 不做 notarization. 因此它不能强保证互联网下载后的首次打开完全免 Gatekeeper 拦截. 如果未来需要 “首次打开无需 `xattr -cr` 或其他绕过动作” 的正式分发体验, 需要切换到 Developer ID signing 和 notarization.

## Scope

In scope:

- macOS-only GitHub Actions release workflow.
- `v*` tag 自动触发 release, 并保留 manual dispatch.
- Tauri updater artifact 生成和 GitHub latest release endpoint.
- Tauri updater signing key 管理.
- Desktop app 启动时静默检查更新.
- Settings 中提供手动检查, 下载, 安装和重启入口.
- 文档化 release secrets, version bump 和验证步骤.

Out of scope:

- Apple Developer ID certificate, notarization 和 Apple API key 管理.
- Windows 和 Linux release.
- App Store 发布.
- 全自动后台下载安装.
- 替换现有 resource library, provider 或 core domain 架构.

## Release Architecture

Release pipeline 使用 GitHub Actions 的 macOS runner. Workflow 在两种场景下运行:

- Push `v*` tag 时自动发布.
- `workflow_dispatch` 手动触发, 用于测试或补发.

Workflow 运行在 `macos-latest`, 安装 Node 和 Rust, 在 `apps/desktop` 执行 Tauri build. 初始实现优先支持 Apple Silicon target `aarch64-apple-darwin`. 是否同时加入 Intel target `x86_64-apple-darwin` 是 implementation plan 的矩阵细节, 不改变设计边界.

Release workflow 使用 ad-hoc codesign, 不依赖 Apple certificate secrets. 它应上传 Tauri 生成的 macOS installer 或 bundle assets, updater artifacts, signature 文件和 `latest.json`. 正式 tag release 默认创建非 draft GitHub Release, 因为 updater endpoint 需要稳定读取 latest release. 手动 dispatch 可以保留 draft input, 但 draft release 不作为自动更新来源.

## Signing Model

本设计包含两个不同的签名概念:

1. macOS ad-hoc signing. 目标是让 bundle 内部签名结构完整, 并让没有 Apple certificate 的构建仍能完成. 它不等价于 notarization, 不能让 Gatekeeper 信任互联网下载的 app.
2. Tauri updater signing. 目标是让已安装 app 验证更新包由项目发布. Tauri updater 要求签名校验, 这个要求不能关闭, 也不应绕过.

Tauri updater key 由开发者本地生成:

```text
cd apps/desktop
npm run tauri signer generate -- -w ~/.tauri/image-prompt-lab.key
```

生成时输入的 password 用于加密 private key. Public key 写入 `apps/desktop/src-tauri/tauri.conf.json` 的 updater plugin 配置. Private key 文件内容进入 GitHub secret `TAURI_SIGNING_PRIVATE_KEY`. Password 进入 GitHub secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Private key 不提交到仓库.

## Updater Architecture

Tauri 配置启用 updater artifact:

- `bundle.createUpdaterArtifacts = true`.
- `plugins.updater.pubkey` 使用 signer 生成的 public key 内容.
- `plugins.updater.endpoints` 指向 GitHub latest release 中的 `latest.json`.

Endpoint 形态:

```text
https://github.com/danshan/image-prompt-lab-gallery/releases/latest/download/latest.json
```

如果仓库未来迁移或 fork 成新的发布源, endpoint 必须随发布仓库一起修改. 已安装 app 会按内置 endpoint 检查更新, 因此发布源迁移需要作为单独兼容性变更处理.

Desktop runtime 注册 Tauri updater plugin 和 process plugin. Process plugin 用于安装完成后的 relaunch 或 restart. Updater 属于 desktop shell concern, 不进入 `imglab-core` domain, 不改变 resource library service boundary.

Rust side 可以封装少量 Tauri commands:

- Check whether an update is available.
- Download and install the pending update.
- Restart the app after installation.

使用 Rust command 封装而不是让 React 直接调用 JavaScript plugin 的原因是当前 desktop 已经以 Tauri command boundary 为主, 错误映射, app logs 和恢复行为更容易统一.

## Product Behavior

App 启动后进行一次静默检查. 静默检查不弹 blocking modal, 不自动下载安装. 如果有新版本, app 在 Settings 或全局 update indicator 中记录可用状态.

Settings 增加 `App Updates` section. 初始位置可以放在 Settings 的 diagnostics/logs 相关区域, 后续如果 Settings IA 继续扩展, 可以拆为独立子页. 该 section 提供:

- Current version.
- Last checked time.
- `Check for Updates`.
- Available version, release date 和 release notes.
- `Download and Install`.
- Installed update pending restart state.
- `Restart`.

失败处理必须可恢复:

- Network failure 显示检查失败, 保留手动重试.
- Signature failure 显示更新不可安装, 不降级到不安全安装.
- Download failure 显示可重试.
- Install failure 显示错误并保持 app 可用.

## Version And Release Discipline

Release tag, `apps/desktop/package.json` version 和 `apps/desktop/src-tauri/tauri.conf.json` version 必须保持一致. 初始实现可以用人工流程维护一致性, release 文档必须要求:

1. Bump desktop version.
2. Run local build checks.
3. Commit version bump.
4. Create and push `vX.Y.Z` tag.
5. Confirm GitHub Release assets include updater metadata and signatures.

如果后续版本发布频率提高, 可以再增加脚本校验 tag 和 config version 一致性.

## GitHub Actions Design

新增 `.github/workflows/release.yml`. Workflow 需要:

- `permissions.contents = write`.
- Checkout repository.
- Setup Node with npm cache for `apps/desktop/package-lock.json`.
- Setup Rust stable with macOS target.
- Install frontend dependencies in `apps/desktop`.
- Build desktop app through `tauri-apps/tauri-action@v0`.
- Pass `GITHUB_TOKEN`.
- Pass `TAURI_SIGNING_PRIVATE_KEY`.
- Pass `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Configure release name and body from version.

Workflow 不需要 Apple secrets:

- No `APPLE_ID`.
- No `APPLE_PASSWORD`.
- No `APPLE_CERTIFICATE`.
- No notarization API key.

## Verification

Local verification:

- `npm run build` in `apps/desktop`.
- `cargo check -p imglab-desktop`.
- `npm run tauri build -- --target aarch64-apple-darwin` on macOS.

CI verification:

- Existing CI continues to run Rust and desktop frontend checks.
- Release workflow is manually triggered once before first tag release.
- Release assets are inspected for macOS installer or bundle, updater artifacts, signature and `latest.json`.

End-to-end updater verification:

1. Install version `vX.Y.Z`.
2. Publish version `vX.Y.Z+1`.
3. Open installed old app.
4. Confirm startup check records update availability without blocking.
5. Use Settings `Check for Updates`.
6. Download and install update.
7. Restart app.
8. Confirm app reports the new version.

Gatekeeper verification is separate. With ad-hoc signing, the expected result is that build and updater artifacts are produced and signed for Tauri updater verification. The expected result is not notarized first-open behavior.

## Risks

- Users may still see macOS Gatekeeper warnings for downloaded builds. Mitigation: document ad-hoc signing limitation and leave Developer ID signing as a future release hardening step.
- Losing the Tauri updater private key prevents publishing updates to existing installed apps. Mitigation: store the private key in GitHub secrets and a password manager backup.
- Public key changes break update compatibility for already-installed apps. Mitigation: treat updater key rotation as a dedicated migration, not a routine config edit.
- Draft releases are invisible to latest-release based updater flow. Mitigation: only non-draft releases are considered production update sources.
- Version mismatch can create confusing release assets. Mitigation: document version discipline first, add automated version check later if needed.

## Implementation Order

1. Generate updater signing key and record the public key.
2. Add updater and process plugin dependencies.
3. Configure `tauri.conf.json` updater settings.
4. Register plugins in desktop bootstrap.
5. Add update commands and frontend Settings UI.
6. Add release workflow.
7. Add release documentation.
8. Verify local build, manual release workflow and installed-app update path.
