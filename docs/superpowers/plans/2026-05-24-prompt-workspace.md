# Prompt Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Prompt Workspace so prompts become managed first-class assets with drafts, immutable versions, template variables, parameter presets, generation runs, and asset prompt lineage.

**Architecture:** Add a new `prompt-workspace` OpenSpec capability, then implement a focused prompt bounded context in `imglab-core`. Generation events remain the immutable run record and connect prompt versions to asset versions, while desktop adds a new `Prompts` workspace and cross-links from Inspector.

**Tech Stack:** Rust workspace (`imglab-core`, `imglab-daemon`, Tauri backend), SQLite migrations, React 19 + TypeScript desktop, OpenSpec.

---

## File Structure

Core prompt domain and application:

- Create `crates/imglab-core/src/domain/prompt/model.rs`: prompt IDs, document/version models, template variable models, status/kind validation.
- Create `crates/imglab-core/src/domain/prompt/policies.rs`: version numbering, immutable version rules, template rendering and validation.
- Create `crates/imglab-core/src/domain/prompt/mod.rs`: prompt domain exports.
- Modify `crates/imglab-core/src/domain/mod.rs`: export `prompt`.
- Modify `crates/imglab-core/src/application/ports/repositories.rs`: add `PromptRepository`.
- Create `crates/imglab-core/src/application/use_cases/prompts.rs`: prompt CRUD, save version, render run, history, save legacy snapshot.
- Modify `crates/imglab-core/src/application/use_cases/mod.rs`: export `prompts`.
- Modify `crates/imglab-core/src/application/facade.rs`: expose prompt use case facade methods.
- Modify `crates/imglab-core/src/dto.rs`: add prompt DTOs and optional `prompt_version_id` on generation DTOs.
- Modify `crates/imglab-core/src/interface_contracts/dto.rs`: continue re-exporting new DTOs.

SQLite and read models:

- Create `crates/imglab-core/src/infrastructure/sqlite/prompts.rs`: prompt repository implementation.
- Modify `crates/imglab-core/src/infrastructure/sqlite/mod.rs`: export prompt repository.
- Modify `crates/imglab-core/src/infrastructure/sqlite/schema.rs`: add schema version, prompt tables, `generation_events.prompt_version_id`, indexes.
- Modify `crates/imglab-core/src/infrastructure/composition.rs`: wire prompt repository.
- Modify `crates/imglab-core/src/library/assets.rs` and `crates/imglab-core/src/library/gallery_detail.rs`: load prompt lineage for asset detail.
- Modify `crates/imglab-core/src/library/generation.rs` or current generation repository implementation: persist prompt version link.
- Modify `crates/imglab-core/src/library/tests.rs`: add migration, prompt, generation, lineage tests.

Daemon, Tauri, desktop:

- Modify `crates/imglab-daemon/src/runtime.rs`: add optional prompt source fields to image generation task input.
- Modify `crates/imglab-daemon/src/scheduler.rs`: pass prompt version link into prepared generation request.
- Modify `apps/desktop/src-tauri/src/views.rs`: add prompt view DTOs and prompt lineage fields.
- Modify `apps/desktop/src-tauri/src/view_mappers.rs`: map prompt DTOs.
- Create `apps/desktop/src-tauri/src/commands/prompts.rs`: prompt CRUD/run commands.
- Modify `apps/desktop/src-tauri/src/lib.rs`: register prompt commands.
- Modify `apps/desktop/src/app/types.ts`: add prompt types and extend `GenerationEvent` / `AssetDetail`.
- Modify `apps/desktop/src/studio-navigation.tsx`: add `Prompts`.
- Modify `apps/desktop/src/app/types.ts`: add `"prompts"` to `View`.
- Create `apps/desktop/src/app/workflows/prompts/state.ts`: Prompt Workspace state.
- Create `apps/desktop/src/app/workflows/prompts/controller.ts`: Prompt Workspace controller.
- Create `apps/desktop/src/app/workflows/prompts/index.ts`: prompt workflow exports.
- Create `apps/desktop/src/app/screens/workflows/prompts.tsx`: Prompt Workspace UI.
- Modify `apps/desktop/src/app/StudioAppController.tsx`: wire prompt workflow and cross-links.
- Modify `apps/desktop/src/app/screens/workflows/inspector.tsx`: show prompt lineage and `Save as Prompt`.
- Modify `apps/desktop/src/styles.css`: compact prompt workspace layout.

OpenSpec:

- Create `openspec/changes/add-prompt-workspace/proposal.md`.
- Create `openspec/changes/add-prompt-workspace/design.md`.
- Create `openspec/changes/add-prompt-workspace/tasks.md`.
- Create `openspec/changes/add-prompt-workspace/specs/prompt-workspace/spec.md`.

## Task 1: Create OpenSpec Change Artifacts

**Files:**
- Create: `openspec/changes/add-prompt-workspace/proposal.md`
- Create: `openspec/changes/add-prompt-workspace/design.md`
- Create: `openspec/changes/add-prompt-workspace/tasks.md`
- Create: `openspec/changes/add-prompt-workspace/specs/prompt-workspace/spec.md`
- Read: `docs/superpowers/specs/2026-05-24-prompt-workspace-design.md`

- [ ] **Step 1: Scaffold the OpenSpec change**

Run:

```bash
openspec new change "add-prompt-workspace"
```

Expected: creates `openspec/changes/add-prompt-workspace/`.

- [ ] **Step 2: Inspect required artifacts**

Run:

```bash
openspec status --change "add-prompt-workspace" --json
```

Expected: JSON shows proposal, design, tasks, and delta spec requirements. Ignore PostHog or `edge.openspec.dev` telemetry flush noise if the command exits successfully and local files exist.

- [ ] **Step 3: Write proposal**

Create `openspec/changes/add-prompt-workspace/proposal.md`:

```markdown
# Add Prompt Workspace

## Summary

把 prompt 从 generation event 的字段提升为可管理的一等资产, 新增 Prompt Workspace, Prompt Library, immutable prompt versions, template variables, parameter presets, prompt notes, generation from prompt, prompt-to-output history, 以及 asset detail 的 prompt lineage.

## Motivation

当前系统可以追溯 generation prompt, 但 prompt 仍只是 generation event 的事实字段. 这无法支撑 "管理 prompt 实验" 的核心产品定位. 用户需要管理 prompt draft, 保存可复现版本, 从 prompt 发起 generation, 并从 output asset 反查 prompt 来源.

## Scope

- 新增 prompt document 和 prompt version 持久化模型.
- 新增 Prompt Workspace 桌面 workflow.
- 支持 template variables 和 run-time rendering.
- 支持 parameter preset 作为 generation defaults.
- generation event 新增 nullable prompt version link, 同时继续保存 prompt snapshot.
- asset detail 支持 prompt lineage link.
- 旧 generation event 继续可读, migration 不批量创建 prompt documents.

## Non-Goals

- 不实现 prompt diff viewer.
- 不实现 prompt folders 或 collections.
- 不实现 AI-assisted prompt improvement.
- 不实现 multi-prompt composition graph.
- 不批量回填历史 generation events 为 prompt documents.
- 不改变 asset metadata review-first 语义.
```

- [ ] **Step 4: Write design artifact**

