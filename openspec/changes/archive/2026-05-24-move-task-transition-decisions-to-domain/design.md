# Design: Move Task Transition Decisions To Domain

## Overview

This change adds focused policy helpers to `domain::task::policies` for cancel requests and daemon recovery transitions.

Daemon code will continue to perform runtime concerns:

- finding task detail across opened libraries
- writing cancellation marker files for running attempts
- appending recovery and cancellation events
- calculating wall-clock retry readiness and backoff timestamps
- mapping HTTP requests and responses

Domain policy helpers will own the status and classification decisions.

## Boundaries

- `domain::task::policies` owns task status decisions for cancel requests, canceled attempts, successful attempts, failed attempts, and daemon recovery interruption.
- `crates/imglab-daemon/src/transport.rs` owns loopback route handling, recovery orchestration, cancellation marker IO, and task application owner calls.
- `crates/imglab-daemon/src/scheduler.rs` owns provider execution, attempt log IO, and scheduler tick execution.
- `library/tasks.rs` remains the SQLite repository/compatibility adapter behind the task application owner.

## Compatibility

The change must preserve:

- Cancel queued/retry-waiting task transitions to `canceled`.
- Cancel running/cancel-requested task transitions to `cancel_requested`.
- Recovery of running/cancel-requested task with committed outputs transitions to `completed`.
- Recovery of interrupted task without outputs transitions to `interrupted_retryable` while attempts remain below max, otherwise `interrupted_final`.
- Error classification values and timeline event payloads.
- Daemon API paths and response shape.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-core`
- `cargo test -p imglab-daemon`
- `scripts/check-architecture.sh`
- `openspec validate move-task-transition-decisions-to-domain --strict`
- `openspec validate --specs --strict`
