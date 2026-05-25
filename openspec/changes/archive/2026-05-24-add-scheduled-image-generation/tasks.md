## 1. Core Domain And Persistence

- [x] 1.1 Add scheduled generation domain models for job, schedule rule, run, run output, prompt mode, overlap policy, missed-run policy, and run status.
- [x] 1.2 Add schedule policy tests for interval minutes, interval hours, daily time, timezone, DST invalid time, overlap skip, and missed no-catch-up.
- [x] 1.3 Add DTOs and application repository ports for schedule CRUD, run creation, run status updates, due job query, and run output idempotence.
- [x] 1.4 Add SQLite schema migration for scheduled generation jobs, runs, and run outputs.
- [x] 1.5 Implement SQLite repository methods and tests for schedule CRUD, due job query, run/task link, run output upsert, and deleted target album validation.
- [x] 1.6 Add registry persistence for per-library `automation_enabled`, defaulting to false for existing and new libraries.
- [x] 1.7 Extend backup/restore tests to prove schedule tables remain inside library backup while automation opt-in remains registry-owned.

## 2. Prompt Expansion Providers

- [x] 2.1 Add prompt expansion provider port, request, response, provider metadata, and error classification.
- [x] 2.2 Implement deterministic fake prompt expansion provider with unit tests.
- [x] 2.3 Implement `codex-cli` prompt expansion adapter with prompt template, output parsing, cancellation/log handling, and unit tests.
- [x] 2.4 Add provider capability reporting for prompt expansion without changing existing image generation capability semantics.

## 3. Daemon Schedule Runner

- [x] 3.1 Add daemon schedule routes for list/create/update/enable/disable/run-now/list-runs/get-run.
- [x] 3.2 Add schedule runner loop separate from existing task scheduler loop.
- [x] 3.3 Implement due job handling: create run, apply overlap skip, apply missed no-catch-up, and update `next_run_at`.
- [x] 3.4 Implement fixed prompt task handoff using ordinary image generation task creation.
- [x] 3.5 Implement dynamic prompt expansion handoff, including failed expansion run state and no task creation on failure.
- [x] 3.6 Implement linked task reconciliation for queued/running/retry/completed/failed task states.
- [x] 3.7 Implement post-processing that adds output assets to target manual album and applies schedule tags idempotently.
- [x] 3.8 Add daemon tests for due schedule, dynamic prompt success/failure, overlap skip, missed no-catch-up, post-processing, restart recovery, and automation-enabled library scanning.

## 4. Background Daemon Service Management

- [x] 4.1 Add daemon runtime config that can use app registry path instead of temp-only runtime registry.
- [x] 4.2 Add macOS LaunchAgent service adapter for install, uninstall, status, repair, start, stop, restart, and graceful drain.
- [x] 4.3 Update desktop daemon discovery to prefer healthy background daemon and avoid duplicate sidecar.
- [x] 4.4 Add Tauri automation daemon commands and tests for status mapping, enable/disable, restart, repair, and recoverable diagnostics.
- [x] 4.5 Update app logs roots to include background daemon logs.

## 5. Desktop Schedules Workflow

- [x] 5.1 Add TypeScript types, Tauri adapter calls, controller state, and hooks for scheduled generation jobs and runs.
- [x] 5.2 Add `Schedules` route/workflow entry to shell navigation without disturbing Gallery, Albums, Prompts, Review, Queue, or Settings ownership.
- [x] 5.3 Build Schedules job list, detail editor, fixed/dynamic prompt mode controls, schedule controls, album selector, tag chips, and run history.
- [x] 5.4 Add run-now, enable/disable, duplicate, delete, and linked Queue task navigation interactions.
- [x] 5.5 Add frontend tests for rendering, validation, mode switching, schedule controls, and run history task links.

## 6. Settings Automation UI

- [x] 6.1 Add Settings Automation section and navigation label.
- [x] 6.2 Add background daemon status, LaunchAgent status, Start/Stop/Restart/Repair controls.
- [x] 6.3 Add per-library automation opt-in toggles and status feedback.
- [x] 6.4 Add frontend tests for daemon toggle, library opt-in toggle, offline/misconfigured diagnostics, and recoverable repair state.

## 7. Integration And Documentation

- [x] 7.1 Add schedule origin context to Queue task detail and back-link to schedule run.
- [x] 7.2 Update development docs with background daemon, LaunchAgent, schedule runner, and local debugging commands.
- [x] 7.3 Run Rust formatting and targeted core/daemon/desktop tests.
- [x] 7.4 Run desktop frontend tests and build.
- [x] 7.5 Run OpenSpec strict validation for the change and all specs.
- [x] 7.6 Complete manual smoke test: enable automation library, create fixed job, create dynamic job with fake provider, run now, verify album membership and tags.
