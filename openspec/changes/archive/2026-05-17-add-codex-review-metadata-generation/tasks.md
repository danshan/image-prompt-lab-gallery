## 1. Backend Metadata Generation

- [x] 1.1 新增 Codex metadata generation 类型, 支持构造 `codex exec --json` command, temp log path 和字段级 prompt.
- [x] 1.2 实现 title 和 description 输出解析与校验, 空结果或明显异常输出返回可恢复错误.
- [x] 1.3 实现 JSON Schema Prompt JSON object 提取, `serde_json` 解析和 pretty-print.
- [x] 1.4 为 metadata generator 添加 Rust 单元测试, 覆盖 command construction, 文本解析, JSON 提取和 invalid JSON error.

## 2. Tauri Commands And Log Access

- [x] 2.1 新增 `generate_review_field` Tauri command, 输入 Review 上下文并返回 `{ field, value, logPath }`.
- [x] 2.2 确保 `generate_review_field` 不修改 `metadata_suggestions`, canonical asset metadata, asset versions 或 generation events.
- [x] 2.3 新增 `list_app_logs` Tauri command, 返回最近 `imglab-codex-cli-*.log` 和 `imglab-codex-metadata-*.log`.
- [x] 2.4 新增 `read_app_log` Tauri command, 只允许读取 temp directory 下 app-owned log pattern 并限制内容大小.
- [x] 2.5 添加 Rust 测试覆盖日志归类, 日志排序, 允许路径读取和任意路径拒绝.

## 3. Review UI Integration

- [x] 3.1 为 Review form 增加字段级 generation state, 支持 title, description 和 schemaPrompt 独立 loading.
- [x] 3.2 将真实 Tauri runtime 下的 `Regenerate` 改为调用 `generate_review_field`, preview mode 保留本地 fallback helper.
- [x] 3.3 在字段生成期间只禁用当前字段和按钮, 其他字段保持可编辑.
- [x] 3.4 实现 stale response guard, 确保切换 suggestion 后旧响应不会覆盖当前表单.
- [x] 3.5 生成失败时保留当前 draft, 清理 loading, 并通过现有 recoverable error 展示错误信息.

## 4. Settings Logs UI

- [x] 4.1 在 Settings view 中新增 Logs 模块, 展示最近日志列表和 empty state.
- [x] 4.2 实现 Logs refresh loading, 调用 `list_app_logs` 并刷新列表.
- [x] 4.3 实现选中日志后调用 `read_app_log` 并展示内容预览.
- [x] 4.4 为日志列表和内容预览补充必要样式, 保持当前 Settings 信息密度和布局风格.

## 5. Tests And Verification

- [x] 5.1 更新 frontend state tests, 覆盖字段级 loading, 成功更新单字段, 失败保留 draft 和 stale response guard.
- [x] 5.2 运行 `cargo test --offline -p imglab-provider-codex -p imglab-core`.
- [x] 5.3 运行 desktop 前端测试命令.
- [x] 5.4 手动 smoke: 打开真实 library, 在 Review 中分别重新生成 title, description 和 JSON Schema Prompt, 验证 loading, 中文输出, JSON 格式和 accept 后才写入 canonical metadata.
- [x] 5.5 手动 smoke: 打开 Settings Logs, 刷新最近日志, 选择 metadata generation 日志并确认内容预览可读.
