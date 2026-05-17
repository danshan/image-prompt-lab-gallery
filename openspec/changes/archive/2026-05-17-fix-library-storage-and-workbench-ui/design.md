## Context

当前资源库已经有 app-level registry, managed directory layout 和 Gallery / Inspector read model, 但实现仍存在几个会影响后续演进的问题: asset version 文件写入 `originals/imported` 或 `originals/generated` 单层目录, checksum 字段名和逻辑固定为 SHA-256, Library Status 使用静态容量展示, 桌面端同时显示系统标题栏和应用内标题栏, Library 切换入口还停留在刷新当前 Library 的按钮语义上.

这次 change 同时触及 Rust core, SQLite read model, Tauri window config 和 React workbench. 设计目标是收敛存储规则和 UI contract, 而不是扩大到 cloud sync, 多用户, daemon 或稳定 native provider client.

## Goals / Non-Goals

**Goals:**

- 统一 managed image file layout 为按日期拆分的 UUID 文件名路径, 降低单目录膨胀和名称冲突风险.
- 将 asset version checksum 语义切换到 MD5, 并让 read model 明确暴露 algorithm 和 digest, 使 UI 能稳定展示 `Checksum    MD5: $hash`.
- 支持桌面端从顶部切换多个已注册 Library, 并保持 Gallery / Inspector 查询上下文跟随当前 Library 切换.
- 去掉 Tauri 系统默认标题栏, 保留应用内标题栏和窗口控制体验.
- 修正 Sidebar 不能滚动, Library Status storage, selector 视觉样式, resolution 和 checksum 展示.
- 新导入和新生成图片需要在 core 写入 asset version 时解析并持久化真实 width 和 height, 使后续 read model 不依赖前端推断.
- 历史资源库需要一个显式 repair 操作, 用当前标准修复旧路径, checksum metadata, dimensions 和可识别脏数据.

**Non-Goals:**

- 不引入多用户协作, cloud sync, 加密, 备份迁移系统或 daemon.
- 不迁移到 native OpenAI / Grok image client.
- 不重做三栏工作台整体信息架构, 只修正当前需求覆盖的布局和展示 contract.
- 不承诺读取旧资源库历史文件的真实图片尺寸, 只保证缺失尺寸不被伪造.

## Decisions

### 文件路径使用日期分桶加 UUID 文件名

新写入的 original asset version 使用 `originals/$year/$month/$uuid.$ext` 相对路径. `$year` 和 `$month` 取 asset version 创建时间所在本地日期或 UTC 日期, 实现时必须保持单一选择并在测试中固定. 文件名使用新生成的 version UUID 或独立 file UUID, 但不能使用外部源文件名.

备选方案是保留 `imported/generated` 子目录再追加日期. 该方案会继续把来源类型编码进路径, 而来源类型已经存在于 version label 和 generation event 中, 对文件系统布局没有必要.

### Checksum read model 显式表达算法

Core 内部新增 MD5 digest 计算路径, 新写入和 integrity check 使用 MD5. DTO 层建议把 `FileContextView` 从单一 `checksum` 字符串扩展为 `checksum_algorithm` 和 `checksum` 或提供已经格式化的 display label. 更稳妥的边界是传结构化字段, 由 UI 渲染 `MD5: $hash`.

当前 SQLite 列名为 `sha256`, 这是历史实现细节. 实施时有两种可行路径:

- 最小迁移: 增加 `checksum_algorithm` 和 `checksum` 新列, 新数据写入新列, read model 兼容读取旧 `sha256`.
- 破坏性较小的 MVP 修正: 保留列名但语义切到 MD5, 同时在代码中隔离字段名债务并安排后续 schema rename.

推荐第一种, 因为它避免列名与语义长期背离. 如果 migration 成本过高, 第二种只能作为短期过渡, 并必须让 UI 和 integrity message 不再显示 SHA-256.

### Library selector 归属 Sidebar 顶部

Library 切换入口放在 Sidebar 顶部, 作为当前 Library 上下文的 primary control. 顶部 app bar 只承担应用内标题栏, 拖拽区域和窗口控制职责. Sidebar Library selector 只展示 Library 名称, 使用向下箭头表达下拉 affordance, 不展示路径. 切换 Library 后, frontend 需要清空 selected asset, 重新加载 Gallery, 重置或保留 query 应由现有 UX 决策决定; 默认保留 query 文本和过滤条件, 但清空 selection 和 detail.

### 无系统装饰窗口需要补齐拖拽和窗口控制

Tauri config 设置窗口 decorations 为 false. 应用内标题栏必须提供可拖拽区域和窗口控制按钮, 并调用 Tauri window API 完成 minimize, maximize / restore 和 close. macOS 可以保留类 traffic-light 视觉, 但不能再与系统按钮重复.

### Sidebar 通过密度和区域分配避免滚动

