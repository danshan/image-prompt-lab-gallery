# Proposal: Remove Daemon Task Service Accessor

## Problem

The DDD boundary inventory still documents `DaemonState::service()` as a compatibility accessor for task paths. Production daemon transport and daemon tests use this generic local-service accessor for task operations even though `DaemonState::tasks()` already exposes the application task owner.

This keeps the daemon boundary wider than necessary and makes future runtime code more likely to bypass task use-case ownership.

## Goals

- Replace daemon task calls through `service()` with `tasks()`.
- Remove `DaemonState::service()` once no daemon code needs it.
- Preserve daemon HTTP response shape, scheduler behavior, recovery behavior, and task persistence semantics.
- Update the architecture inventory and specs to reflect the narrowed daemon boundary.

## Non-Goals

- Do not redesign scheduler policy.
- Do not change task status transition semantics.
- Do not change daemon loopback API endpoints or JSON payloads.
- Do not change SQLite schema or task persistence layout.

## Impact

- Daemon production code and tests call the task application owner for task operations.
- The generic local-service accessor is removed from `DaemonState`.
- Remaining daemon compatibility work can focus on broad prelude imports and runtime view mapping rather than task entrypoints.
