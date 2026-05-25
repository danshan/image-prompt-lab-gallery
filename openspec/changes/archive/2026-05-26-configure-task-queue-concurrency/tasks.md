## 1. Daemon Settings Contract

- [x] 1.1 Add daemon task queue settings DTOs for reading and updating max parallel tasks, including default, minimum, maximum and effective values.
- [x] 1.2 Add app-level settings load/save logic for daemon scheduler settings with fallback to `TaskSchedulerConfig::default()`.
- [x] 1.3 Add daemon HTTP routes for reading and updating task queue settings, with validation errors for values outside the allowed range.
- [x] 1.4 Add daemon route tests for default read, successful update, invalid value rejection and corrupted or missing settings fallback.

## 2. Concurrent Scheduler Execution

- [x] 2.1 Refactor scheduler selection so one scheduler pass can claim multiple eligible queued tasks up to the configured global slots.
- [x] 2.2 Ensure claiming a task is atomic against task status, so concurrent scheduler passes cannot start the same task twice.
- [x] 2.3 Run provider execution in worker boundaries that do not hold daemon shared-state locks during long-running provider calls.
- [x] 2.4 Preserve provider-level concurrency wait reasons, queue priority, queue position and retry wait behavior.
- [x] 2.5 Add daemon scheduler tests for multiple tasks running concurrently, global slot saturation, provider slot saturation and lowering the configured maximum below current running count.

## 3. Desktop Backend Integration

- [x] 3.1 Add Tauri daemon client methods and commands to read and update task queue settings.
- [x] 3.2 Map daemon validation and offline failures into recoverable desktop errors.
- [x] 3.3 Add Rust desktop backend tests or command-level tests for settings read/update mapping where existing test patterns support it.

## 4. Settings UI

- [x] 4.1 Add `taskQueue` to Settings workflow section state, section navigation and URL/default section handling without changing the default `Libraries` section.
- [x] 4.2 Add a compact `SettingsTaskQueueView` with labeled numeric input or stepper, allowed range, default value, save state and provider-limit explanatory copy.
- [x] 4.3 Wire Settings controller state/actions to load current task queue settings, save changes and display daemon offline or validation errors.
- [x] 4.4 Add dictionary entries and keep text layout safe at the `960px` compact desktop target.
- [x] 4.5 Add frontend tests for section navigation, valid save flow, invalid input handling and daemon offline state.

## 5. Verification

- [x] 5.1 Run `openspec validate configure-task-queue-concurrency --strict`.
- [x] 5.2 Run `cargo fmt --all --check`.
- [x] 5.3 Run `cargo test -p imglab-core` and `cargo test -p imglab-daemon`.
- [x] 5.4 Run `cargo test -p imglab-desktop` if desktop command tests are changed.
- [x] 5.5 Run `npm test --prefix apps/desktop` and `npm run build --prefix apps/desktop`.
- [x] 5.6 Run `git diff --check` and `git status --short`.
