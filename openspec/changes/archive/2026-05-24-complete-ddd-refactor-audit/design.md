# Design: Complete DDD Refactor Audit

## Approach

The audit records completion evidence against four authoritative inputs:

1. `docs/architecture/ddd-systematic-code-review.md`.
2. `openspec/specs/core-ddd-architecture/spec.md`.
3. `openspec/specs/performance-code-health/spec.md`.
4. `docs/architecture/ddd-boundary-inventory.md`.

The audit does not redefine success around the already-completed implementation. It checks that the current state satisfies the original objective: using OpenSpec to complete the refactor implied by the code review.

## Evidence Model

Each audit row records:

- requirement or finding area,
- current evidence source,
- completion decision,
- residual risk or deferred work.

Residual work is acceptable only when it is future product capability, optional hardening, or explicitly bounded compatibility cleanup. It is not acceptable when it leaves required DDD boundary migration incomplete.

## Validation

Validation must include:

- `openspec validate complete-ddd-refactor-audit --strict`,
- `openspec validate --specs --strict`,
- architecture guardrail check,
- Rust and desktop behavior-preserving checks,
- `git diff --check`.