Sidebar 本身不设置 `overflow: auto`. 在窄高窗口下, 不关键内容优先折叠或隐藏, 例如 app version, Library Status 次要行和低优先级 nav count. Library Status 保持 storage size, integrity 和 run check 入口, 不显示容量上限和 meter.

### Selector 组件统一为 app 内控件样式

所有 native `select` 应使用统一 class 或抽象为 `SelectControl` component, 包含一致的 border, background, focus ring, height, icon affordance 和 disabled 状态. 不改变底层控件语义, 避免为视觉修正引入复杂 combobox 行为.

### 图片分辨率在 Core 写入时解析

Core 在复制 managed file 后, 写入 `asset_versions` 前解析图片文件头并记录 `width` 和 `height`. 第一阶段只做轻量 header parsing, 支持 MVP 常见格式 PNG, JPEG 和 WebP. 对无法识别, 文件不完整或 header 异常的文件, 返回空尺寸并继续导入或生成流程, 因为分辨率是派生 metadata, 不应阻塞文件入库.

备选方案是前端加载图片后回写尺寸. 该方案会把写操作和 metadata 事实来源拆到 UI, 违反 Rust core 作为唯一业务事实来源的边界, 因此不采用.

### 历史资源库 Repair 是显式操作

Repair 不在 `open_library` 或 schema migration 中自动执行, 因为它会移动文件并更新 SQLite. Core 提供 `repair_library` 服务, 支持 dry run 和 apply 两种模式. Repair 使用当前标准作为目标状态:

- 文件路径为 `originals/$year/$month/$version_uuid.$extension`.
- checksum algorithm 为 `MD5`, checksum 为当前文件内容的 MD5.
- width 和 height 来自 core-side image header parser.
- SQLite 中的 `file_path`, `checksum_algorithm`, `checksum`, `sha256`, `width`, `height` 与文件系统保持一致.

路径修复只处理当前 `file_path` 指向的文件存在且目标路径没有冲突的情况. 如果目标路径已存在且无法证明等价, 或当前文件缺失, repair summary 记录 issue 并跳过该 version. 这样可以避免在历史脏数据中做高风险猜测.

备选方案是提供一次性 SQL migration. 该方案无法可靠移动文件, 也无法读取图片 header 和重新计算 checksum, 因此不采用.

## Risks / Trade-offs

- [旧资源库已有 SHA-256 数据] → migration 或兼容读取必须明确. 新写入用 MD5 后, integrity check 需要知道每条 version 的算法, 否则会误报 mismatch.
- [无系统标题栏影响窗口操作] → 实施后必须用 browser / Tauri 手动验证拖拽, resize, maximize, close 和 keyboard focus.
- [Sidebar 不滚动可能隐藏信息] → 通过优先级折叠而不是挤压文本解决, 并确保核心导航和 Library Status 关键字段仍可见.
- [日期分桶的时区选择影响可复现性] → 测试中固定时间源, 文档中明确使用创建时间日期, 避免同一操作在不同时区路径不同导致断言不稳.
- [MD5 不是安全哈希] → 本场景只用于本地文件完整性和 UI checksum, 不用于安全校验或防篡改信任边界.
- [图片 header parser 覆盖有限] → 先覆盖 PNG, JPEG 和 WebP, 未识别格式保持尺寸为空并在 UI 展示 unavailable, 后续再按真实 provider 输出扩展格式.
- [Repair 会修改历史文件路径和 SQLite] → 必须提供 dry run, apply 模式逐条处理, 对缺失文件和目标冲突只报告 issue, 不做破坏性猜测.

## Migration Plan

1. 扩展 schema, DTO 和 migration, 支持 checksum algorithm 与 digest. 对旧 `sha256` 数据标记为 `SHA-256`, 对新写入数据标记为 `MD5`.
2. 替换 import, generated version persistence 和 integrity check 的新写入路径为 MD5 + `originals/$year/$month/$uuid.$ext`.
3. 在 import 和 generated version persistence 中解析图片尺寸, 并写入 `asset_versions.width` 和 `asset_versions.height`.
4. 增加显式 repair service, 对历史 asset version 执行路径标准化, MD5 checksum 修复和 dimensions 回填.
5. 更新 file context read model, storage size 计算和 Tauri command mapping.
6. 更新 Tauri window config 和 React app shell, 增加应用内标题栏与 Sidebar Library selector, 调整 Sidebar 和 selector 样式.
7. 添加 core tests, frontend state tests 和手工桌面验证步骤.

Rollback 策略是保留 migration 兼容读取旧列, 如果 UI 修改回滚, core 写入的新路径和 MD5 数据仍应能被旧 read model 至少作为 managed files 打开. 若旧版本完全不理解新 checksum 列, 需要把版本兼容性限制记录在 schema version 中.

## Open Questions

- 日期分桶应使用 UTC 还是本地时区. 推荐 UTC, 因为更适合测试和跨平台一致性.
- 是否在本次实现中重命名 SQLite `sha256` 列. 推荐新增结构化 checksum 字段, 避免破坏旧数据和 SQL 查询.
