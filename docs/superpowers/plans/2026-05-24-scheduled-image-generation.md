# Scheduled Image Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build scheduled image generation with fixed/dynamic prompts, background daemon execution, manual album output routing, user tags, Schedules workflow, and Settings Automation controls.

**Architecture:** Add a scheduled-generation bounded context in `imglab-core`, then let `imglab-daemon` run due schedules and enqueue ordinary image generation tasks. Desktop remains a client/control plane: `Schedules` manages recurring jobs, Queue displays concrete tasks, and Settings manages background daemon service state and per-library automation opt-in.

**Tech Stack:** Rust workspace (`imglab-core`, `imglab-daemon`, `imglab-provider-codex`, `imglab-desktop`), SQLite migrations, Tauri 2 commands, React 19 + TypeScript, OpenSpec.

---

## File Structure

Create:

- `crates/imglab-core/src/domain/schedule/mod.rs`: schedule domain exports.
- `crates/imglab-core/src/domain/schedule/model.rs`: job, rule, run, output, status, policy types.
- `crates/imglab-core/src/domain/schedule/policies.rs`: next-run, overlap, missed-run, status transition helpers.
- `crates/imglab-core/src/application/use_cases/schedules.rs`: schedule CRUD, run lifecycle, due query, output post-processing use cases.
- `crates/imglab-core/src/application/ports/prompt_expansion.rs`: prompt expansion provider port.
- `crates/imglab-core/src/infrastructure/sqlite/schedules.rs`: SQLite repository implementation.
- `crates/imglab-daemon/src/schedule_runner.rs`: daemon schedule runner loop and run/task reconciliation.
- `crates/imglab-daemon/src/prompt_expansion.rs`: daemon prompt expansion dispatch.
- `apps/desktop/src-tauri/src/commands/schedules.rs`: Tauri schedule commands.
- `apps/desktop/src-tauri/src/automation_service.rs`: macOS LaunchAgent service adapter.
- `apps/desktop/src/app/workflows/schedules/state.ts`: Schedules workflow state.
- `apps/desktop/src/app/workflows/schedules/controller.ts`: Schedules workflow actions.
- `apps/desktop/src/app/workflows/schedules/screen.ts`: workflow screen exports.
- `apps/desktop/src/app/screens/workflows/schedules.tsx`: Schedules UI.

Modify:

- `crates/imglab-core/src/domain/mod.rs`: export schedule domain.
- `crates/imglab-core/src/application/mod.rs`: export schedule use case and prompt expansion port.
- `crates/imglab-core/src/application/ports/repositories.rs`: add schedule repository operations.
- `crates/imglab-core/src/infrastructure/sqlite/schema.rs`: schema migration and current version.
- `crates/imglab-core/src/infrastructure/sqlite/mod.rs`: include schedules repository.
- `crates/imglab-core/src/infrastructure/composition.rs`: expose schedule use case.
- `crates/imglab-core/src/dto.rs`: schedule DTOs and prompt expansion DTOs.
- `crates/imglab-core/src/provider.rs`: fake prompt expansion provider.
- `crates/imglab-provider-codex/src/lib.rs`: Codex CLI prompt expansion implementation.
- `crates/imglab-daemon/src/runtime.rs`: schedule runtime DTOs and accessors.
- `crates/imglab-daemon/src/transport.rs`: route schedule endpoints and spawn runner.
- `crates/imglab-daemon/src/routes.rs`: parse schedule API inputs.
- `crates/imglab-daemon/src/views.rs`: schedule/run response views.
- `apps/desktop/src-tauri/src/lib.rs`: register new commands/modules.
- `apps/desktop/src-tauri/src/daemon_client.rs`: schedule API client and daemon discovery preference.
- `apps/desktop/src-tauri/src/views.rs`: schedule and automation service views.
- `apps/desktop/src/app/types.ts`: schedule and automation types.
- `apps/desktop/src/app/tauri-adapter.ts`: schedule/service command wrappers.
- `apps/desktop/src/app/StudioAppController.tsx`: wire schedules and Settings Automation state.
- `apps/desktop/src/studio-navigation.tsx`: add Schedules workflow entry.
- `apps/desktop/src/app/screens/workflows/settings.tsx`: add Automation section.
- `apps/desktop/src/app/i18n/dictionaries.ts`: labels and messages.
- `apps/desktop/src/styles.css`: compact Schedules and Automation styling.
- `docs/development.md`: background daemon and schedule debugging docs.
- `openspec/changes/add-scheduled-image-generation/tasks.md`: mark tasks complete as they land.

