## Purpose

Define local resource library layout, integrity checking, repair, import/export, and file context behavior.

## Requirements

### Requirement: 创建受管理资源库

系统 SHALL 支持在指定本地路径创建一个受管理资源库, 包含 `manifest.json`, `library.sqlite`, `originals`, `derivatives`, `sidecars`, `exports` 和 `trash`.

#### Scenario: 创建新资源库

- **WHEN** 用户请求在空目录创建资源库并提供名称
- **THEN** 系统创建资源库目录结构, 写入 manifest, 初始化 SQLite schema, 并返回资源库 id 和路径

### Requirement: 打开和校验资源库

系统 SHALL 在打开资源库时校验 manifest, schema version 和必要目录, 并拒绝打开不兼容或损坏的资源库.

#### Scenario: 打开 schema 不兼容的资源库

- **WHEN** 用户打开 schema version 高于当前应用支持版本的资源库
- **THEN** 系统返回 `SchemaMismatch` 错误且不修改资源库内容

### Requirement: 管理资源库 registry

系统 SHALL 维护本机 app-level registry, 记录最近打开的资源库, display name, root path, hidden 状态和 last opened time.

#### Scenario: 隐藏资源库

- **WHEN** 用户隐藏一个已注册资源库
- **THEN** 系统将该资源库从默认列表中隐藏, 但不删除资源库目录或 SQLite 数据

### Requirement: 按日期分桶存储 Managed 图片文件

系统 SHALL 在写入新的 managed 图片文件时使用 UUID 文件名, 并将文件保存到按创建日期拆分的相对路径 `$year/$month/$filename`.

#### Scenario: 导入图片写入日期分桶路径

- **WHEN** 用户导入一个本地图片文件
- **THEN** 系统复制图片到 managed library 的 `originals/$year/$month/$uuid.$extension` 路径, 且后续不依赖原始外部路径

#### Scenario: 生成图片写入日期分桶路径

- **WHEN** generation flow 产生一个新的图片 asset version
- **THEN** 系统将生成图片写入 managed library 的 `originals/$year/$month/$uuid.$extension` 路径

#### Scenario: 文件名不复用外部源文件名

- **WHEN** 两个不同源文件具有相同文件名
- **THEN** 系统为每个 managed 图片文件生成不同 UUID 文件名, 并在 asset version 中记录各自相对路径

### Requirement: 使用 SHA-256 Checksum 校验 Asset Version 文件

系统 SHALL 对新写入的 asset version 文件计算 SHA-256 checksum, 并在 integrity check 和 file context 中使用对应 checksum algorithm 校验文件. 系统 MUST 保持对历史 MD5 metadata 的兼容读取, 但当前标准 checksum algorithm 是 `SHA-256`.

#### Scenario: 新导入文件记录 SHA-256

- **WHEN** 用户导入一个图片文件
- **THEN** 系统记录该 managed 文件的 checksum algorithm 为 `SHA-256`, 并记录 64 位十六进制 SHA-256 digest

#### Scenario: Integrity Check 使用记录的算法

- **WHEN** integrity check 校验一个 asset version 文件
- **THEN** 系统使用该 version 记录的 checksum algorithm 重新计算 digest, 并与记录的 checksum 比较

#### Scenario: SHA-256 不匹配

- **WHEN** integrity check 发现文件当前 SHA-256 与记录的 SHA-256 不一致
- **THEN** 系统报告 `FileIntegrityMismatch` 并标明受影响的 version id

#### Scenario: 历史 MD5 Metadata 保持可读

- **WHEN** integrity check 校验一个历史 asset version 且其 checksum algorithm 为 `MD5`
- **THEN** 系统使用 MD5 重新计算 digest, 并与该 version 记录的 checksum 比较

### Requirement: 提供资源库实际存储大小

系统 SHALL 提供当前 resource library 的实际 managed storage size, 且该值不得包含固定容量上限.

#### Scenario: 读取 Library Storage Size

- **WHEN** 桌面端请求 Library Status read model
- **THEN** 系统返回当前 Library managed files 占用大小, 且不返回固定容量上限或百分比上限

### Requirement: 写入 Asset Version 时记录图片分辨率

系统 SHALL 在导入或生成图片写入新的 asset version 时解析图片文件头, 并在能够可靠读取时记录 width 和 height. 对无法识别或无法可靠读取尺寸的文件, 系统 MUST 将 width 和 height 保持为空, 不得伪造分辨率.

#### Scenario: 导入 PNG 图片记录分辨率

- **WHEN** 用户导入一个包含有效 PNG header 的图片文件
- **THEN** 系统在 asset version 中记录该图片的 width 和 height

#### Scenario: 生成图片记录分辨率

- **WHEN** generation flow 产生一个可识别分辨率的图片文件
- **THEN** 系统在输出 asset version 中记录该图片的 width 和 height

#### Scenario: 图片尺寸不可用

