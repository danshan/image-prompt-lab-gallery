## ADDED Requirements

### Requirement: 桌面应用支持 App Updates 管理

桌面应用 SHALL 在 Settings 中提供 App Updates 管理区域, 展示当前版本, 最近检查时间, 更新检查状态和可用更新信息. App Updates 管理 SHALL 支持手动检查更新, 下载并安装更新, 以及在安装完成后重启应用.

#### Scenario: 查看当前更新状态

- **WHEN** 用户打开 Settings 中的 App Updates 区域
- **THEN** 桌面应用展示当前 app version, 最近检查时间和当前更新状态

#### Scenario: 手动检查更新

- **WHEN** 用户点击 `Check for Updates`
- **THEN** 桌面应用通过 Tauri updater 检查 GitHub release endpoint, 并展示无更新, 有更新或检查失败状态

#### Scenario: 展示可用更新

- **WHEN** Tauri updater 返回可用更新
- **THEN** 桌面应用展示目标版本, release notes 或等价说明, 并提供 `Download and Install` 操作

#### Scenario: 安装更新并重启

- **WHEN** 用户下载并安装更新成功
- **THEN** 桌面应用展示 pending restart 状态, 并提供 `Restart` 操作以启动新版本

#### Scenario: 更新失败可恢复

- **WHEN** 更新检查, 签名验证, 下载或安装失败
- **THEN** 桌面应用展示可恢复错误, 保持主工作流可用, 并允许用户稍后重试

### Requirement: 桌面应用启动时静默检查更新

桌面应用 SHALL 在启动后静默检查一次更新. 静默检查 MUST NOT 自动下载或安装更新, MUST NOT 弹出阻塞式 modal, 并 MUST 将结果映射到 Settings App Updates 状态.

#### Scenario: 启动时发现新版本

- **WHEN** 用户启动已安装 app 且 GitHub latest release 存在更高版本
- **THEN** 桌面应用记录 update available 状态, 不阻塞当前工作流, 并允许用户稍后在 Settings 中安装

#### Scenario: 启动时检查失败

- **WHEN** 启动静默检查因网络或 endpoint 错误失败
- **THEN** 桌面应用不阻塞启动, 保持主工作流可用, 并在 App Updates 状态中保留可重试错误

### Requirement: macOS 发布包支持 GitHub Release 自动更新

桌面应用 release build SHALL 生成 Tauri updater artifacts, 并 SHALL 使用 Tauri updater signing key 对更新包签名. 已安装 app SHALL 通过内置 public key 验证更新包, 并从 GitHub latest release 的 `latest.json` endpoint 获取更新信息.

#### Scenario: Release 包包含 updater artifacts

- **WHEN** macOS release workflow 成功完成
- **THEN** GitHub Release assets 包含 macOS bundle 或 installer, Tauri updater artifact, signature 和 `latest.json`

#### Scenario: 已安装 app 验证更新签名

- **WHEN** 已安装 app 检查到 GitHub Release 中的新版本
- **THEN** Tauri updater 使用内置 public key 验证更新包签名, 并只允许安装验证通过的更新

#### Scenario: 签名验证失败

- **WHEN** 更新包签名缺失或与内置 public key 不匹配
- **THEN** 桌面应用拒绝安装该更新, 显示可恢复错误, 且不得降级为不安全安装

### Requirement: macOS Release Workflow 不依赖 Apple Developer 证书

项目 SHALL 提供 macOS-only GitHub Actions release workflow. Workflow SHALL 支持 `v*` tag 自动触发和 manual dispatch, SHALL 使用 ad-hoc signing 完成无 Apple Developer certificate 的 macOS package build, 并 MUST NOT 要求 Apple Developer ID, notarization 或 Apple certificate secrets.

#### Scenario: Tag 触发 macOS release

- **WHEN** 开发者 push `v*` tag
- **THEN** GitHub Actions 运行 macOS release workflow, 构建 desktop app, 并创建或更新对应 GitHub Release assets

#### Scenario: Manual dispatch 触发 macOS release

- **WHEN** 开发者手动触发 release workflow
- **THEN** GitHub Actions 运行 macOS release workflow, 并按 workflow input 创建测试或正式 release assets

#### Scenario: 无 Apple secrets 也能构建

- **WHEN** GitHub repository 只配置 Tauri updater signing secrets, 没有 Apple certificate 或 notarization secrets
- **THEN** macOS release workflow 仍能执行 ad-hoc signed build 和 updater artifact 生成

#### Scenario: Ad-hoc signing 边界明确

- **WHEN** 开发者或用户查看 release 文档
- **THEN** 文档明确 ad-hoc signing 不等价于 Developer ID signing 或 notarization, 且不承诺首次打开完全免 Gatekeeper 限制