## 1. OpenSpec Baseline

- [ ] **Step 1: Confirm artifacts are apply-ready**

Run:

```bash
openspec status --change add-scheduled-image-generation --json
```

Expected: `proposal`, `design`, `specs`, and `tasks` are `done`.

- [ ] **Step 2: Validate change before code**

Run:

```bash
openspec validate add-scheduled-image-generation --strict
```

Expected: `Change 'add-scheduled-image-generation' is valid`.

## 2. Core Schedule Domain

- [ ] **Step 1: Write schedule policy tests**

Add tests under `crates/imglab-core/src/domain/schedule/policies.rs` for:

- interval minutes next run.
- interval hours next run.
- daily local `HH:mm`.
- missed no-catch-up.
- overlap active run skip.
- DST invalid local time diagnostic.

Run:

```bash
cargo test -p imglab-core schedule::policies -- --nocapture
```

Expected before implementation: compile failure because `domain::schedule` does not exist.

- [ ] **Step 2: Add schedule domain modules**

Create `domain/schedule/model.rs` with typed enums for `SchedulePromptMode`, `ScheduleRule`, `ScheduledGenerationJobStatus`, `ScheduledGenerationRunStatus`, `ScheduleOverlapPolicy`, and `ScheduleMissedRunPolicy`. Keep parsing/formatting methods close to the enums.

Update `domain/mod.rs`:

```rust
pub mod schedule;
```

Run:

```bash
cargo test -p imglab-core schedule::policies -- --nocapture
```

Expected: policy tests compile and fail on missing policy functions.

- [ ] **Step 3: Implement schedule policies**

Implement pure helpers in `domain/schedule/policies.rs`:

- `next_interval_run_after(now, interval_minutes)`.
- `next_daily_run_after(now, timezone_id, local_hh_mm)`.
- `resolve_due_schedule(now, next_run_at, active_run_exists, missed_policy, overlap_policy)`.

Run:

```bash
cargo test -p imglab-core schedule::policies -- --nocapture
```

Expected: all schedule policy tests pass.

## 3. Core Persistence And Use Cases

- [ ] **Step 1: Write SQLite repository tests**

Add tests in `crates/imglab-core/src/library/tests.rs` or a focused schedules test module covering:

- create/list/update/enable/disable job.
- due job query.
- create run and link image task id.
- upsert run output idempotently.
- reject or pause job with missing/non-manual target album.

Run:

```bash
cargo test -p imglab-core scheduled_generation -- --nocapture
```

Expected before implementation: compile failure on missing DTOs/repository methods.

- [ ] **Step 2: Add DTOs and repository port**

Modify `crates/imglab-core/src/dto.rs` with request/view structs:

- `CreateScheduledGenerationJobRequest`.
- `UpdateScheduledGenerationJobRequest`.
- `ScheduledGenerationJobView`.
- `ScheduledGenerationRunView`.
- `ScheduledGenerationRunOutputView`.
- `CreateScheduledGenerationRunRequest`.
- `UpdateScheduledGenerationRunRequest`.

Modify `application/ports/repositories.rs` to add a `ScheduleRepository` trait. Keep path-scoped methods using `library_path` for library DB work and registry-scoped methods for automation opt-in.

Run:

```bash
cargo check -p imglab-core
```

Expected: compile fails until use case and implementation are added.

- [ ] **Step 3: Add migration and SQLite repository**

Modify `infrastructure/sqlite/schema.rs`:

