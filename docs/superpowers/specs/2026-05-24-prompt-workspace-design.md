# Prompt Workspace Design

Date: 2026-05-24

## Summary

本设计把 Prompt 从 `generation_events.prompt` 中的事实字段提升为可管理的一等资产. 核心结论:

- 新增 `Prompt Workspace`, 与 `Gallery`, `Albums`, `Review`, `Queue`, `Settings` 同级.
- Prompt 的主身份是 `Prompt document`, 由 `prompt_id` 标识, 拥有 draft, notes, status 和多个 immutable versions.
- `prompt_versions` 保存可复现 generation source snapshot, 包含 prompt body, negative prompt, style prompt, template variables, default values, parameter preset 和 notes.
- `generation_events` 继续保存 rendered prompt snapshot 和 negative prompt snapshot, 同时新增 nullable `prompt_version_id`.
- Prompt version 和 asset version 不互相拥有. 它们通过 generation event 建立历史关系.
- 旧 library 迁移不批量回填 prompt documents. 只有用户执行 `Save as Prompt` 或从 Prompt Workspace 发起 generation 后, 才创建 prompt lineage link.

## Context

当前系统能追溯 generation prompt, 但 prompt 仍主要是 generation event 的字段. 这满足事实审计, 但不能支撑 "管理 prompt 实验" 的产品定位.

现有事实:

- `generation_events` 保存 provider, model, operation, prompt, negative prompt, input version, output version, parameters, raw request/response 和 status.
- `asset_versions` 通过 `generation_event_id` 追溯某个输出版本的生成来源.
- Gallery/Search 可以按 generation prompt 查询 asset.
- Review-first metadata 语义已经明确: title, description, tags, category, schema prompt 不应由生成或 suggestion 直接写入 canonical asset metadata.
- Asset-level version lineage 已经有独立 identity, 包括内部 UUID 和用户可见 `v1`, `v2`.

当前缺口:

- Prompt 不能作为独立对象被命名, 编辑, 版本化, 复用或加 notes.
- Negative prompt, style prompt, template variables 和 parameter preset 没有稳定 ownership.
- 从 prompt 发起 generation 与从 quick composer 发起 generation 没有语义区分.
- Asset detail 只能看到 prompt snapshot, 不能反查它来自哪个 managed prompt version.

## Goals

- 提供 Prompt Library, 支持创建, 搜索, 编辑 draft, 归档 prompt document.
- 支持 Prompt version history, 每个 version 是 immutable snapshot.
- 支持 template variables, default values 和 run-time values rendering.
- 支持 reusable negative prompt 和 style prompt, 作为 prompt version 的一等字段.
- 支持 parameter preset, 保存 provider/model/operation/parameters 的默认组合.
- 支持 prompt notes, 与 image metadata 分离.
- 支持从 prompt version 发起 generation task.
- 支持 prompt-to-output history, 从 prompt version 查看生成过的 asset/version/task/event.
- 支持从 asset detail 反查 prompt lineage.
- 保持旧 generation event snapshot 可读, 不破坏旧 library.

## Non-Goals

- 不实现 prompt diff viewer.
- 不实现 prompt folders, collections 或 advanced taxonomy.
- 不实现 AI-assisted prompt improvement.
- 不实现 multi-prompt composition graph.
- 不批量回填历史 generation events 为 prompt documents.
- 不引入 cloud shared prompt library 或 multi-user collaboration.
- 不改变 asset metadata review-first 语义.
- 不把 asset input source 固化进 prompt version. Image-to-image 的 source asset/version 仍属于 generation request context.

## Domain Model

### Prompt Document

`prompt_documents` 是 Prompt Workspace 的 aggregate root.

建议字段:

```text
id TEXT PRIMARY KEY
library_id TEXT NOT NULL
name TEXT NOT NULL
kind TEXT NOT NULL
status TEXT NOT NULL
draft_body TEXT NOT NULL
draft_negative_prompt TEXT
draft_style_prompt TEXT
draft_variables_schema_json TEXT NOT NULL
draft_default_values_json TEXT NOT NULL
draft_parameter_preset_json TEXT NOT NULL
notes TEXT
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
archived_at TEXT
```

`kind` 的 MVP 值:

- `draft`: 普通 prompt document.
- `template`: 包含 template variables 的 prompt document.

`status` 的 MVP 值:

- `active`
- `archived`

Draft 是当前可编辑状态. Draft 可以被覆盖, 但不参与历史追溯. 只有保存 version 后, 才能作为 generation source 被稳定引用.

### Prompt Version

`prompt_versions` 保存 immutable snapshot.

建议字段:

