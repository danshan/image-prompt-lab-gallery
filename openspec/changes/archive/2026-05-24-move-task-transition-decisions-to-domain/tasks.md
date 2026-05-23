# Tasks: Move Task Transition Decisions To Domain

## Implementation

- [x] Add domain task policy helpers for cancel request and recovery transition decisions.
- [x] Update daemon cancel handling to use the domain cancel policy.
- [x] Update daemon recovery handling to use the domain recovery policy.
- [x] Update architecture inventory and canonical specs.

## Verification

- [x] Run `cargo fmt --all --check`.
- [x] Run `cargo test -p imglab-core`.
- [x] Run `cargo test -p imglab-daemon`.
- [x] Run `scripts/check-architecture.sh`.
- [x] Run `openspec validate move-task-transition-decisions-to-domain --strict`.
- [x] Run `openspec validate --specs --strict`.
- [x] Run `git diff --check`.
