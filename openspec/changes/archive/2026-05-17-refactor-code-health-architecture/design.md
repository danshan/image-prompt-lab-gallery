## Context

当前代码已经完成 MVP 主路径, 但核心边界仍带有原型阶段痕迹. `imglab-core` 是 GUI 和 CLI 写操作的唯一业务事实来源, 但 desktop command 仍保留 direct SQL read. `library.rs` 聚合了 schema, storage, service, read model, repair, export 和测试, 导致任何行为修改都需要在一个超大文件中穿梭. Generation orchestration 在 CLI 和 Tauri 中重复, 使 provider dispatch, input loading 和 image-to-image validation 容易分叉.

Checksum 是这次变更中唯一明确的数据语义调整. 当前实现将 MD5 digest 写入历史 `sha256` 列, 同时又暴露 `checksum_algorithm/checksum`, 这会让 public API 和持久化 metadata 同时存在两套互相冲突的语义. 本次变更将当前标准统一为 SHA-256.

## Goals / Non-Goals

**Goals:**

- 修正 checksum 当前标准, 新写入和 repair normalization 统一使用 SHA-256.
- 保持历史 library 可读, 但 repair 后 metadata 收敛为当前标准.
- 移除 desktop direct SQL read path, 真实 Tauri read path 只通过 core read model.
- 拆分 `library.rs`, 让 schema, storage, assets, gallery, metadata, albums, repair, export, generation helper 分别有清晰职责.
- 收敛 CLI 和 desktop generation request construction, 避免参数校验和 provider dispatch 分叉.
- 删除或提取低收益重复逻辑, 包括 tag attach, version row mapping, latest event lookup 和 rating validation.

**Non-Goals:**

- 不引入 daemon, IPC, local HTTP API 或多用户协作.
- 不实现 native OpenAI/Grok provider.
- 不改变 managed file directory layout.
- 不引入通用 repository trait 层.
- 不在本轮做桌面视觉重设计.
- 不把 archived OpenSpec 历史改写为新语义.

## Decisions

### 1. 当前 checksum 标准使用 SHA-256

新导入, 新生成和 child version 创建都计算 SHA-256. SQLite 历史列 `sha256` 保留, 但只写入真实 SHA-256 digest. `checksum_algorithm` 写入 `SHA-256`, `checksum` 写入同一个 digest.

选择这个方案的原因是 `sha256` 列已经存在, 且 SHA-256 是比 MD5 更符合列名和长期完整性语义的默认算法. 相比继续使用 MD5, 该方案能消除 "MD5 写入 sha256" 的根本语义错误. 相比立即重命名数据库列, 它避免了高风险 migration 和大量 SQL churn.

### 2. Public API 使用 canonical checksum fields

业务 API 和 JSON output 以 `checksum_algorithm` 和 `checksum` 为 canonical fields. `sha256` 不再作为业务字段暴露; 如短期为兼容保留, 值也必须是真实 SHA-256.

这样可以避免未来再次被 SQLite 历史列名绑定. 代价是前端 types 和 CLI tests 需要同步更新.

### 3. Repair 负责标准化历史 metadata

Integrity check 继续按 row 自身 `checksum_algorithm` 校验, 保证旧数据在 repair 前可读. Repair dry-run 报告 checksum metadata 与当前标准不一致的 rows. Repair apply 重新计算 SHA-256 并更新 `sha256`, `checksum_algorithm`, `checksum`.

该方案将兼容读取和标准化写入分离, 避免打开 library 时自动修改数据. 用户仍通过显式 repair 承担 metadata 更新.

### 4. Core 模块拆分优先使用私有函数, 不引入 repository trait

目标模块布局为:

```text
crates/imglab-core/src/
  library/
    mod.rs
    service.rs
    schema.rs
    storage.rs
    assets.rs
    gallery.rs
    metadata.rs
    albums.rs
    repair.rs
    export.rs
    generation.rs
```

拆分以职责为单位, `LocalLibraryService` 仍是 public entry point. 内部模块通过接收 `&Connection` 的私有函数组合. 当前项目是 single-process local SQLite, 抽象 repository trait 不会降低实际复杂度, 反而会增加间接性.

### 5. Tauri command 只做 transport mapping

移除 `gallery_items` direct SQL command. 真实 Tauri Gallery 使用 `query_gallery`, Inspector 使用 `get_asset_detail`. Browser preview/mock 仍可保留本地 mock data 和本地 filtering, 但不能影响真实 Tauri mode.

该决策确保 Gallery query, provider/model/prompt 映射, file context 和 lineage 都由 core read model 统一定义.

### 6. Generation orchestration 由 core 提供共享边界

CLI 和 desktop 共享 provider name normalization, provider dispatch, operation inference, image-to-image input validation, input bytes loading 和 request construction. Provider crate 仍负责 provider-specific execution, 例如 Codex CLI command 构造, stdout/stderr logging 和 output parsing.

该方案消除 CLI/Tauri 参数分叉, 但不会把 provider implementation details 拉进 core.

## Risks / Trade-offs

- [Risk] 从 public DTO 或 JSON output 移除 `sha256` 会破坏仍引用该字段的 frontend types 和 tests. → Mitigation: 同步更新 CLI tests, Tauri views 和 frontend `Version` type, 并只保留 `checksum_algorithm/checksum`.
- [Risk] Repair 将 MD5 metadata 改为 SHA-256 会改变已有 SQLite digest fields. → Mitigation: repair 必须显式触发, 且测试证明 asset ids, version ids, file paths 和 file bytes 不变.
- [Risk] 拆分 `library.rs` diff 很大, 容易混入行为变更. → Mitigation: 先完成 checksum 行为变更并测试, 再做主要为机械移动的模块拆分.
- [Risk] Generation helper 放在 core 后可能吸收 provider-specific 细节. → Mitigation: helper 只处理 request construction 和 dispatch 边界, provider execution 仍在 provider crate.
- [Risk] 保留 SQLite `sha256` 列但 public API 不暴露它, 可能让内部命名仍显历史. → Mitigation: 在 repository row 层隔离该列, 业务 DTO 只使用 canonical checksum fields.

## Migration Plan

1. 修改新写入路径, 让 import, generate 和 child version 创建写入 SHA-256.
2. 修改 repair dry-run/apply, 将非 SHA-256 标准 metadata 识别并修复为 SHA-256.
3. 修改 DTO/transport/CLI output, 以 canonical checksum fields 为主.
4. 移除 `gallery_items` command, 清理前端真实 Tauri path 中的旧引用.
5. 拆分 core 模块, 每次拆分后运行 Rust 测试.
6. 提取重复 helper, 确保行为由现有 tests 和新增 tests 覆盖.
7. 更新 docs 和 current OpenSpec specs.

Rollback strategy: 本变更不自动修改用户 library. 如果实现期间出现问题, 可回滚代码变更. 对已经显式执行 repair 的测试 library, 可从 repair 前备份恢复; 本轮不设计自动 downgrade 到 MD5.

## Open Questions

当前没有阻塞实施的开放问题. `sha256` 是否保留为短期兼容 JSON 字段由实现时按测试影响决定, 但最终目标是不作为业务字段暴露.
