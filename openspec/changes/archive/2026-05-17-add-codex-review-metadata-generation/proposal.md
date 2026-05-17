## Why

Review Inbox 当前对 title, description 和 JSON Schema Prompt 的重新生成仍由前端确定性 helper 派生, 输出质量有限, 且无法满足通过 Codex CLI 生成 Review metadata 的产品要求.

现在需要把这三个字段的生成能力放到后端 Codex CLI 边界中, 同时在页面上提供明确 loading 反馈, 并让 Settings 能查看最近的生成日志, 便于定位 Codex CLI 失败原因.

## What Changes

- 新增 Codex CLI metadata generation 边界, 专门生成 Review 字段文本, 不复用图片生成 provider.
- Review Inbox 的 title, description 和 JSON Schema Prompt 重新生成通过 Tauri command 调用 Codex CLI.
- title 和 description 生成结果以简体中文为主.
- JSON Schema Prompt 生成结果必须能被解析为合法 JSON object, 并以稳定 pretty JSON 写回 Review 表单草稿.
- Review UI 增加字段级 loading 状态, 请求失败时保留当前草稿.
- Settings 增加 Logs 模块, 支持查看最近 app 生成日志, 刷新列表, 以及读取单个日志内容.
- 不改变 review-first 规则: 接受 suggestion 前, 生成结果只更新本地 Review form draft, 不写 canonical asset metadata.

## Capabilities

### New Capabilities

- `app-logs`: Settings 中查看 app 生成日志的能力, 包括最近日志列表, 刷新和日志内容预览.

### Modified Capabilities

- `metadata-review`: Review 字段重新生成改为通过 Codex CLI 生成 title, description 和 JSON Schema Prompt, 并保持 review-first.
- `desktop-workbench`: Review Inbox 增加字段级 loading 反馈, Settings 增加 Logs 模块.

## Impact

- Rust backend:
  - `apps/desktop/src-tauri/src/lib.rs`
  - 可能新增 Codex metadata generator 模块或 helper.
- Desktop frontend:
  - `apps/desktop/src/main.tsx`
  - `apps/desktop/src/workbench-state.ts`
  - `apps/desktop/src/styles.css`
- Tests:
  - Rust command / parser tests.
  - Frontend state tests.
- External dependency:
  - 继续依赖本机 `codex` CLI 和当前登录态, 不新增 native OpenAI / Grok API client.
