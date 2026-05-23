# Tasks: Add Library Lifecycle Use Case

## 1. OpenSpec and Boundary Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm the change preserves resource library persistence contracts.

## 2. Core Application Owner

- [x] 2.1 Implement `LibraryUseCase` over the `LibraryRepository` port.
- [x] 2.2 Add owner-local tests for lifecycle delegation and compatibility-sensitive behavior.
- [x] 2.3 Wire `library_lifecycle()` in the application facade and SQLite composition.

## 3. Runtime Migration

- [x] 3.1 Move CLI library lifecycle commands to `app.library_lifecycle()`.
- [x] 3.2 Move daemon open-library lifecycle path to `app.library_lifecycle()`.
- [x] 3.3 Move Tauri library lifecycle commands to `app.library_lifecycle()`.
- [x] 3.4 Keep direct legacy service usage documented only for remaining compatibility gaps.

## 4. Documentation and Guardrails

- [x] 4.1 Update DDD boundary inventory and runtime adapter review.
- [x] 4.2 Update OpenSpec specs for library lifecycle ownership.
- [x] 4.3 Run architecture guardrails.

## 5. Verification

- [x] 5.1 Run Rust core, CLI, daemon, and desktop tests.
- [x] 5.2 Run OpenSpec validation.
- [x] 5.3 Record any remaining gaps for follow-up changes.
