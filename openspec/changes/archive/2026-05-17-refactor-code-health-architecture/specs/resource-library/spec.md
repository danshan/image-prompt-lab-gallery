## MODIFIED Requirements

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
