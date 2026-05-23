# Tasks: Remove Daemon Task Service Accessor

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm daemon API and task persistence contracts remain stable.

## 2. Daemon Runtime Migration

- [x] 2.1 Replace production daemon task calls through `service()` with `tasks()`.
- [x] 2.2 Replace daemon test task calls through `service()` with `tasks()`.
- [x] 2.3 Remove `DaemonState::service()` after all call sites are migrated.

## 3. Documentation

- [x] 3.1 Update architecture inventory for narrowed daemon task boundary.
- [x] 3.2 Update specs with the daemon task owner requirement.

## 4. Verification

- [x] 4.1 Run daemon tests.
- [x] 4.2 Run architecture guardrails.
- [x] 4.3 Run OpenSpec validation.
