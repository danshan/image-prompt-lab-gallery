# Design: Remove Daemon Task Service Accessor

## Overview

This change narrows daemon runtime access by replacing direct task calls through the concrete local service with calls through `TaskUseCase`.

`TaskUseCase` already exposes create, list, detail, status update, event append, attempt append, attempt completion, output append, reorder, retry, and duplicate behavior. Therefore production transport and tests can use `state.tasks()` without introducing new task behavior.

## Boundaries

- Daemon transport owns HTTP parsing, authentication, response mapping, recovery orchestration, and scheduler invocation.
- `TaskUseCase` owns task repository operations for daemon task workflows.
- `LocalLibraryService` remains the SQLite adapter behind the task repository port.

## Compatibility

The change must preserve:

- Daemon loopback API endpoint behavior and response shape.
- Recovery event and status behavior.
- Scheduler wait-reason update behavior.
- Task reorder, cancel, retry, duplicate, and output lookup behavior.

## Verification

- `cargo fmt --all --check`
- `cargo test -p imglab-daemon`
- `scripts/check-architecture.sh`
- `openspec validate remove-daemon-task-service-accessor --strict`
