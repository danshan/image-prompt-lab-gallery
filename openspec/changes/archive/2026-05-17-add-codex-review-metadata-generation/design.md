## Context

当前 Review Inbox 已有 review-first 边界: pending suggestion 可以在本地编辑, 只有用户接受后才写入 canonical asset metadata. 但 `title`, `description` 和 `JSON Schema Prompt` 的重新生成仍由前端 helper 根据 prompt 做确定性派生, 既不是 Codex CLI 生成, 也缺少真实长任务 loading 反馈.

现有 Codex CLI image provider 已经承担图片生成, 但它的 contract 是图片专用: prompt 要求使用 imagegen skill, 输出解析为图片路径, 成功后导入 managed library 并创建 asset version. Review metadata generation 的输出是文本字段草稿, 不应复用 `ImageProvider`.

Settings 当前只展示 library 基本信息, 还没有统一的 app log 浏览入口. Codex CLI 失败时, 用户需要能在 UI 中刷新并查看最近日志.

## Goals / Non-Goals

**Goals:**

- 通过 Codex CLI 生成 Review `title`, `description` 和 `JSON Schema Prompt`.
- 让 title 和 description 以简体中文输出为主.
- 对每个 Review 字段提供独立 loading 状态, 成功后只更新该字段草稿.
- JSON Schema Prompt 输出必须能解析为合法 JSON object, 并以稳定格式写回表单.
- 在 Settings 中提供 Logs 模块, 支持最近日志列表, 刷新和内容预览.
- 保持 review-first: 生成草稿不直接写入 canonical metadata.

**Non-Goals:**

- 不修改图片生成 provider 的外部行为.
- 不把 metadata generation 写入 generation event 或 asset version lineage.
- 不自动生成 tags 或 category.
- 不把 Settings log reader 变成任意文件读取能力.
- 不新增 OpenAI 或 Grok native metadata client.

## Decisions

### 1. 新增 Codex metadata generator, 不复用 ImageProvider

新增后端边界 `CodexCliMetadataGenerator`, 与 `CodexCliImageProvider` 并列. 它负责构造 `codex exec --json`, 写日志, 解析 stdout/stderr, 返回单个字段结果.

选择原因:

- 图片生成和 metadata 生成的输出类型不同. 前者是图片 bytes, 后者是文本草稿.
- 图片生成有持久化副作用: 写入 version 和 generation event. Review 字段生成不能有这些副作用.
- 独立边界可以让 prompt contract, parser 和错误处理更小, 更容易测试.

备选方案是复用 `CodexCliImageProvider`, 但会污染 `ImageProvider` 语义并引入错误的持久化路径, 因此不采用.

### 2. 通过 Tauri command 触发字段级生成

新增 `generate_review_field` command. 输入包含 library path, asset id, suggestion id, field name 和当前上下文. 输出包含 field, value 和 log path.

选择原因:

- 前端不直接执行本机命令, 避免命令构造, stdout 解析和日志路径安全分散到 UI.
- 字段级 command 能精确表达 loading 状态, 失败时只影响当前字段.
- 结果只返回到 Review form draft, 不修改 `metadata_suggestions` 或 `assets`.

### 3. JSON Schema Prompt 使用严格解析后返回

Codex prompt 要求 schema prompt 返回 JSON object. 后端从最终输出中提取第一个 JSON object, 用 `serde_json` 解析, 再 pretty-print 返回.

选择原因:

- UI 中的 JSON Schema Prompt 是可继续编辑和复用的结构化文本, 不应接受 Markdown fence 或解释性输出.
- 后端解析失败可以给出 recoverable error 和 log path, 用户草稿不丢失.

### 4. 初始 Review entry 保持快速 fallback

Gallery 点击 Review 时仍快速创建 pending suggestion, 使用已有 asset metadata 和本地 fallback 作为初始值. 不在创建 suggestion 时自动跑三次 Codex.

选择原因:

- Review entry 不应被 Codex CLI 可用性阻塞.
- 用户可以按需对具体字段 regenerate, 失败也只影响一个字段.
- 避免一次点击造成多个长任务和复杂取消语义.

### 5. Settings Logs 只读取 app-owned temp logs

新增 `list_app_logs` 和 `read_app_log`. 日志来源限制为 temp directory 中的 app-owned pattern:

- `imglab-codex-cli-*.log`
- `imglab-codex-metadata-*.log`

`read_app_log` 必须拒绝不匹配的路径, 并限制返回内容大小.

选择原因:

- 用户需要查看 Codex CLI 最近失败原因.
- Settings 适合承载横跨 image generation 和 metadata generation 的诊断入口.
- 路径白名单避免任意文件读取风险.

## Risks / Trade-offs

- [Risk] Codex 输出可能包含解释文字或 Markdown fence. → Mitigation: title/description 做文本归一化, schema prompt 提取并解析 JSON object, 无法解析时返回 recoverable error.
- [Risk] Codex CLI 不存在或未登录会导致 regenerate 失败. → Mitigation: 保留当前草稿, 展示错误和 log path, 不影响 accept 已有草稿.
- [Risk] 字段生成耗时较长造成 UI 状态不清晰. → Mitigation: 使用字段级 loading 和禁用状态, 其他字段仍可编辑.
- [Risk] 用户切换 suggestion 后旧请求返回覆盖新表单. → Mitigation: 前端响应处理检查 `suggestionId` 是否仍匹配当前 form.
- [Risk] 日志读取存在路径安全风险. → Mitigation: 只允许 temp directory 下 app-owned log pattern, 且限制读取大小.
- [Risk] 生成能力范围膨胀到 tags/category 或全自动 metadata. → Mitigation: 本 change 只覆盖 title, description 和 JSON Schema Prompt.

## Migration Plan

该变更不需要数据迁移. 现有 `metadata_suggestions`, canonical asset metadata, asset version 和 generation event schema 均保持不变.

实施顺序:

1. 增加后端 Codex metadata generator 和 parser tests.
2. 增加 Tauri commands: `generate_review_field`, `list_app_logs`, `read_app_log`.
3. 更新 Review UI 字段级 loading 和 regenerate flow.
4. 更新 Settings Logs 模块.
5. 补充 frontend state tests 和手动 smoke 验证.

Rollback 策略:

- 如果 Codex metadata generation 不稳定, 可以保留后端 command 但让 UI 暂时回退到本地 helper fallback.
- 因为没有 schema migration 和持久化副作用, rollback 不需要修复已有 library 数据.

## Open Questions

- 暂无阻塞性开放问题. 日志 preview 的具体字节上限在实现中选择保守默认值即可.
