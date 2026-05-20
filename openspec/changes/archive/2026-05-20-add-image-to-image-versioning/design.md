## Context

项目当前已经具备图生图的基础枚举, provider capability 检查, `input_version_id`, `parent_version_id` 和 generation event 记录. 但现有 `asset_versions.id` 是 UUID, `version_label` 只是弱语义标签, 不能承担用户可见的递增版本号. 上传参考图当前更接近临时输入 bytes, 缺少 managed library 内可追溯的 reference asset/version 语义.

这次变更跨越 Rust core schema, generation orchestration, daemon task output, CLI 输出和 Desktop read model. 主要约束是保持 local-first resource library 的兼容升级, 不重写历史 UUID 和文件路径, 不引入新的外部 provider 或 graph lineage 模型.

## Goals / Non-Goals

**Goals:**

- 为每个 logical asset 提供从 `v1` 开始递增的用户可见版本号.
- 基于已有 version 图生图时, 在同一 asset 下创建下一数字版本并保留 parent chain.
- 上传参考图生成时, 将上传图导入为独立 reference asset/version, 输出创建独立 generated asset/version.
- generation event 能追溯 source version, output version, prompt, provider 和 parameters.
- 旧 library 和旧备份通过 schema migration 自动回填版本号后继续可用.

**Non-Goals:**

- 不替换内部 UUID 主键.
- 不支持多参考图, mask, 局部编辑或 graph-style lineage.
- 不改变 Codex CLI imagegen adapter 的模型和输出路径限制.
- 不改变 review-first metadata suggestion 语义.

## Decisions

### 保留 UUID 主键, 新增 `version_number`

`asset_versions.id` 继续作为内部稳定主键. 新增 `asset_versions.version_number INTEGER`, 并以 `(asset_id, version_number)` 约束保证同一 asset 内唯一.

理由: UUID 已经被 generation events, task outputs, CLI 参数和 read model 使用. 替换主键会扩大迁移风险, 而用户真实需要的是稳定的人类可读版本号.

替代方案是把 version id 改为数字或复合键. 这会破坏现有引用和 task output, 收益低于风险.

### 版本号按 asset 内部分配

新 asset 的首个 version 是 `1`. 同一 asset 创建 child version 时, 在插入 version 的同一事务内读取 `MAX(version_number) + 1`.

理由: 版本号是 logical asset 内的编辑历史, 不应该是全局序列. 全局序列会暴露无关资产之间的插入顺序, 也不能表达 `asset A v3` 这种用户心智.

### 上传参考图作为独立 reference asset

`input_file` 图生图先把上传图导入 managed library, 标记为 `status = reference`, reference version 为 `v1`. 生成输出创建新的 generated asset, output version 为 `v1`, generation event 的 `input_asset_version_id` 指向 reference version.

理由: 上传图是输入素材, 不应被强行放入输出作品的 parent chain. 但它必须进入 managed library, 这样 checksum, backup, restore, integrity check 和 lineage 都有统一事实来源.

替代方案是只保存临时 input bytes 或新增 reference file 表. 前者不可追溯, 后者会复制 asset version 已经具备的文件生命周期能力.

### Same-asset variation 使用 parent chain

`input_version_id` 图生图输出写回 source version 所属 asset, `parent_version_id = input_version_id`, `version_number = max + 1`.

理由: 这符合"基于已生成图片, 以新版本完成图片生成"的语义. Parent chain 仍然只表达同一 asset 内的版本 lineage, reference source 单独展示.

### 历史 library 通过 migration 升级

schema version 递增. 打开旧 schema 时添加并回填 `version_number`, 排序规则为同一 asset 内 `created_at ASC, id ASC`. Migration 不重写 UUID, 文件路径, checksum, parent links, generation events 或 task output links.

理由: 旧 library 没有显式数字版本号, 只能用确定性顺序建立兼容合同. 不尝试根据 parent graph 重排, 因为历史数据可能缺失或不完整, 确定性比猜测更可维护.

## Risks / Trade-offs

- [Risk] 上传参考图 provider 执行失败后留下 reference asset. -> Mitigation: 仅在 capability 和参数校验通过后导入 reference, provider 失败时保留 reference 并记录 failed generation event.
- [Risk] 历史 version 的数字顺序不完全符合真实创作 lineage. -> Mitigation: 保留 parent chain 作为真实 lineage, version_number 只按确定性历史顺序回填.
- [Risk] `MAX(version_number) + 1` 在并发写入下冲突. -> Mitigation: 在同一 SQLite transaction 内分配和插入, 依赖 `(asset_id, version_number)` unique index 兜底.
- [Risk] Reference asset 默认进入 Gallery 会污染主要内容流. -> Mitigation: default Gallery query 排除 `status = reference`, source link 和显式 filter 可访问.
- [Risk] DTO 增加字段影响 Desktop 和 CLI 映射. -> Mitigation: 同步更新 Tauri DTO, React types, CLI formatter 和 tests.

## Migration Plan

1. 将 library schema version 从当前版本递增.
2. Migration 添加 nullable `version_number`.
3. 按 `asset_id`, `created_at ASC`, `id ASC` 回填每个 asset 的 `1..N`.
4. 创建 `(asset_id, version_number)` unique index.
5. 所有新写入路径都必须设置 `version_number`.
6. Read model 在 migration 后统一暴露 `version_number` 和 `version_name`.
7. 如果打开未来 schema, 保持现有 `SchemaMismatch` 拒绝语义.

Rollback 策略: 不提供自动降级 schema. 如果升级失败, 返回可恢复 migration error, 不报告 library 打开成功. 已备份的旧 library 或旧 backup 仍可由新版本 app 通过同一 migration path 升级.

## Open Questions

无. Brainstorming 阶段已确认: 上传参考图使用独立 reference asset, 内部 UUID 保留, 用户可见版本号按 asset 数字递增.