```text
id TEXT PRIMARY KEY
prompt_id TEXT NOT NULL
version_number INTEGER NOT NULL
body TEXT NOT NULL
negative_prompt TEXT
style_prompt TEXT
variables_schema_json TEXT NOT NULL
default_values_json TEXT NOT NULL
parameter_preset_json TEXT NOT NULL
notes TEXT
created_at TEXT NOT NULL
FOREIGN KEY(prompt_id) REFERENCES prompt_documents(id)
UNIQUE(prompt_id, version_number)
```

`version_number` 在单个 prompt document 内递增. UI 可以展示为 `p1/v1`, `p1/v2`, 或在 selected prompt context 下展示 `v1`, `v2`.

Prompt version 一旦创建不得修改. 如果用户要调整 body, negative prompt, style prompt, variables 或 preset, 需要先更新 draft, 再保存新 version.

### Template Variables

Template body 使用 `{{variable_name}}` 占位符. MVP 只支持简单变量替换, 不支持 condition, loop 或 expression.

`variables_schema_json` 建议使用稳定 JSON shape:

```json
{
  "variables": [
    {
      "name": "subject",
      "label": "Subject",
      "required": true,
      "defaultValue": "botanical study"
    }
  ]
}
```

Run 前需要校验:

- body 中的变量名必须可解析.
- required variable 必须有 run-time value 或 default value.
- 未声明但出现在 body 中的 variable 应返回 validation error, 不静默生成空文本.
- 用户填写但 schema 未声明的 values 可以忽略或返回 warning. MVP 推荐返回 warning 但不阻塞.

### Parameter Preset

`parameter_preset_json` 保存 prompt version 推荐的 generation defaults.

建议 MVP shape:

```json
{
  "provider": "fake",
  "model": "default",
  "operation": "text_to_image",
  "parameters": {}
}
```

Preset 是默认值, 不是强制值. 从 Prompt Workspace 发起 run 时, 用户可以覆盖 provider/model/operation/parameters. 最终执行仍通过 shared generation planning 做 provider normalization, operation inference 和 capability check.

Image-to-image 的 `input_version_id` 或 `input_file` 不属于 prompt version. 它是每次 run 的 input context, 避免 prompt asset 拥有 image asset lineage.

### Generation Event Link

`generation_events` 新增 nullable `prompt_version_id`.

Generation event 仍保存:

- rendered `prompt` snapshot.
- rendered or selected `negative_prompt` snapshot.
- `parameters_json`.
- provider/model/operation.
- input/output asset version references.
- raw request/response.

新增 link 的作用是历史反查, 不是替代 snapshot. 这样旧库和外部审计仍可只依赖 generation event 本身读到完整事实.

### Asset And Prompt Lineage

Asset lineage:

```text
asset_versions.generation_event_id
  -> generation_events.id
  -> generation_events.prompt_version_id
  -> prompt_versions.id
  -> prompt_documents.id
```

Prompt-to-output history:

```text
prompt_versions.id
  -> generation_events.prompt_version_id
  -> generation_events.output_version_id
  -> asset_versions.id
  -> assets.id
```

不变量:

- Prompt version 不拥有 asset version.
- Asset version 不拥有 prompt version.
- Generation event 是一次 run 的事实连接点.
- 修改 prompt draft 不会 retroactively 改变 generation event 或 asset detail.
- Prompt notes, variables, style prompt 和 preset 不写入 canonical asset metadata.

## Application Boundary

新增 `prompt-workspace` capability, 不把 Prompt 生命周期塞进 `image-generation` 或 `metadata-review`.

建议 core 边界:

- `domain::prompt`: prompt document/version invariants, template validation, version numbering policy.
- `application::use_cases::prompts`: create/update/archive prompt, save version, render prompt run, query prompt history.
- `application::ports::PromptRepository`: prompt persistence port.
- `infrastructure::sqlite::prompts`: SQLite repository implementation.
- `interface_contracts`: runtime-facing prompt DTOs.

Generation use case 只需要接受 optional `prompt_version_id` 和 rendered snapshot. 它不负责更新 prompt document draft, 也不解释 template variables.

Daemon scheduler 继续执行 `image_generation` task. Task input 可以包含 prompt source context:

```json
{
  "promptVersionId": "prompt-version-id",
  "prompt": "rendered prompt snapshot",
  "negativePrompt": "rendered negative prompt snapshot",
  "variables": {
    "subject": "botanical study"
  },
  "provider": "fake",
  "model": "default",
  "parameters": {}
}
```

Commit generation output 时, core persistence 把 `prompt_version_id` 写入 generation event. Idempotent recovery 不得重复创建 output, 也不得改写已提交 event 的 prompt link.