Create `openspec/changes/add-prompt-workspace/design.md` by condensing `docs/superpowers/specs/2026-05-24-prompt-workspace-design.md`. Keep the same decisions:

```markdown
# Design

## Model Boundary

Prompt 的主身份是 `Prompt document`, 由 `prompt_id` 标识. `Prompt version` 是 immutable generation source snapshot. Asset version 是图像输出版本. 两者不互相拥有, 只通过 generation event 建立历史关系.

`generation_events` 继续保存 rendered prompt snapshot 和 negative prompt snapshot, 并新增 nullable `prompt_version_id`. Snapshot 是审计事实, link 是 managed prompt lineage.

## Persistence

新增 `prompt_documents` 和 `prompt_versions` 表. `prompt_documents` 保存 editable draft, notes, kind, status 和 updated timestamp. `prompt_versions` 保存 immutable body, negative prompt, style prompt, variables schema, default values, parameter preset 和 notes.

旧 library migration 只创建新表和 nullable column, 不批量回填 prompt documents.

## Template Rendering

MVP 支持 `{{variable_name}}` 简单替换. Required variable 缺失时返回 validation error. Body 中引用未声明 variable 时返回 validation error. Run-time values 不写回 prompt version.

## Generation Integration

Prompt Workspace 从 selected prompt version render run payload. Daemon 继续执行 `image_generation` task. Generation commit 写入 prompt snapshot 和 prompt version link. Idempotent recovery 不改写已提交 prompt link.

## Desktop UX

新增 top-level `Prompts` workspace. 布局为 Prompt Library list, Editor/Version, Run/History 三栏. Gallery / Inspector 展示 prompt lineage link 或 legacy `Save as Prompt`.

## Compatibility

旧 generation events 没有 `prompt_version_id` 时仍显示 raw prompt snapshot. Gallery/Search prompt behavior 继续可用.
```

- [ ] **Step 5: Write delta spec**

Create `openspec/changes/add-prompt-workspace/specs/prompt-workspace/spec.md`:

```markdown
## ADDED Requirements

### Requirement: Prompt Library

系统 SHALL 支持在 managed resource library 内创建, 更新, 搜索和归档 prompt documents.

#### Scenario: 创建 Prompt Document

- **WHEN** 用户创建 prompt document 并提供 name 和 draft body
- **THEN** 系统保存 active prompt document
- **AND** draft body 尚不创建 prompt version

#### Scenario: 更新 Prompt Draft

- **WHEN** 用户修改 prompt document 的 body, negative prompt, style prompt, variables, preset 或 notes
- **THEN** 系统只更新 prompt document draft
- **AND** 不修改任何已保存 prompt version

#### Scenario: 搜索 Prompt Library

- **WHEN** 用户按文本搜索 Prompt Library
- **THEN** 系统按 prompt name, draft body 和 notes 返回匹配 active prompts

#### Scenario: 归档 Prompt Document

- **WHEN** 用户归档 prompt document
- **THEN** 系统将 status 设置为 archived
- **AND** 默认 Prompt Library list 不再显示该 prompt

### Requirement: Prompt Versioning

系统 SHALL 支持为 prompt document 保存 immutable prompt versions.

#### Scenario: 保存初始 Version

- **WHEN** 用户对没有 version 的 prompt document 执行 Save version
- **THEN** 系统创建 version number 为 1 的 prompt version

#### Scenario: 保存新 Version

- **WHEN** 用户对已有 version 的 prompt document 执行 Save version
- **THEN** 系统使用该 prompt document 内的下一个 version number 创建 prompt version

#### Scenario: Prompt Version 不可修改

- **WHEN** prompt version 已经创建
- **THEN** 系统不得通过 draft 更新修改该 version snapshot

#### Scenario: Restore Version To Draft

- **WHEN** 用户从旧 prompt version restore to draft
- **THEN** 系统用该 version snapshot 覆盖 prompt document draft
- **AND** 不修改旧 prompt version

### Requirement: Template Variables

系统 SHALL 支持 `{{variable}}` template variables, default values 和 required validation.

#### Scenario: Render Prompt 成功

- **WHEN** prompt version body 引用声明过的 variables 且 required values 均可解析
- **THEN** 系统生成 rendered prompt snapshot

#### Scenario: Required Variable 缺失

- **WHEN** required variable 没有 run-time value 且没有 default value
- **THEN** 系统返回 validation error
- **AND** 不创建 generation task

#### Scenario: Body 引用未声明 Variable

- **WHEN** prompt body 包含未在 schema 中声明的 variable
- **THEN** 系统返回 validation error
- **AND** 不创建 generation task

### Requirement: Generation From Prompt

系统 SHALL 支持从 prompt version 发起 image generation task.

#### Scenario: 从 Prompt Version Enqueue Task

- **WHEN** 用户从 prompt version 发起 generation
- **THEN** task input 包含 prompt version id, rendered prompt snapshot, variables, provider, model 和 parameters

#### Scenario: Generation Event 保存 Prompt Link

- **WHEN** prompt-sourced image generation task completed
- **THEN** generation event 保存 prompt snapshot
- **AND** generation event 保存 prompt version link
- **AND** output asset version 通过 generation event 可反查 prompt version

### Requirement: Prompt-To-Output History

系统 SHALL 支持查询 prompt version 的 output history.

#### Scenario: Prompt Detail 展示 Output History

- **WHEN** 用户查看 prompt version
- **THEN** 系统展示该 version 生成过的 asset, version, task 和 generation event

#### Scenario: Legacy Events 不进入 Prompt History

- **WHEN** generation event 没有 prompt version link
- **THEN** prompt output history 不包含该 event

### Requirement: Asset Prompt Lineage

系统 SHALL 支持从 asset detail 反查 prompt lineage.

#### Scenario: Linked Prompt Lineage

- **WHEN** asset version 的 generation event 有 prompt version link
- **THEN** asset detail 展示 prompt document 和 prompt version link

#### Scenario: Legacy Prompt Snapshot

- **WHEN** asset version 的 generation event 没有 prompt version link
- **THEN** asset detail 仍展示 raw prompt snapshot
- **AND** 用户可以执行 Save as Prompt

#### Scenario: Prompt Metadata 不进入 Asset Metadata

- **WHEN** 用户从 prompt version 发起 generation
- **THEN** prompt notes, template variables, style prompt 和 preset 不写入 canonical asset metadata

### Requirement: Library Compatibility

系统 SHALL 在 schema migration 后保持旧 library 可读.

#### Scenario: 旧 Library Migration

- **WHEN** 系统打开旧 schema library
- **THEN** migration 创建 prompt tables 和 generation event prompt link column
- **AND** 不批量创建 prompt documents

#### Scenario: Existing Prompt Search Still Works

- **WHEN** 用户在 Gallery/Search 搜索 legacy generation prompt 文本
- **THEN** 系统仍返回匹配 asset
```

- [ ] **Step 6: Write tasks artifact**

Create `openspec/changes/add-prompt-workspace/tasks.md`:

