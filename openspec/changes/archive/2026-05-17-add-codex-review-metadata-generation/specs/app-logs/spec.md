## ADDED Requirements

### Requirement: App-owned 日志列表

系统 SHALL 提供查询最近 app-owned 生成日志的能力. 日志列表 SHALL 至少包含日志路径, 日志类型, 修改时间, 文件大小和内容预览. 系统 MUST 支持 Codex image generation 日志和 Codex metadata generation 日志.

#### Scenario: 查询最近 App Logs

- **WHEN** 桌面应用请求最近 app logs
- **THEN** 系统返回按修改时间倒序排列的 app-owned 日志列表

#### Scenario: 日志类型归类

- **WHEN** 日志文件名匹配 `imglab-codex-cli-*.log`
- **THEN** 系统将该日志标记为 `codex-image-generation`

#### Scenario: Metadata 日志类型归类

- **WHEN** 日志文件名匹配 `imglab-codex-metadata-*.log`
- **THEN** 系统将该日志标记为 `codex-metadata-generation`

### Requirement: 安全读取 App Log 内容

系统 SHALL 提供读取单个 app-owned 日志内容的能力. 系统 MUST 只允许读取 temp directory 中匹配已知 app log pattern 的文件, 并 MUST 拒绝任意其他路径.

#### Scenario: 读取允许的日志

- **WHEN** 桌面应用请求读取 temp directory 中匹配 app log pattern 的日志
- **THEN** 系统返回该日志的内容预览

#### Scenario: 拒绝任意路径读取

- **WHEN** 桌面应用请求读取不在允许目录或不匹配 app log pattern 的路径
- **THEN** 系统拒绝读取并返回可恢复错误

#### Scenario: 限制大型日志读取

- **WHEN** app-owned 日志文件超过内容预览上限
- **THEN** 系统只返回安全大小范围内的内容预览, 不阻塞 Settings UI