- increment `CURRENT_SCHEMA_VERSION`.
- create `scheduled_generation_jobs`.
- create `scheduled_generation_runs`.
- create `scheduled_generation_run_outputs`.
- add indexes for due query and job/run lookup.

Create `infrastructure/sqlite/schedules.rs` with repository implementation using existing `LocalLibraryService` patterns.

Run:

```bash
cargo test -p imglab-core scheduled_generation -- --nocapture
```

Expected: repository tests pass.

- [ ] **Step 4: Add schedule use case facade**

Create `application/use_cases/schedules.rs` and expose it from composition. Use cases should validate target album, prompt mode required fields, schedule rule, and tags before persistence.

Run:

```bash
cargo test -p imglab-core scheduled_generation -- --nocapture
```

Expected: core schedule tests pass.

## 4. Prompt Expansion Providers

- [ ] **Step 1: Add prompt expansion port tests**

Add tests for fake prompt expansion in `crates/imglab-core/src/provider.rs`:

- non-empty base/dynamic prompt returns deterministic expanded prompt.
- empty dynamic prompt fails validation.

Run:

```bash
cargo test -p imglab-core prompt_expansion -- --nocapture
```

Expected before implementation: compile failure on missing port.

- [ ] **Step 2: Add provider port and fake implementation**

Create `application/ports/prompt_expansion.rs` and implement fake provider in `provider.rs`. Expose capability separately from image generation support.

Run:

```bash
cargo test -p imglab-core prompt_expansion -- --nocapture
```

Expected: fake prompt expansion tests pass.

- [ ] **Step 3: Add Codex CLI prompt expansion**

Modify `crates/imglab-provider-codex/src/lib.rs`:

- add prompt expansion request builder.
- run Codex CLI with log/cancel path support.
- parse final expanded prompt from stdout.
- unit test command construction and output parsing.

Run:

```bash
cargo test -p imglab-provider-codex prompt_expansion -- --nocapture
```

Expected: Codex prompt expansion unit tests pass without launching real Codex.

## 5. Daemon Schedule Runner

- [ ] **Step 1: Write daemon runner tests**

Add tests in `crates/imglab-daemon/src/tests/scheduler.rs` or a new schedules test module for:

- due fixed job creates run and image task.
- dynamic job calls fake expansion and stores expanded prompt.
- expansion failure creates failed run and no task.
- previous active run creates skipped run.
- completed linked task applies album and tags.
- restart resumes post-processing idempotently.
- daemon scans only automation-enabled libraries.

Run:

```bash
cargo test -p imglab-daemon scheduled_generation -- --nocapture
```

Expected before implementation: compile failure on missing schedule runner.

- [ ] **Step 2: Add daemon schedule DTOs and routes**

Modify `runtime.rs`, `routes.rs`, `views.rs`, and `transport.rs` to support schedule APIs listed in OpenSpec. Keep route parsing outside expensive mutable state sections where possible.

Run:

```bash
cargo test -p imglab-daemon api -- --nocapture
```

Expected: existing daemon API tests pass and new schedule route tests compile.

- [ ] **Step 3: Implement schedule runner loop**

Create `schedule_runner.rs`. The loop should:

- query due jobs.
- create skipped runs for active overlap.
- handle missed no-catch-up.
- resolve prompt.
- enqueue image generation task.
- reconcile linked task status.
- post-process completed outputs.

Run:

```bash
cargo test -p imglab-daemon scheduled_generation -- --nocapture
```

Expected: daemon schedule runner tests pass.

## 6. Background Daemon Service

- [ ] **Step 1: Add service adapter tests**

In `apps/desktop/src-tauri/src/tests.rs` or a focused module, test status mapping for:

- disabled.
- enabled/running.
- enabled/offline.
- LaunchAgent missing.
- LaunchAgent misconfigured.

Run:

```bash
cargo test -p imglab-desktop automation_daemon -- --nocapture
```

Expected before implementation: compile failure on missing automation service module.