```markdown
# Tasks

- [ ] 1. Add Prompt Workspace OpenSpec delta and validate the change.
- [ ] 2. Add core prompt domain models, policies, DTOs, and tests.
- [ ] 3. Add SQLite prompt schema migration and repository tests.
- [ ] 4. Add prompt application use cases for draft CRUD, versioning, rendering, history, and Save as Prompt.
- [ ] 5. Integrate prompt version links into generation task input and generation event persistence.
- [ ] 6. Expose prompt commands and prompt lineage DTOs through Tauri.
- [ ] 7. Add desktop Prompt Workspace state, controller, and UI.
- [ ] 8. Add Inspector prompt lineage and legacy Save as Prompt workflow.
- [ ] 9. Run Rust, desktop, OpenSpec, and git verification.
```

- [ ] **Step 7: Validate OpenSpec change**

Run:

```bash
openspec validate add-prompt-workspace --strict
```

Expected: validation passes. If telemetry flush prints network errors after local validation, treat them as non-blocking only if command exit status is 0.

- [ ] **Step 8: Commit OpenSpec artifacts**

Run:

```bash
git add openspec/changes/add-prompt-workspace
git commit -m "spec: add prompt workspace change"
```

Expected: commit contains only OpenSpec artifacts.

## Task 2: Add Core Prompt Domain And DTOs

**Files:**
- Create: `crates/imglab-core/src/domain/prompt/model.rs`
- Create: `crates/imglab-core/src/domain/prompt/policies.rs`
- Create: `crates/imglab-core/src/domain/prompt/mod.rs`
- Modify: `crates/imglab-core/src/domain/mod.rs`
- Modify: `crates/imglab-core/src/dto.rs`
- Test: `crates/imglab-core/src/library/tests.rs`

- [ ] **Step 1: Write failing domain policy tests**

Add tests in `crates/imglab-core/src/library/tests.rs`:

```rust
#[test]
fn prompt_template_rendering_requires_declared_values() {
    use crate::domain::prompt::{render_prompt_template, PromptTemplateVariable};

    let variables = vec![PromptTemplateVariable {
        name: "subject".to_string(),
        label: Some("Subject".to_string()),
        required: true,
        default_value: None,
    }];
    let values = serde_json::json!({});

    let error = render_prompt_template("A {{subject}} study", &variables, &values)
        .expect_err("missing required variable should fail");

    assert!(error.to_string().contains("subject"));
}

#[test]
fn prompt_template_rendering_rejects_undeclared_variables() {
    use crate::domain::prompt::render_prompt_template;

    let error = render_prompt_template("A {{subject}} study", &[], &serde_json::json!({}))
        .expect_err("undeclared variable should fail");

    assert!(error.to_string().contains("subject"));
}

#[test]
fn prompt_template_rendering_uses_runtime_values() {
    use crate::domain::prompt::{render_prompt_template, PromptTemplateVariable};

    let variables = vec![PromptTemplateVariable {
        name: "subject".to_string(),
        label: Some("Subject".to_string()),
        required: true,
        default_value: Some("orchid".to_string()),
    }];
    let rendered = render_prompt_template(
        "A {{subject}} study",
        &variables,
        &serde_json::json!({ "subject": "fern" }),
    )
    .expect("render prompt");

    assert_eq!(rendered, "A fern study");
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test -p imglab-core prompt_template_rendering -- --nocapture
```

Expected: compile fails because `domain::prompt` does not exist.

- [ ] **Step 3: Add prompt domain module**

Create `crates/imglab-core/src/domain/prompt/mod.rs`:

```rust
mod model;
mod policies;

pub use model::*;
pub use policies::*;
```

Create `crates/imglab-core/src/domain/prompt/model.rs`:

```rust
use crate::domain::shared::identity::{define_id_type, new_id};
use serde::{Deserialize, Serialize};

define_id_type!(PromptId);
define_id_type!(PromptVersionId);

impl PromptId {
    pub fn new() -> Self {
        Self(new_id())
    }
}

impl PromptVersionId {
    pub fn new() -> Self {
        Self(new_id())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptDocumentKind {
    Draft,
    Template,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptDocumentStatus {
    Active,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTemplateVariable {
    pub name: String,
    pub label: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptDocumentSummary {
    pub id: PromptId,
    pub name: String,
    pub kind: PromptDocumentKind,
    pub status: PromptDocumentStatus,
    pub draft_body: String,
    pub draft_negative_prompt: Option<String>,
    pub draft_style_prompt: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVersionSummary {
    pub id: PromptVersionId,
    pub prompt_id: PromptId,
    pub version_number: u32,
    pub version_name: String,
    pub body: String,
    pub negative_prompt: Option<String>,
    pub style_prompt: Option<String>,
    pub variables_schema_json: String,
    pub default_values_json: String,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
    pub created_at: String,
}
```

Create `crates/imglab-core/src/domain/prompt/policies.rs`:

```rust
use crate::{DomainError, DomainResult};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::PromptTemplateVariable;

pub fn next_prompt_version_number(current_max: Option<u32>) -> u32 {
    current_max.unwrap_or(0) + 1
}

pub fn prompt_version_name(version_number: u32) -> String {
    format!("v{version_number}")
}

pub fn render_prompt_template(
    body: &str,
    variables: &[PromptTemplateVariable],
    values: &Value,
) -> DomainResult<String> {
    let declared = variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable))
        .collect::<BTreeMap<_, _>>();
    let referenced = referenced_variables(body);

    for name in &referenced {
        if !declared.contains_key(name.as_str()) {
            return Err(DomainError::InvalidGenerationParameters {
                message: format!("prompt variable `{name}` is not declared"),
            });
        }
    }

    let mut rendered = body.to_string();
    for variable in variables {
        let value = values
            .get(&variable.name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| variable.default_value.clone());

        if variable.required && value.as_ref().is_none_or(|value| value.trim().is_empty()) {
            return Err(DomainError::InvalidGenerationParameters {
                message: format!("prompt variable `{}` is required", variable.name),
            });
        }

        if let Some(value) = value {
            rendered = rendered.replace(&format!("{{{{{}}}}}", variable.name), &value);
        }
    }

    Ok(rendered)
}

fn referenced_variables(body: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut remaining = body;
    while let Some(start) = remaining.find("{{") {
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("}}") else {
            break;
        };
        let name = after_start[..end].trim();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
        remaining = &after_start[end + 2..];
    }
    names
}
```

Modify `crates/imglab-core/src/domain/mod.rs`:

```rust
pub mod album;
pub mod asset;
pub mod generation;
pub mod library;
pub mod metadata_review;
pub mod prompt;
pub mod shared;
pub mod task;
```

- [ ] **Step 4: Add DTOs**

