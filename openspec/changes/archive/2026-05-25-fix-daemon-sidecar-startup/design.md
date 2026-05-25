## Context

Desktop daemon lifecycle 当前有两个入口: 普通 Queue / Review workflow 会通过 `ensure_daemon_client` 复用 background daemon 或启动 app-owned sidecar; Settings Automation 会通过 macOS LaunchAgent 安装, 重启或修复 background daemon. 两条路径都依赖同一个 `imglab-daemon` binary, 但现有实现没有在启动前验证 binary 是否存在, packaged app 也没有明确 bundle daemon binary.

`Restart` / `Repair` 当前同步执行 `launchctl status`, 这类 OS-level service command 不应无限占用 Tauri command 线程. 即使 LaunchAgent 异常或系统服务卡住, UI 也应得到 recoverable diagnostic, 而不是整个 Settings workflow 停住.

## Goals / Non-Goals

**Goals:**

- daemon binary 缺失时 fail fast, 返回可操作 recoverable error.
- packaged app 能从 Tauri external sidecar resources 位置解析 daemon binary, 且 release packaging 会包含 daemon binary.
- `launchctl` bootstrap, enable, bootout, kickstart 操作有 bounded timeout.
- 保持 background daemon 优先, sidecar fallback 和 loopback token contract 不变.

**Non-Goals:**

- 不改变 daemon HTTP API, token format, runtime file format 或 task persistence schema.
- 不引入新的 background service manager abstraction.
- 不改变 LaunchAgent label, registry path 或 automation-enabled library scanning 语义.

## Decisions

1. 在 `daemon_binary_path` 内做存在性验证.

   这样所有调用方在写 plist 或启动 sidecar 前都能得到同一种 recoverable error. 替代方案是在 `Command::new` 失败后改错误文案, 但 Settings Repair 仍可能写入指向不存在 binary 的 LaunchAgent.

2. Runtime discovery 增加 app bundle resources 和 target-triple sidecar lookup.

   `beforeBuildCommand` 先构建 release daemon, 再将真实 binary 复制到 Tauri `externalBin` 期望的 `src-tauri/binaries/imglab-daemon-aarch64-apple-darwin`. Runtime 从当前 executable 同目录和 macOS `Contents/Resources` 中查找 plain binary name 和 target-triple sidecar name. 替代方案是立即切换到 shell plugin sidecar API, 但当前代码直接用 `std::process::Command`, 该切换会扩大改动范围并需要重新设计 runtime path resolution.

3. `launchctl` 使用 `spawn` + `try_wait` loop 实现短 timeout.

   Rust 标准库没有跨平台 `wait_timeout`, 因此用 polling loop 避免新增依赖. Timeout 后 best-effort kill child, 返回 recoverable service timeout. 替代方案是把 command 放到前端异步状态里等待, 但 backend command 仍会占用线程并使用户无法判断结果.

## Risks / Trade-offs

- Packaged resource path 不同平台可能不同 -> runtime 先保留 current executable 同目录 fallback, macOS 增加 `Contents/Resources` fallback, 并同时查找 Tauri external sidecar target-triple 文件名.
- `launchctl` timeout 过短可能在慢机器上误报 -> timeout 只影响 service control command, 用户可以再次刷新状态或重试; daemon health discovery 仍通过 runtime file 判断真实状态.
- 缺失 binary 从 late spawn error 变为 early recoverable error -> 这是更明确的失败模式, 不改变成功路径.