- **WHEN** 系统无法从图片文件头可靠读取 width 或 height
- **THEN** 系统写入 asset version 时保持 width 和 height 为空

### Requirement: 修复历史 Asset Version 文件元数据

系统 SHALL 提供显式的资源库 repair 操作, 用于将历史 asset version 文件和 SQLite metadata 修复到当前 managed library 标准. repair MUST 支持 dry run, 并在执行模式下同步修复文件路径, checksum metadata 和图片分辨率. 当前 checksum 标准 MUST 为 SHA-256.

#### Scenario: Dry Run 检查历史脏数据

- **WHEN** 用户对资源库执行 repair dry run
- **THEN** 系统返回将要移动的文件数量, 将要更新的 checksum 数量, 将要更新的 dimensions 数量和无法自动修复的问题列表, 且不修改文件系统或 SQLite

#### Scenario: 修复非标准文件路径

- **WHEN** 某个 asset version 的文件存在, 但 `file_path` 不符合 `originals/$year/$month/$uuid.$extension`
- **THEN** repair 操作将文件移动到标准路径, 并同步更新 SQLite 中该 version 的 `file_path`

#### Scenario: 回填历史图片分辨率

- **WHEN** 某个 asset version 文件可读取图片尺寸, 且 SQLite 中 width 或 height 缺失或不一致
- **THEN** repair 操作更新 SQLite 中该 version 的 width 和 height

#### Scenario: 修复历史 Checksum Metadata

- **WHEN** 某个 asset version 文件存在, 但 checksum algorithm 或 checksum 不符合当前 SHA-256 标准
- **THEN** repair 操作重新计算文件 SHA-256, 并更新该 version 的 checksum algorithm 和 checksum

#### Scenario: 文件缺失无法自动修复

- **WHEN** repair 操作发现某个 asset version 记录的文件不存在
- **THEN** 系统在 repair summary 中报告该 version 的问题, 且不删除该 SQLite 记录

### Requirement: 导入和导出资源库内容

系统 SHALL 支持将外部图片导入 managed library, 并支持按资源库或相册导出图片和 sidecar metadata. 导入时, 系统 MUST 将图片复制到按日期分桶的 managed file layout, 文件名 MUST 使用 UUID, 不得复用外部源文件名作为 managed 文件名.

#### Scenario: 导入外部图片

- **WHEN** 用户导入一个本地图片文件
- **THEN** 系统复制图片到 `originals/$year/$month/$uuid.$extension`, 计算 SHA-256 checksum, 写入 SQLite, 并且后续不依赖原始外部路径

### Requirement: 校验资源库完整性

系统 SHALL 支持检查 SQLite 记录与 managed file layout 的一致性, 包括文件存在性和 checksum. 校验 checksum 时, 系统 MUST 使用 asset version 记录的 checksum algorithm.

#### Scenario: 原始文件丢失

- **WHEN** integrity check 发现某个 asset version 的文件不存在
- **THEN** 系统报告 `FileIntegrityMismatch` 并标明受影响的 version id

#### Scenario: Checksum 不匹配

- **WHEN** integrity check 发现某个 asset version 的当前 digest 与记录 checksum 不一致
- **THEN** 系统报告 `FileIntegrityMismatch` 并在问题信息中包含记录的 checksum algorithm

### Requirement: 提供 File Context Read Model

系统 SHALL 在 asset detail read model 中提供当前 version 的文件上下文, 包括 filename, relative location, MIME type, checksum algorithm, checksum 和 integrity status.

#### Scenario: 查看文件上下文

- **WHEN** 用户在 Inspector 查看 File section
- **THEN** 系统返回当前 version 的文件名, 相对位置, MIME type, checksum algorithm, checksum 和 integrity 状态

#### Scenario: 查看文件分辨率

- **WHEN** 当前 version 记录了 width 和 height
- **THEN** 系统在 File Context Read Model 中返回该 width 和 height

#### Scenario: 展示 SHA-256 Checksum

- **WHEN** 当前 version 的 checksum algorithm 为 `SHA-256`
- **THEN** UI 展示 `Checksum    SHA-256: $hash`

### Requirement: File Context 允许缺失的派生字段为空

系统 SHALL 在无法可靠获得 file size, dimensions 或 generation duration 时返回空值, 不得伪造真实文件元数据.

#### Scenario: 文件尺寸信息不可用

- **WHEN** 当前资源库没有记录某个 version 的尺寸信息
- **THEN** 系统返回空尺寸字段, UI 展示 unavailable 状态

### Requirement: 支持 Inspector 触发完整性复查

系统 SHALL 提供从桌面端触发资源库或当前 asset version 完整性复查的能力.

#### Scenario: 重新校验当前文件

- **WHEN** 用户在 Inspector File section 点击 re-verify
- **THEN** 系统校验当前 version 文件存在性和 checksum, 并返回更新后的 integrity 状态