Modify `crates/imglab-core/src/dto.rs` to add prompt structs near other runtime DTOs:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptDocumentView {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub draft_body: String,
    pub draft_negative_prompt: Option<String>,
    pub draft_style_prompt: Option<String>,
    pub variables_schema_json: String,
    pub default_values_json: String,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
    pub latest_version_id: Option<String>,
    pub latest_version_number: Option<u32>,
    pub latest_version_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVersionView {
    pub id: String,
    pub prompt_id: String,
    pub version_number: u32,
    pub version_name: String,
    pub body: String,
    pub negative_prompt: Option<String>,
    pub style_prompt: Option<String>,
    pub variables_schema_json: String,
    pub default_values_json: String,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptLineageView {
    pub prompt_id: String,
    pub prompt_name: String,
    pub prompt_version_id: String,
    pub prompt_version_number: u32,
    pub prompt_version_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptOutputHistoryItem {
    pub generation_event_id: String,
    pub asset_id: Option<String>,
    pub output_version_id: Option<String>,
    pub task_id: Option<String>,
    pub provider: String,
    pub provider_model: String,
    pub status: String,
    pub prompt_snapshot: String,
    pub created_at: String,
}
```

Also extend generation DTOs with optional prompt version links:

```rust
pub prompt_version_id: Option<String>,
```

Add that field to `GenerationParameters`, `CreateGenerationEventRequest`, and `GenerationEventSummary`. If `GenerationParameters` currently uses typed IDs for input versions, use `Option<PromptVersionId>` instead of `Option<String>` and serialize at boundaries.

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p imglab-core prompt_template_rendering -- --nocapture
```

Expected: the three prompt template tests pass.

- [ ] **Step 6: Commit**

Run:

```bash
git add crates/imglab-core/src/domain crates/imglab-core/src/dto.rs crates/imglab-core/src/library/tests.rs
git commit -m "feat: add prompt domain model"
```

Expected: commit contains domain models, policy tests, and DTO additions.

## Task 3: Add SQLite Prompt Schema And Repository

**Files:**
- Modify: `crates/imglab-core/src/infrastructure/sqlite/schema.rs`
- Create: `crates/imglab-core/src/infrastructure/sqlite/prompts.rs`
- Modify: `crates/imglab-core/src/infrastructure/sqlite/mod.rs`
- Modify: `crates/imglab-core/src/application/ports/repositories.rs`
- Test: `crates/imglab-core/src/library/tests.rs`

- [ ] **Step 1: Write failing migration and repository tests**

Add tests in `crates/imglab-core/src/library/tests.rs`:

```rust
#[test]
fn migration_adds_prompt_workspace_schema_without_backfilling_documents() {
    let root = test_root("prompt-workspace-migration");
    let registry = test_root("prompt-workspace-migration-registry").join("registry.sqlite");
    let service = LibraryService::new_with_registry_path(registry);
    service.init_library(&root, "Prompt Migration").expect("init library");

    let connection = rusqlite::Connection::open(root.join("library.sqlite")).expect("open sqlite");
    let prompt_table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('prompt_documents', 'prompt_versions')",
            [],
            |row| row.get(0),
        )
        .expect("prompt tables");
    assert_eq!(prompt_table_count, 2);

    let prompt_link_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('generation_events') WHERE name = 'prompt_version_id'",
            [],
            |row| row.get(0),
        )
        .expect("prompt link column");
    assert_eq!(prompt_link_count, 1);

    let prompt_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM prompt_documents", [], |row| row.get(0))
        .expect("prompt count");
    assert_eq!(prompt_count, 0);
}
```

- [ ] **Step 2: Run migration test and verify failure**

Run:

```bash
cargo test -p imglab-core migration_adds_prompt_workspace_schema_without_backfilling_documents -- --nocapture
```

Expected: fails because prompt tables and column do not exist.

- [ ] **Step 3: Add schema migration**

Modify `crates/imglab-core/src/infrastructure/sqlite/schema.rs`:

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 8;
```

Add to the `CREATE TABLE IF NOT EXISTS` batch:

```sql
CREATE TABLE IF NOT EXISTS prompt_documents (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    draft_body TEXT NOT NULL,
    draft_negative_prompt TEXT,
    draft_style_prompt TEXT,
    draft_variables_schema_json TEXT NOT NULL,
    draft_default_values_json TEXT NOT NULL,
    draft_parameter_preset_json TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE TABLE IF NOT EXISTS prompt_versions (
    id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    body TEXT NOT NULL,
    negative_prompt TEXT,
    style_prompt TEXT,
    variables_schema_json TEXT NOT NULL,
    default_values_json TEXT NOT NULL,
    parameter_preset_json TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(prompt_id) REFERENCES prompt_documents(id),
    UNIQUE(prompt_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_prompt_documents_library_status
    ON prompt_documents(library_id, status, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_prompt_versions_prompt_number
    ON prompt_versions(prompt_id, version_number DESC);
```

Add migration guard:

```rust
if !column_exists(connection, "generation_events", "prompt_version_id")? {
    connection
        .execute("ALTER TABLE generation_events ADD COLUMN prompt_version_id TEXT", [])
        .map_err(database_error)?;
}
```

Add index:

```sql
CREATE INDEX IF NOT EXISTS idx_generation_events_prompt_version
    ON generation_events(prompt_version_id, started_at DESC, id DESC);
```

- [ ] **Step 4: Add repository port**

Modify `crates/imglab-core/src/application/ports/repositories.rs`:

```rust
pub trait PromptRepository {
    fn create_prompt_document(
        &self,
        request: crate::CreatePromptDocumentRequest,
    ) -> DomainResult<crate::PromptDocumentView>;

    fn update_prompt_draft(
        &self,
        request: crate::UpdatePromptDraftRequest,
    ) -> DomainResult<crate::PromptDocumentView>;

    fn save_prompt_version(
        &self,
        request: crate::SavePromptVersionRequest,
    ) -> DomainResult<crate::PromptVersionView>;

    fn list_prompt_documents(
        &self,
        request: crate::ListPromptDocumentsRequest,
    ) -> DomainResult<Vec<crate::PromptDocumentView>>;

    fn list_prompt_versions(
        &self,
        request: crate::ListPromptVersionsRequest,
    ) -> DomainResult<Vec<crate::PromptVersionView>>;
}
```

Define the request DTOs in `crates/imglab-core/src/dto.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePromptDocumentRequest {
    pub library_path: PathBuf,
    pub name: String,
    pub draft_body: String,
    pub draft_negative_prompt: Option<String>,
    pub draft_style_prompt: Option<String>,
    pub variables_schema_json: String,
    pub default_values_json: String,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePromptDraftRequest {
    pub library_path: PathBuf,
    pub prompt_id: String,
    pub name: String,
    pub draft_body: String,
    pub draft_negative_prompt: Option<String>,
    pub draft_style_prompt: Option<String>,
    pub variables_schema_json: String,
    pub default_values_json: String,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavePromptVersionRequest {
    pub library_path: PathBuf,
    pub prompt_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPromptDocumentsRequest {
    pub library_path: PathBuf,
    pub query: Option<String>,
    pub include_archived: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPromptVersionsRequest {
    pub library_path: PathBuf,
    pub prompt_id: String,
}
```

- [ ] **Step 5: Implement SQLite repository**

Create `crates/imglab-core/src/infrastructure/sqlite/prompts.rs` with focused SQL functions. Use existing `database_error`, connection helpers, and timestamp helpers from nearby SQLite modules. Implement only CRUD/version/list in this task.

Key insert for saving version:

```rust
let next_version_number = connection.query_row(
    "SELECT COALESCE(MAX(version_number), 0) + 1 FROM prompt_versions WHERE prompt_id = ?1",
    [&request.prompt_id],
    |row| row.get::<_, u32>(0),
)?;
```

Then insert by selecting draft fields from `prompt_documents`:

```sql
INSERT INTO prompt_versions (
    id, prompt_id, version_number, body, negative_prompt, style_prompt,
    variables_schema_json, default_values_json, parameter_preset_json,
    notes, created_at
)
SELECT ?1, id, ?2, draft_body, draft_negative_prompt, draft_style_prompt,
       draft_variables_schema_json, draft_default_values_json,
       draft_parameter_preset_json, notes, ?3
FROM prompt_documents
WHERE id = ?4
```

- [ ] **Step 6: Export repository module**

Modify `crates/imglab-core/src/infrastructure/sqlite/mod.rs`:

```rust
pub mod prompts;
```

- [ ] **Step 7: Run migration test**

Run:

```bash
cargo test -p imglab-core migration_adds_prompt_workspace_schema_without_backfilling_documents -- --nocapture
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add crates/imglab-core/src/infrastructure/sqlite crates/imglab-core/src/application/ports/repositories.rs crates/imglab-core/src/dto.rs crates/imglab-core/src/library/tests.rs
git commit -m "feat: add prompt persistence schema"
```

Expected: commit contains schema, repository, port, DTO request types, and migration test.

## Task 4: Add Prompt Use Cases

**Files:**
- Create: `crates/imglab-core/src/application/use_cases/prompts.rs`
- Modify: `crates/imglab-core/src/application/use_cases/mod.rs`
- Modify: `crates/imglab-core/src/application/facade.rs`
- Test: `crates/imglab-core/src/library/tests.rs`

- [ ] **Step 1: Write use case tests**

Add tests:

```rust
#[test]
fn prompt_document_save_version_keeps_previous_version_immutable() {
    let root = test_root("prompt-version-immutable");
    let registry = test_root("prompt-version-immutable-registry").join("registry.sqlite");
    let app = test_application_with_registry(registry);
    app.library().init_library(&root, "Prompt Versions").expect("init");

    let created = app.prompts().create_prompt_document(CreatePromptDocumentRequest {
        library_path: root.clone(),
        name: "Botanical".to_string(),
        draft_body: "A {{subject}} study".to_string(),
        draft_negative_prompt: Some("blur".to_string()),
        draft_style_prompt: Some("macro".to_string()),
        variables_schema_json: r#"{"variables":[{"name":"subject","required":true,"defaultValue":"orchid"}]}"#.to_string(),
        default_values_json: r#"{"subject":"orchid"}"#.to_string(),
        parameter_preset_json: r#"{"provider":"fake","model":"default","operation":"text_to_image","parameters":{}}"#.to_string(),
        notes: Some("first note".to_string()),
    }).expect("create prompt");

    let version = app.prompts().save_prompt_version(SavePromptVersionRequest {
        library_path: root.clone(),
        prompt_id: created.id.clone(),
    }).expect("save version");

    app.prompts().update_prompt_draft(UpdatePromptDraftRequest {
        library_path: root.clone(),
        prompt_id: created.id.clone(),
        name: "Botanical".to_string(),
        draft_body: "A {{subject}} poster".to_string(),
        draft_negative_prompt: Some("noise".to_string()),
        draft_style_prompt: Some("editorial".to_string()),
        variables_schema_json: r#"{"variables":[{"name":"subject","required":true,"defaultValue":"fern"}]}"#.to_string(),
        default_values_json: r#"{"subject":"fern"}"#.to_string(),
        parameter_preset_json: r#"{"provider":"fake","model":"default","operation":"text_to_image","parameters":{}}"#.to_string(),
        notes: Some("second note".to_string()),
    }).expect("update draft");

    let versions = app.prompts().list_prompt_versions(ListPromptVersionsRequest {
        library_path: root,
        prompt_id: created.id,
    }).expect("versions");

    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].id, version.id);
    assert_eq!(versions[0].body, "A {{subject}} study");
    assert_eq!(versions[0].negative_prompt.as_deref(), Some("blur"));
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test -p imglab-core prompt_document_save_version_keeps_previous_version_immutable -- --nocapture
```

Expected: compile fails until facade/use cases exist.

- [ ] **Step 3: Implement prompt use cases**

Create `crates/imglab-core/src/application/use_cases/prompts.rs`:

```rust
use crate::application::ports::PromptRepository;
use crate::{
    CreatePromptDocumentRequest, DomainResult, ListPromptDocumentsRequest,
    ListPromptVersionsRequest, PromptDocumentView, PromptVersionView, SavePromptVersionRequest,
    UpdatePromptDraftRequest,
};

pub struct PromptWorkspaceUseCase<R> {
    prompts: R,
}

impl<R> PromptWorkspaceUseCase<R> {
    pub fn new(prompts: R) -> Self {
        Self { prompts }
    }
}

impl<R> PromptWorkspaceUseCase<R>
where
    R: PromptRepository,
{
    pub fn create_prompt_document(
        &self,
        request: CreatePromptDocumentRequest,
    ) -> DomainResult<PromptDocumentView> {
        self.prompts.create_prompt_document(request)
    }

    pub fn update_prompt_draft(
        &self,
        request: UpdatePromptDraftRequest,
    ) -> DomainResult<PromptDocumentView> {
        self.prompts.update_prompt_draft(request)
    }

    pub fn save_prompt_version(
        &self,
        request: SavePromptVersionRequest,
    ) -> DomainResult<PromptVersionView> {
        self.prompts.save_prompt_version(request)
    }

    pub fn list_prompt_documents(
        &self,
        request: ListPromptDocumentsRequest,
    ) -> DomainResult<Vec<PromptDocumentView>> {
        self.prompts.list_prompt_documents(request)
    }

    pub fn list_prompt_versions(
        &self,
        request: ListPromptVersionsRequest,
    ) -> DomainResult<Vec<PromptVersionView>> {
        self.prompts.list_prompt_versions(request)
    }
}
```

Modify `crates/imglab-core/src/application/use_cases/mod.rs`:

```rust
pub mod prompts;
```

Modify `crates/imglab-core/src/application/facade.rs` to expose:

```rust
pub fn prompts(&self) -> PromptWorkspaceUseCase<SqlitePromptRepository> {
    PromptWorkspaceUseCase::new(self.composition.prompt_repository())
}
```

Use the actual composition type names from `facade.rs`; do not add global state.

- [ ] **Step 4: Run use case test**

Run:

```bash
cargo test -p imglab-core prompt_document_save_version_keeps_previous_version_immutable -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add crates/imglab-core/src/application crates/imglab-core/src/infrastructure crates/imglab-core/src/library/tests.rs
git commit -m "feat: add prompt workspace use cases"
```

Expected: commit contains prompt use case facade and tests.

## Task 5: Integrate Prompt Versions With Generation Events

**Files:**
- Modify: `crates/imglab-core/src/dto.rs`
- Modify: `crates/imglab-core/src/application/use_cases/generation.rs`
- Modify: `crates/imglab-core/src/library/assets.rs`
- Modify: `crates/imglab-core/src/library/generation.rs`
- Modify: `crates/imglab-daemon/src/runtime.rs`
- Modify: `crates/imglab-daemon/src/scheduler.rs`
- Test: `crates/imglab-core/src/library/tests.rs`
- Test: `crates/imglab-daemon/src/tests/scheduler.rs`

- [ ] **Step 1: Write core generation link test**

Add test:

```rust
#[test]
fn generation_event_records_prompt_version_link_without_losing_snapshot() {
    let root = test_root("generation-prompt-link");
    let registry = test_root("generation-prompt-link-registry").join("registry.sqlite");
    let app = test_application_with_registry(registry);
    app.library().init_library(&root, "Prompt Link").expect("init");

    let prompt = app.prompts().create_prompt_document(CreatePromptDocumentRequest {
        library_path: root.clone(),
        name: "Prompt Link".to_string(),
        draft_body: "A botanical study".to_string(),
        draft_negative_prompt: Some("blur".to_string()),
        draft_style_prompt: None,
        variables_schema_json: r#"{"variables":[]}"#.to_string(),
        default_values_json: "{}".to_string(),
        parameter_preset_json: r#"{"provider":"fake","model":"default","operation":"text_to_image","parameters":{}}"#.to_string(),
        notes: None,
    }).expect("create prompt");
    let version = app.prompts().save_prompt_version(SavePromptVersionRequest {
        library_path: root.clone(),
        prompt_id: prompt.id,
    }).expect("save version");

    let output = app.generation().generate(GenerateImageRequest {
        library_path: root.clone(),
        input_file: None,
        input_bytes: None,
        parameters: GenerationParameters {
            provider: "fake".to_string(),
            model: "default".to_string(),
            prompt: "A botanical study".to_string(),
            negative_prompt: Some("blur".to_string()),
            prompt_version_id: Some(version.id.clone()),
            operation: GenerationOperation::TextToImage,
            input_version_id: None,
            parameters_json: "{}".to_string(),
        },
    }).expect("generate");

    let detail = app.assets().get_asset_detail(&root, &output[0].asset_id, None).expect("detail");
    assert_eq!(detail.prompt.as_deref(), Some("A botanical study"));
    assert_eq!(detail.negative_prompt.as_deref(), Some("blur"));
    assert_eq!(detail.prompt_lineage.expect("prompt lineage").prompt_version_id, version.id);
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test -p imglab-core generation_event_records_prompt_version_link_without_losing_snapshot -- --nocapture
```

Expected: compile or assertion failure until prompt link is persisted and read.

- [ ] **Step 3: Add prompt version id to generation DTOs**

Modify `GenerationParameters`, `CreateGenerationEventRequest`, and `GenerationEventSummary` in `crates/imglab-core/src/dto.rs`:

```rust
pub prompt_version_id: Option<String>,
```

If typed IDs are preferred in this file, use `Option<PromptVersionId>` internally and map to strings at runtime boundaries.

- [ ] **Step 4: Persist prompt link**

Modify generation event insert SQL in `crates/imglab-core/src/library/assets.rs` or the active event repository:

```sql
INSERT INTO generation_events (
    id, asset_id, output_version_id, provider, provider_model, operation_type,
    prompt, negative_prompt, prompt_version_id, input_asset_version_id,
    parameters_json, raw_request_json, raw_response_json, status, started_at,
    completed_at, error_code, error_message
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
```

Ensure loaders select `prompt_version_id` and map it to summaries.

- [ ] **Step 5: Add asset detail prompt lineage**

Extend `AssetDetailView` in `crates/imglab-core/src/dto.rs`:

```rust
pub prompt_lineage: Option<PromptLineageView>,
```

In asset detail read model, when latest/focused generation event has `prompt_version_id`, join:

```sql
SELECT pd.id, pd.name, pv.id, pv.version_number
FROM prompt_versions pv
JOIN prompt_documents pd ON pd.id = pv.prompt_id
WHERE pv.id = ?1
```

Map `version_name` with `prompt_version_name(version_number)`.

- [ ] **Step 6: Update daemon input**

Modify `crates/imglab-daemon/src/runtime.rs` image generation input:

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageGenerationTaskInput {
    pub prompt: String,
    #[serde(default, rename = "negativePrompt")]
    pub negative_prompt: Option<String>,
    #[serde(default, rename = "promptVersionId")]
    pub prompt_version_id: Option<String>,
    // keep existing fields
}
```

Modify `crates/imglab-daemon/src/scheduler.rs` to pass `prompt_version_id` into `GenerationParameters`.

- [ ] **Step 7: Run core and daemon tests**

Run:

```bash
cargo test -p imglab-core generation_event_records_prompt_version_link_without_losing_snapshot -- --nocapture
cargo test -p imglab-daemon image_generation -- --nocapture
```

Expected: core prompt link test passes; existing daemon image generation tests still pass.

- [ ] **Step 8: Commit**

Run:

```bash
git add crates/imglab-core/src crates/imglab-daemon/src
git commit -m "feat: link prompt versions to generation events"
```

Expected: commit contains generation integration and tests.

## Task 6: Add Tauri Prompt Commands And DTO Mapping

**Files:**
- Create: `apps/desktop/src-tauri/src/commands/prompts.rs`
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/views.rs`
- Modify: `apps/desktop/src-tauri/src/view_mappers.rs`

- [ ] **Step 1: Add Tauri prompt views**

In `apps/desktop/src-tauri/src/views.rs`, add:

```rust
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptDocumentView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) draft_body: String,
    pub(crate) draft_negative_prompt: Option<String>,
    pub(crate) draft_style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
    pub(crate) latest_version_id: Option<String>,
    pub(crate) latest_version_number: Option<u32>,
    pub(crate) latest_version_name: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PromptVersionView {
    pub(crate) id: String,
    pub(crate) prompt_id: String,
    pub(crate) version_number: u32,
    pub(crate) version_name: String,
    pub(crate) body: String,
    pub(crate) negative_prompt: Option<String>,
    pub(crate) style_prompt: Option<String>,
    pub(crate) variables_schema_json: String,
    pub(crate) default_values_json: String,
    pub(crate) parameter_preset_json: String,
    pub(crate) notes: Option<String>,
    pub(crate) created_at: String,
}
```

- [ ] **Step 2: Add commands**

Create `apps/desktop/src-tauri/src/commands/prompts.rs`:

```rust
use crate::{error::CommandResult, library_path};
use imglab_core::{
    CreatePromptDocumentRequest, ListPromptDocumentsRequest, ListPromptVersionsRequest,
    SavePromptVersionRequest, UpdatePromptDraftRequest,
};

#[tauri::command]
pub(crate) async fn list_prompt_documents(
    library_root: String,
    query: Option<String>,
    include_archived: bool,
) -> CommandResult<Vec<crate::views::PromptDocumentView>> {
    tauri::async_runtime::spawn_blocking(move || {
        let app = crate::services::application();
        let request = ListPromptDocumentsRequest {
            library_path: library_path(library_root)?,
            query,
            include_archived,
        };
        app.prompts()
            .list_prompt_documents(request)
            .map(|items| items.into_iter().map(crate::view_mappers::prompt_document_view).collect())
            .map_err(Into::into)
    })
    .await
    .map_err(|error| crate::error::CommandError::runtime(format!("prompt worker failed: {error}")))?
}
```

Add equivalent commands for `create_prompt_document`, `update_prompt_draft`, `save_prompt_version`, and `list_prompt_versions`.

- [ ] **Step 3: Register commands**

Modify `apps/desktop/src-tauri/src/commands/mod.rs`:

```rust
pub(crate) mod prompts;
```

Modify `apps/desktop/src-tauri/src/lib.rs` invoke handler:

```rust
commands::prompts::list_prompt_documents,
commands::prompts::create_prompt_document,
commands::prompts::update_prompt_draft,
commands::prompts::save_prompt_version,
commands::prompts::list_prompt_versions,
```

- [ ] **Step 4: Build desktop backend**

Run:

```bash
cargo test -p imglab-desktop
```

Expected: Tauri command crate compiles and tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add apps/desktop/src-tauri/src
git commit -m "feat: expose prompt workspace commands"
```

Expected: commit contains Tauri DTOs, mappers, commands, and registration.

## Task 7: Add Desktop Prompt Workspace

**Files:**
- Modify: `apps/desktop/src/app/types.ts`
- Modify: `apps/desktop/src/studio-navigation.tsx`
- Create: `apps/desktop/src/app/workflows/prompts/state.ts`
- Create: `apps/desktop/src/app/workflows/prompts/controller.ts`
- Create: `apps/desktop/src/app/workflows/prompts/index.ts`
- Create: `apps/desktop/src/app/screens/workflows/prompts.tsx`
- Modify: `apps/desktop/src/app/StudioAppController.tsx`
- Modify: `apps/desktop/src/styles.css`

- [ ] **Step 1: Add frontend types**

Modify `apps/desktop/src/app/types.ts`:

```ts
export type View = "gallery" | "albums" | "review" | "queue" | "prompts" | "settings";

export type PromptDocument = {
  id: string;
  name: string;
  kind: "draft" | "template";
  status: "active" | "archived";
  draftBody: string;
  draftNegativePrompt: string | null;
  draftStylePrompt: string | null;
  variablesSchemaJson: string;
  defaultValuesJson: string;
  parameterPresetJson: string;
  notes: string | null;
  latestVersionId: string | null;
  latestVersionNumber: number | null;
  latestVersionName: string | null;
  createdAt: string;
  updatedAt: string;
};

export type PromptVersion = {
  id: string;
  promptId: string;
  versionNumber: number;
  versionName: string;
  body: string;
  negativePrompt: string | null;
  stylePrompt: string | null;
  variablesSchemaJson: string;
  defaultValuesJson: string;
  parameterPresetJson: string;
  notes: string | null;
  createdAt: string;
};

export type PromptLineage = {
  promptId: string;
  promptName: string;
  promptVersionId: string;
  promptVersionNumber: number;
  promptVersionName: string;
};
```

Extend `AssetDetail` with:

```ts
promptLineage?: PromptLineage | null;
```

- [ ] **Step 2: Add prompt workflow state**

Create `apps/desktop/src/app/workflows/prompts/state.ts`:

```ts
import type { PromptDocument, PromptVersion } from "../../types";

export type PromptWorkspaceState = {
  documents: PromptDocument[];
  versions: PromptVersion[];
  selectedPromptId: string | null;
  selectedVersionId: string | null;
  query: string;
  includeArchived: boolean;
  loading: boolean;
  saving: boolean;
  runVariablesJson: string;
  error: string | null;
};

export const initialPromptWorkspaceState: PromptWorkspaceState = {
  documents: [],
  versions: [],
  selectedPromptId: null,
  selectedVersionId: null,
  query: "",
  includeArchived: false,
  loading: false,
  saving: false,
  runVariablesJson: "{}",
  error: null,
};

export function selectedPrompt(state: PromptWorkspaceState): PromptDocument | null {
  return state.documents.find((prompt) => prompt.id === state.selectedPromptId) ?? null;
}

export function selectedVersion(state: PromptWorkspaceState): PromptVersion | null {
  return state.versions.find((version) => version.id === state.selectedVersionId) ?? null;
}
```

- [ ] **Step 3: Add prompt screen**

Create `apps/desktop/src/app/screens/workflows/prompts.tsx`:

```tsx
import React from "react";
import type { PromptDocument, PromptVersion } from "../../types";

export function PromptsWorkspace({
  documents,
  versions,
  selectedPromptId,
  selectedVersionId,
  query,
  loading,
  saving,
  onQueryChange,
  onSelectPrompt,
  onSelectVersion,
  onCreatePrompt,
  onSaveVersion,
  onRunVersion,
}: {
  documents: PromptDocument[];
  versions: PromptVersion[];
  selectedPromptId: string | null;
  selectedVersionId: string | null;
  query: string;
  loading: boolean;
  saving: boolean;
  onQueryChange: (value: string) => void;
  onSelectPrompt: (id: string) => void;
  onSelectVersion: (id: string) => void;
  onCreatePrompt: () => void;
  onSaveVersion: () => void;
  onRunVersion: () => void;
}) {
  const prompt = documents.find((item) => item.id === selectedPromptId) ?? null;
  const version = versions.find((item) => item.id === selectedVersionId) ?? versions[0] ?? null;

  return (
    <section className="prompts-workspace">
      <aside className="prompts-library">
        <div className="panel-heading">
          <h2>Prompts</h2>
          <button className="icon-button" onClick={onCreatePrompt} aria-label="New prompt">+</button>
        </div>
        <input
          className="search-input"
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          aria-label="Search prompts"
        />
        <div className="prompt-list" aria-busy={loading}>
          {documents.map((item) => (
            <button
              key={item.id}
              className={`prompt-row${item.id === selectedPromptId ? " is-selected" : ""}`}
              onClick={() => onSelectPrompt(item.id)}
            >
              <strong>{item.name}</strong>
              <small>{item.latestVersionName ?? "Draft only"}</small>
            </button>
          ))}
        </div>
      </aside>

      <main className="prompt-editor-pane">
        <div className="panel-heading">
          <h2>{prompt?.name ?? "No prompt selected"}</h2>
          <button className="primary-button" disabled={!prompt || saving} onClick={onSaveVersion}>
            Save version
          </button>
        </div>
        <textarea className="prompt-body-editor" value={prompt?.draftBody ?? ""} readOnly />
        <div className="prompt-version-list">
          {versions.map((item) => (
            <button
              key={item.id}
              className={`version-chip${item.id === selectedVersionId ? " is-selected" : ""}`}
              onClick={() => onSelectVersion(item.id)}
            >
              {item.versionName}
            </button>
          ))}
        </div>
      </main>

      <aside className="prompt-run-pane">
        <div className="panel-heading">
          <h2>Run</h2>
          <button className="primary-button" disabled={!version} onClick={onRunVersion}>
            Generate
          </button>
        </div>
        <pre className="prompt-preview">{version?.body ?? "Save a version before running."}</pre>
      </aside>
    </section>
  );
}
```

- [ ] **Step 4: Wire navigation**

Modify `apps/desktop/src/studio-navigation.tsx` to add `Prompts` between `Queue` and `Settings`.

Modify `apps/desktop/src/app/StudioAppController.tsx` to render `PromptsWorkspace` when `view === "prompts"`.

- [ ] **Step 5: Add CSS**

Modify `apps/desktop/src/styles.css`:

```css
.prompts-workspace {
  display: grid;
  grid-template-columns: minmax(180px, 240px) minmax(360px, 1fr) minmax(260px, 340px);
  gap: 12px;
  min-height: 0;
}

.prompts-library,
.prompt-editor-pane,
.prompt-run-pane {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-subtle);
  background: var(--panel-bg);
}

.prompt-list {
  min-height: 0;
  overflow: auto;
}

.prompt-row {
  display: grid;
  gap: 2px;
  width: 100%;
  padding: 8px 10px;
  border: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.prompt-row.is-selected {
  background: var(--selection-bg);
}

.prompt-body-editor,
.prompt-preview {
  min-height: 220px;
  max-height: 420px;
  overflow: auto;
}

@media (max-width: 1080px) {
  .prompts-workspace {
    grid-template-columns: minmax(160px, 220px) minmax(0, 1fr);
  }

  .prompt-run-pane {
    grid-column: 1 / -1;
  }
}
```

Adjust CSS variable names to match existing `styles.css`; do not invent names if equivalents already exist.

- [ ] **Step 6: Run desktop tests/build**

Run:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
```

Expected: tests and build pass.

- [ ] **Step 7: Commit**

Run:

```bash
git add apps/desktop/src
git commit -m "feat: add prompt workspace UI"
```

Expected: commit contains frontend types, workflow, screen, navigation, and CSS.

## Task 8: Add Inspector Prompt Lineage And Save As Prompt

**Files:**
- Modify: `apps/desktop/src/app/screens/workflows/inspector.tsx`
- Modify: `apps/desktop/src/app/StudioAppController.tsx`
- Modify: `apps/desktop/src-tauri/src/commands/prompts.rs`
- Modify: `crates/imglab-core/src/application/use_cases/prompts.rs`

- [ ] **Step 1: Add Save as Prompt core use case**

Add request DTO:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveGenerationPromptAsPromptRequest {
    pub library_path: PathBuf,
    pub generation_event_id: String,
    pub name: String,
}
```

Add method to prompt use case:

```rust
pub fn save_generation_prompt_as_prompt(
    &self,
    request: SaveGenerationPromptAsPromptRequest,
) -> DomainResult<PromptVersionView> {
    self.prompts.save_generation_prompt_as_prompt(request)
}
```

Repository implementation loads generation event snapshot, creates prompt document, then saves initial version. Do not rewrite the old generation event link in MVP.

- [ ] **Step 2: Add Tauri command**

In `apps/desktop/src-tauri/src/commands/prompts.rs`:

```rust
#[tauri::command]
pub(crate) async fn save_generation_prompt_as_prompt(
    library_root: String,
    generation_event_id: String,
    name: String,
) -> CommandResult<crate::views::PromptVersionView> {
    tauri::async_runtime::spawn_blocking(move || {
        let app = crate::services::application();
        app.prompts()
            .save_generation_prompt_as_prompt(imglab_core::SaveGenerationPromptAsPromptRequest {
                library_path: library_path(library_root)?,
                generation_event_id,
                name,
            })
            .map(crate::view_mappers::prompt_version_view)
            .map_err(Into::into)
    })
    .await
    .map_err(|error| crate::error::CommandError::runtime(format!("prompt worker failed: {error}")))?
}
```

Register it in `lib.rs`.

- [ ] **Step 3: Update Inspector UI**

In `apps/desktop/src/app/screens/workflows/inspector.tsx`, near the prompt section:

```tsx
{detail.promptLineage ? (
  <div className="prompt-lineage-link">
    <span>Prompt version</span>
    <button className="text-button" onClick={() => onOpenPromptVersion(detail.promptLineage!.promptId, detail.promptLineage!.promptVersionId)}>
      {detail.promptLineage.promptName} / {detail.promptLineage.promptVersionName}
    </button>
  </div>
) : detail.prompt ? (
  <button className="text-button" onClick={() => onSavePromptSnapshot(detail)}>
    Save as Prompt
  </button>
) : null}
```

Thread `onOpenPromptVersion` and `onSavePromptSnapshot` props from `StudioAppController.tsx`.

- [ ] **Step 4: Run focused tests**

Run:

```bash
cargo test -p imglab-core save_generation_prompt_as_prompt -- --nocapture
cargo test -p imglab-desktop
npm test --prefix apps/desktop
```

Expected: core save-as-prompt test passes, desktop tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add crates/imglab-core/src apps/desktop/src-tauri/src apps/desktop/src
git commit -m "feat: add prompt lineage actions"
```

Expected: commit contains Save as Prompt and Inspector prompt lineage.

## Task 9: Final Verification And OpenSpec Closeout

**Files:**
- Modify: `openspec/specs/prompt-workspace/spec.md` if syncing manually.
- Move: `openspec/changes/add-prompt-workspace` to archive through OpenSpec archive command.

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --all --check
cargo test -p imglab-core
cargo test -p imglab-daemon
cargo test -p imglab-desktop
```

Expected: all pass.

- [ ] **Step 2: Run desktop verification**

Run:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
```

Expected: all pass.

- [ ] **Step 3: Run architecture and diff checks**

Run:

```bash
scripts/check-architecture.sh
git diff --check
git status --short
```

Expected: architecture check passes, diff check passes, status shows only intentional files before final commit.

- [ ] **Step 4: Validate OpenSpec change**

Run:

```bash
openspec validate add-prompt-workspace --strict
```

Expected: passes.

- [ ] **Step 5: Sync/archive OpenSpec change**

Run:

```bash
openspec archive add-prompt-workspace --yes
openspec validate --specs --strict
```

Expected: change moves to `openspec/changes/archive/YYYY-MM-DD-add-prompt-workspace/`; specs validation passes.

- [ ] **Step 6: Final commit**

Run:

```bash
git add crates apps openspec docs
git commit -m "feat: add prompt workspace"
```

Expected: final implementation and archive are committed. If previous task commits already captured implementation, this commit should contain only archive/spec sync and any final fixes.

- [ ] **Step 7: Final status**

Run:

```bash
git status --short
git log --oneline -5
```

Expected: worktree clean; recent commits show Prompt Workspace design, OpenSpec, implementation slices, and closeout.

## Self-Review

Spec coverage:

- Prompt Library: Task 1 OpenSpec, Task 3 repository, Task 4 use cases, Task 7 UI.
- Prompt version: Task 2 domain policy, Task 3 persistence, Task 4 use cases, Task 7 UI.
- Template variables: Task 2 rendering tests/policy, Task 1 spec, Task 7 run UI shell.
- Parameter preset: Task 1 spec, Task 3 persistence, Task 7 UI data flow.
- Negative/style prompt: Task 2/3 model and persistence, Task 7 UI shell.
- Prompt notes: Task 2/3 model and persistence, Task 7 UI shell.
- Generation from prompt: Task 5 generation/task integration, Task 7 run UI.
- Prompt-to-output history: Task 1 spec, Task 5 generation link, Task 7 history panel follow-up.
- Asset prompt lineage: Task 5 read model, Task 8 Inspector link and Save as Prompt.
- Compatibility: Task 3 migration, Task 5 snapshot preservation, Task 9 OpenSpec/spec validation.

Completion-marker scan:

- The plan avoids incomplete-work markers and vague implementation instructions.

Type consistency:

- `promptVersionId` is camelCase in desktop/daemon JSON.
- `prompt_version_id` is snake_case in Rust core DTOs and SQLite.
- UI uses `PromptDocument`, `PromptVersion`, and `PromptLineage`.
