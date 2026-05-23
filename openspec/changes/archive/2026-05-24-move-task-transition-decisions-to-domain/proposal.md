# Proposal: Move Task Transition Decisions To Domain

## Problem

The DDD completion audit found that daemon recovery and cancel request handling still choose task transition statuses directly. Core already owns successful, failed, and canceled attempt status policies, but daemon transport still decides:

- cancel request target status for queued, retry waiting, running, and terminal tasks
- recovery status for running or cancel-requested tasks after daemon restart
- whether recovery interruption is retryable based on attempt count and max attempts

These are task state-machine decisions and should live in the task domain policy owner. The daemon should keep loopback transport, cancellation marker IO, scheduler ticking, and log IO.

## Goals

- Move cancel request and recovery transition decisions into `domain::task::policies`.
- Keep daemon transport and scheduler behavior unchanged.
- Preserve task API response shape, timeline events, retry classification, and task persistence semantics.
- Update architecture inventory and specs so task transition ownership is no longer recorded as an open gap.

## Non-Goals

- Do not change task output persistence or generation output creation.
- Do not change retry backoff timing, scheduler selection, cancellation marker file IO, or log IO.
- Do not split `library/tasks.rs` in this change.
- Do not change daemon route paths or response JSON.

## Impact

- Task state decisions are easier to test near the owning domain model.
- Daemon recovery and cancel paths delegate transition decisions to core domain policy helpers.
- The final DDD completion audit has stronger evidence for task state-machine ownership.