## Desktop UX

### Top-Level Navigation

新增 `Prompts` workspace, 与 `Gallery`, `Albums`, `Review`, `Queue`, `Settings` 同级.

Prompt Workspace 是 prompt 实验资产管理台, 不是 quick generate form 的简单扩展.

### Layout

采用三栏紧凑 desktop layout:

1. Prompt Library list.
2. Prompt Editor / Version.
3. Run and History.

#### Left: Prompt Library

能力:

- Search by prompt name, body, notes.
- Filter by `draft`, `template`, `has variables`, `recently used`, `unused`.
- Row click 切换 selected prompt.
- Archived prompts 默认隐藏, 可通过 filter 显示.

UI 约束:

- Dense list, 固定 row height.
- 使用 stable prompt id 作为 React key.
- 搜索使用 debounced 或 deferred input.
- 列表超过 100 items 后可以引入 virtualization, 但 MVP 可先保留普通 list.

#### Center: Editor And Versions

中心区域是主工作区.

能力:

- 编辑 draft body.
- 编辑 negative prompt.
- 编辑 style prompt.
- 管理 template variables 和 default values.
- 管理 parameter preset.
- 编辑 prompt notes.
- `Save version` 创建 immutable prompt version.
- 查看 version history 并选中某个 version.

交互:

- Draft 与 selected version 明确区分.
- 选中旧 version 后, 用户可以 `Restore to draft`, 但不能直接修改该 version.
- `Save version` 只在 draft 与 latest version 有差异时启用.
- 长 prompt body 使用固定高度 editor, 不撑破 shell layout.

#### Right: Run And History

能力:

- 从 selected prompt version 发起 generation.
- 如果 selected prompt 是 draft 且没有 version, UI 引导先 `Save version`.
- Variables run form 展示 required/default values.
- Provider/model/operation 使用 preset 作为默认值, 允许 run-time override.
- Run history 展示 asset/version/task/event.
- 支持打开 output asset detail 或 task detail.

Run history row 展示:

- Status.
- Output thumbnail or placeholder.
- Asset title or id.
- Asset version label.
- Provider/model.
- Created time.
- Link actions.

### Cross-Workspace Entry Points

Gallery / Inspector:

- 有 `prompt_version_id` 的 generation event 展示 `Prompt lineage` link.
- 点击 link 跳转到 Prompt Workspace, 选中 document 和 version.
- 没有 `prompt_version_id` 的 legacy event 展示 raw prompt snapshot 和 `Save as Prompt`.
- `Save as Prompt` 从 generation event snapshot 创建 prompt document 和 initial version, 但不回写旧 event 的 historical meaning unless explicitly linked.

Queue / Compose:

- 保留 quick prompt 输入.
- 增加 `Use saved prompt` 入口.
- Queue batch composer 可以选择 saved prompt version, 但 MVP 不需要完整嵌入 prompt editor.

### Responsive Behavior

当前 desktop 的一等 compact width 目标是 `960px`.

在 960px:

- Left list 压缩为窄列.
- Center editor 保持主操作宽度.
- Right Run/History 可以变为 tabs 或 collapsible panel.
- 操作按钮使用 icon + tooltip, 避免文本挤压.
- Prompt body, notes 和 JSON preset 必须固定高度或可滚动.

## UI Style Guidelines

使用 Studio Console 的紧凑工作台语言:

- Data-dense, professional, low decoration.
- Grid-based layout.
- Neutral background, functional accent colors.
- No hero layout, no marketing cards, no decorative gradients.
- Real SVG icon actions for save, copy, run, history, external link, archive.
- Row hover, focus visible, keyboard navigation tab order matches visual order.
- Tables/lists must not overflow viewport; narrow layouts use scroll or tabs.

## OpenSpec Delta

建议新增 capability: `prompt-workspace`.

### Requirement: Prompt Library

系统 SHALL 支持在 managed resource library 内创建, 更新, 搜索和归档 prompt documents.

Scenarios:

- 创建 Prompt Document.
- 更新 Prompt Draft.
- 归档 Prompt Document.
- 搜索 Prompt Library.
- Archived prompt 默认不出现在 active list.

### Requirement: Prompt Versioning

系统 SHALL 支持为 prompt document 保存 immutable prompt versions.

Scenarios:

- 保存初始 version.
- 保存新 version 时 version number per prompt 递增.
- 旧 version 不可修改.
- 从旧 version restore to draft 不改变旧 version.

### Requirement: Template Variables

系统 SHALL 支持 `{{variable}}` template variables, default values 和 required validation.

