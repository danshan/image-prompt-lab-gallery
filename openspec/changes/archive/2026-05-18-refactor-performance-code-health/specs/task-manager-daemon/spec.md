## ADDED Requirements

### Requirement: Daemon HTTP 查询在长任务期间保持响应
daemon SHALL 在长任务执行和 scheduler activity 期间保持 health, capabilities 和 task read API 可响应, 并避免为不需要 mutable state 的请求持有全局 state lock.

#### Scenario: Health Check 不等待长任务
- **WHEN** daemon 正在执行长任务
- **THEN** `/v1/health` 请求仍能快速返回 daemon health 和 API version

#### Scenario: Capabilities Check 不等待 Scheduler Tick
- **WHEN** scheduler tick 正在评估任务
- **THEN** `/v1/capabilities` 请求不得因为无关 mutable state lock 长时间阻塞

### Requirement: Scheduler 无任务时避免深拷贝 Daemon State
daemon scheduler SHALL 在无 opened library 或无 eligible task 时避免深拷贝完整 daemon state.

#### Scenario: 无 Opened Library
- **WHEN** daemon 没有 opened libraries
- **THEN** scheduler tick 直接返回 no work, 不 clone 完整 `DaemonState`

#### Scenario: 无 Runnable Task
- **WHEN** opened libraries 中没有 queued 或 retry-ready task
- **THEN** scheduler tick 避免执行昂贵 snapshot, 并保持下一轮 tick 可继续检查

### Requirement: Daemon Client Timeout 和 Backoff 区分请求类型
desktop daemon client SHALL 根据请求类型使用合适 timeout 和 transient failure backoff.

#### Scenario: Health Check 使用短 Timeout
- **WHEN** desktop 检查 daemon health
- **THEN** client 使用短 timeout, 失败后可尝试启动 sidecar

#### Scenario: Task Detail 使用较长 Timeout
- **WHEN** desktop 请求 task detail 或 log preview
- **THEN** client 使用足够长的 timeout, 避免在 daemon 短暂繁忙时误报失败

## MODIFIED Requirements

### Requirement: 提供本地 Task Manager Daemon
系统 SHALL 提供独立本地 daemon 执行长任务, daemon 只能通过 loopback local HTTP 接受本机 client 请求, 且所有请求必须通过本地 session token 鉴权. daemon implementation MUST keep request parsing and authentication outside expensive mutable state sections where possible.

#### Scenario: Desktop 启动并发现 Daemon
- **WHEN** desktop app 启动且 runtime file 指向健康 daemon
- **THEN** desktop app 通过 `/v1/health` 确认 API version 和 daemon 状态, 并复用该 daemon

#### Scenario: Desktop 启动 Daemon
- **WHEN** desktop app 找不到健康 daemon
- **THEN** desktop app 启动 app-owned daemon sidecar, 读取 runtime file 中的 port 和 token location, 并通过 loopback API 连接

#### Scenario: 拒绝非授权请求
- **WHEN** local HTTP request 未携带有效 session token
- **THEN** daemon 拒绝请求, 且不得返回 task, library 或日志内容

#### Scenario: 只绑定 Loopback
- **WHEN** daemon 启动 HTTP listener
- **THEN** daemon 只绑定 loopback address, 不暴露局域网或远程访问接口
