# Design

## Model Boundary

Prompt 的主身份是 `Prompt document`, 由 `prompt_id` 标识. Prompt document 拥有 editable draft, notes, kind, status 和多个 immutable prompt versions.

`Prompt version` 是 generation source snapshot. 它保存 body, negative prompt, style prompt, template variables schema, default values, parameter preset 和 notes. Prompt version 一旦创建不得修改. 用户需要调整内容时, 先更新 draft, 再保存新 version.

Asset version 是图像输出版本. Prompt version 和 asset version 不互相拥有, 只通过 generation event 建立历史关系.

## Persistence

新增 `prompt_documents` 和 `prompt_versions` 表. `prompt_documents` 保存 editable draft, notes, kind, status, created timestamp, updated timestamp 和 archived timestamp. `prompt_versions` 保存 immutable snapshot, 并在单个 prompt document 内使用递增 `version_number`.

`generation_events` 继续保存 rendered prompt snapshot 和 negative prompt snapshot, 并新增 nullable `prompt_version_id`. Snapshot 是审计事实, link 是 managed prompt lineage.

旧 library migration 只创建新表和 nullable column, 不批量回填 prompt documents. 没有 prompt link 的旧 event 仍保持可读.

## Template Rendering

MVP 支持 `{{variable_name}}` 简单替换, 不支持 condition, loop 或 expression.

Run 前需要校验:

- body 中引用的 variable 必须已声明.
- required variable 必须有 run-time value 或 default value.
- body 中引用未声明 variable 时返回 validation error, 不静默生成空文本.
- 用户提供但 schema 未声明的 values 可以返回 warning, 但 MVP 不阻塞 generation.

Run-time values 只属于本次 run context, 不写回 prompt version.

## Parameter Preset

`parameter_preset_json` 保存 prompt version 推荐的 generation defaults, 包括 provider, model, operation 和 provider parameters. Preset 是默认值, 不是强制值. 用户从 Prompt Workspace 发起 run 时可以覆盖 provider, model, operation 和 parameters.

Image-to-image 的 `input_version_id` 或 `input_file` 不属于 prompt version. 它是每次 run 的 input context, 避免 prompt asset 拥有 image asset lineage.

## Generation Integration

Prompt Workspace 从 selected prompt version render run payload. Daemon 继续执行 `image_generation` task. Task input 可以携带 prompt version id, rendered prompt snapshot, rendered negative prompt snapshot, variables, provider, model 和 parameters.

Generation commit 写入 prompt snapshot 和 prompt version link. Idempotent recovery 不得重复创建 output, 也不得改写已提交 event 的 prompt link.

Generation use case 只接受 optional `prompt_version_id` 和 rendered snapshot. 它不负责更新 prompt document draft, 也不解释 template variables.

## Desktop UX

新增 top-level `Prompts` workspace, 与 Gallery, Albums, Review, Queue, Settings 同级. Prompt Workspace 是 prompt 实验资产管理台, 不是 quick generate form 的简单扩展.

布局采用三栏紧凑 desktop layout:

- Prompt Library list.
- Prompt Editor / Version.
- Run and History.

Gallery / Inspector 展示 linked prompt lineage. 对 legacy generation snapshot, Inspector 保留 raw prompt snapshot, 并提供 Save as Prompt.

## Compatibility

旧 generation events 没有 `prompt_version_id` 时仍显示 raw prompt snapshot. Gallery/Search 按 generation prompt 查询 asset 的行为继续可用.

Prompt notes, variables, style prompt 和 preset 不写入 canonical asset metadata, 保持 metadata review-first 语义.