- [ ] **Step 2: Implement macOS LaunchAgent adapter**

Create `automation_service.rs`. Keep platform-specific file paths and plist generation in one module. Use dry, testable functions for label, plist path, runtime dir, install/uninstall/status mapping.

Run:

```bash
cargo test -p imglab-desktop automation_daemon -- --nocapture
```

Expected: service adapter tests pass without requiring actual launchctl.

- [ ] **Step 3: Add Tauri commands**

Add commands:

- `get_automation_daemon_status`.
- `set_automation_daemon_enabled`.
- `set_library_automation_enabled`.
- `restart_automation_daemon`.
- `repair_automation_launch_agent`.

Run:

```bash
cargo test -p imglab-desktop --lib
```

Expected: Tauri backend tests pass.

## 7. Desktop Schedules Workflow

- [ ] **Step 1: Add frontend types and adapter methods**

Modify `apps/desktop/src/app/types.ts` and `tauri-adapter.ts` for schedule jobs, runs, run outputs, automation daemon status, and commands.

Run:

```bash
npm test --prefix apps/desktop -- schedules
```

Expected before UI implementation: compile/test failure on missing workflow.

- [ ] **Step 2: Add workflow state and controller**

Create `app/workflows/schedules/state.ts`, `controller.ts`, and `screen.ts`. Keep state ownership local to Schedules workflow. Do not reuse Gallery or Prompts state as hidden global state.

Run:

```bash
npm test --prefix apps/desktop -- schedules
```

Expected: schedule state/controller unit tests pass.

- [ ] **Step 3: Add Schedules screen**

Create `app/screens/workflows/schedules.tsx` with:

- job list.
- fixed/dynamic prompt editor.
- schedule controls.
- album selector.
- tag chips.
- run history.
- linked task navigation.

Run:

```bash
npm test --prefix apps/desktop -- schedules
```

Expected: Schedules workflow tests pass.

- [ ] **Step 4: Wire navigation and controller**

Modify `StudioAppController.tsx`, `studio-navigation.tsx`, dictionaries, and styles. Keep compact desktop width at `960px`.

Run:

```bash
npm run build --prefix apps/desktop
```

Expected: production frontend build passes.

## 8. Settings Automation

- [ ] **Step 1: Add Settings Automation tests**

Add tests for:

- daemon enabled/disabled UI.
- LaunchAgent status display.
- per-library automation opt-in toggle.
- repair action visibility for misconfigured state.

Run:

```bash
npm test --prefix apps/desktop -- settings
```

Expected before implementation: tests fail on missing Automation section.

- [ ] **Step 2: Implement Settings Automation section**

Modify settings workflow state/controller/screen to add `automation` section. Do not merge this into Providers; daemon service lifecycle is its own Settings concern.

Run:

```bash
npm test --prefix apps/desktop -- settings
```

Expected: Settings Automation tests pass.

## 9. Verification And OpenSpec Closeout

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --all --check
cargo test -p imglab-core
cargo test -p imglab-daemon
cargo test -p imglab-desktop
cargo test -p imglab-provider-codex
```

Expected: all commands pass.

- [ ] **Step 2: Run frontend verification**

Run:

```bash
npm test --prefix apps/desktop
npm run build --prefix apps/desktop
```

Expected: tests and build pass.

- [ ] **Step 3: Run OpenSpec verification**

Run:

```bash
openspec validate add-scheduled-image-generation --strict
openspec validate --specs --strict
```

Expected: change and specs are valid.

- [ ] **Step 4: Update OpenSpec task checklist**

Mark completed tasks in `openspec/changes/add-scheduled-image-generation/tasks.md`.

Run:

```bash
git diff --check
git status --short
```

Expected: no whitespace errors and only intended files changed.

- [ ] **Step 5: Archive after implementation is complete**

After all implementation and validation pass, sync delta specs into `openspec/specs`, archive the change under `openspec/changes/archive/YYYY-MM-DD-add-scheduled-image-generation/`, and re-run strict specs validation.