Scenarios:

- Run 前成功 render prompt.
- Required variable 缺失时报错.
- Body 引用未声明 variable 时报错.
- Run-time values 不写回 prompt version, 除非用户显式更新 draft/defaults.

### Requirement: Generation From Prompt

系统 SHALL 支持从 prompt version 发起 image generation task.

Scenarios:

- 从 prompt version enqueue task.
- Task input 包含 rendered prompt snapshot 和 prompt version id.
- Daemon 执行时使用 shared generation planning.
- Generation event 保存 prompt snapshot 和 `prompt_version_id`.

### Requirement: Prompt-To-Output History

系统 SHALL 支持查询 prompt version 的 output history.

Scenarios:

- Prompt detail 展示 output assets/versions/tasks/events.
- History 不包含没有 prompt link 的 legacy events.
- Task recovery 不重复创建 history row.

### Requirement: Asset Prompt Lineage

系统 SHALL 支持从 asset detail 反查 prompt lineage.

Scenarios:

- Linked generation event 展示 prompt document/version link.
- Legacy generation event 仍展示 raw prompt snapshot.
- Legacy prompt snapshot 可以通过 `Save as Prompt` 创建新 prompt document/version.
- Prompt metadata 不进入 canonical asset metadata.

### Requirement: Library Compatibility

系统 SHALL 在 schema migration 后保持旧 library 可读.

Scenarios:

- 旧 generation events 没有 prompt version link 时仍可显示 prompt.
- Migration 不批量创建 prompt documents.
- Existing Gallery/Search prompt behavior 继续可用.

## Verification Plan

Rust core:

- `cargo test -p imglab-core` for prompt repository, schema migration, version numbering, template rendering, generation event prompt link, asset detail prompt lineage.
- Migration tests for old schema without prompt tables and without `generation_events.prompt_version_id`.
- Tests ensure prompt metadata does not update asset canonical metadata.

Daemon/task:

- `cargo test -p imglab-daemon` for prompt-source image task input, scheduler execution, output links and recovery/idempotence.
- Ensure existing image generation tasks without `promptVersionId` still work.

Desktop:

- `npm test --prefix apps/desktop` for Prompt Workspace state, list search, version save behavior, run payload construction, Inspector prompt lineage states.
- `npm run build --prefix apps/desktop`.
- Browser or Playwright visual checks at desktop and 960px compact width before finishing UI implementation.

OpenSpec:

- `openspec validate <change-name> --strict`.
- After implementation and sync/archive, `openspec validate --specs --strict`.

General:

- `git diff --check`.
- `git status --short`.

## Risks And Mitigations

### Prompt Version And Asset Version Confusion

Risk: 用户或代码把 prompt version 误解成 asset version 的 owner.

Mitigation: 只通过 generation event 建立 link. DTO 和 UI 文案使用 `Prompt version` 与 `Asset version` 的完整名, 不共享 ambiguous `versionId`.

### Prompt Metadata Leaks Into Asset Metadata

Risk: notes, style prompt 或 template fields 被当成 image metadata 写入 asset canonical fields.

Mitigation: Prompt bounded context 独立. Asset metadata 仍通过 Review-first workflow. Tests 覆盖 generation from prompt 不改变 canonical asset metadata.

### Schema Upgrade Ambiguity

Risk: 旧 generation events 是否应自动变成 prompt documents 不清楚.

Mitigation: 不自动回填. 旧 event 保持 snapshot-only. 用户通过 `Save as Prompt` 显式提升.

### Task Payload Drift

Risk: Prompt Workspace 自己拼 generation request, 与 CLI / Queue / daemon planning 漂移.

Mitigation: Prompt Workspace 只负责 render prompt source 和选择 defaults. Provider normalization, operation inference, capability check 仍走 shared generation planning.

### UI Scope Creep

Risk: Prompt Workspace 容易扩张成 prompt IDE.

Mitigation: MVP 限定为 Prompt Library, versioning, variables, run, history, asset lineage. Diff, folders, AI improvement 和 composition graph 全部 defer.

## Implementation Slices

1. OpenSpec change: create `prompt-workspace` proposal, design, tasks and delta spec.
2. Core model and schema: prompt tables, migration, repository, domain policies.
3. Prompt use cases: CRUD draft, save version, render run, query history, save legacy snapshot as prompt.
4. Generation integration: optional prompt version id in task/generation/event commit.
5. Desktop data contracts and Tauri commands.
6. Prompt Workspace UI.
7. Cross-workspace links: Inspector lineage and Save as Prompt.
8. Verification, OpenSpec sync/archive, and final cleanup.
