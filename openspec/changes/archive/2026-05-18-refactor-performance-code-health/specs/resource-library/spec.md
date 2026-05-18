## ADDED Requirements

### Requirement: SQLite Schema 提供热路径索引
系统 SHALL 为 Gallery, Search, Metadata Review, Album 和 Task 热路径创建必要 SQLite indexes, 并使用 idempotent migration 保证既有资源库可升级.

#### Scenario: 打开旧资源库时补索引
- **WHEN** 用户打开缺少热路径索引的旧资源库
- **THEN** 系统通过 migration 创建缺失索引, 且不改变既有业务数据

#### Scenario: Gallery 热路径使用索引
- **WHEN** Gallery/Search 查询读取 assets, asset_versions, generation_events, metadata_suggestions, album_items, asset_tags 或 tags
- **THEN** schema 必须存在支持常用 filter, join 和 ordering 的索引

## MODIFIED Requirements

### Requirement: 使用 SHA-256 Checksum 校验 Asset Version 文件
系统 SHALL 对新写入的 asset version 文件计算 SHA-256 checksum, 并在 integrity check 和 file context 中使用对应 checksum algorithm 校验文件. 系统 MUST 保持对历史 MD5 metadata 的兼容读取, 但当前标准 checksum algorithm 是 `SHA-256`. Checksum 计算 MUST 使用 bounded memory streaming read, 不得把完整大文件读入内存后再计算 digest.

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

#### Scenario: 大文件 Checksum 使用 Bounded Memory
- **WHEN** 系统对 asset version 文件计算 checksum
- **THEN** 系统按固定大小 buffer streaming 读取文件, 内存峰值不得随文件大小线性增长

### Requirement: 写入 Asset Version 时记录图片分辨率
系统 SHALL 在导入或生成图片写入新的 asset version 时解析图片文件头, 并在能够可靠读取时记录 width 和 height. 对无法识别或无法可靠读取尺寸的文件, 系统 MUST 将 width 和 height 保持为空, 不得伪造分辨率. 图片尺寸解析 MUST 只读取必要 header 范围, 不得为了读取尺寸而加载完整大图片.

#### Scenario: 导入 PNG 图片记录分辨率
- **WHEN** 用户导入一个包含有效 PNG header 的图片文件
- **THEN** 系统在 asset version 中记录该图片的 width 和 height

#### Scenario: 生成图片记录分辨率
- **WHEN** generation flow 产生一个可识别分辨率的图片文件
- **THEN** 系统在输出 asset version 中记录该图片的 width 和 height

#### Scenario: 图片尺寸不可用
- **WHEN** 系统无法从图片文件头可靠读取 width 或 height
- **THEN** 系统写入 asset version 时保持 width 和 height 为空

#### Scenario: 大图片尺寸解析使用 Header Read
- **WHEN** 系统解析 PNG, JPEG 或 WebP 文件尺寸
- **THEN** 系统只读取识别格式所需的 header 或 bounded prefix, 不得整文件读入内存
