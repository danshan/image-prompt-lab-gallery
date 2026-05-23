# Tasks: Add Asset Tag Use Case

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm the change preserves persistence and CLI contracts.

## 2. Application Boundary

- [x] 2.1 Add an asset tag request model.
- [x] 2.2 Add an `AssetUseCase` tag mutation method backed by `AssetRepository`.
- [x] 2.3 Implement the SQLite repository method using existing tag persistence behavior.

## 3. Runtime Migration

- [x] 3.1 Route CLI `tag add` through `app.assets()`.
- [x] 3.2 Keep CLI output and dry-run behavior unchanged.

## 4. Documentation

- [x] 4.1 Update architecture inventory to remove CLI tag as a direct legacy-service exception.
- [x] 4.2 Update specs with the tag use-case boundary.

## 5. Verification

- [x] 5.1 Run Rust core and CLI tests.
- [x] 5.2 Run architecture guardrails.
- [x] 5.3 Run OpenSpec validation.
