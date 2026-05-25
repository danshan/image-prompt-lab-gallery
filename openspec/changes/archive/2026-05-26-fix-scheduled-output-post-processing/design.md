## Context

Scheduled generation run completion is reconciled in daemon schedule ticks after the linked image task reaches `completed`. The run already records output counters, but the core requirement is broader: each output asset must be added to the job target manual album, job tags must become canonical asset tags, and repeated reconciliation must remain idempotent.

The ownership boundary remains unchanged. Daemon owns schedule runner orchestration and post-processing; core owns album membership, canonical tag persistence, run output upsert, and counter calculation through existing use cases and repositories.

## Goals / Non-Goals

**Goals:**

- Ensure completed scheduled image tasks apply target album membership to every output asset.
- Ensure job tags are written as canonical asset tags.
- Ensure run output rows and counters reflect actual post-processing results.
- Ensure repeating schedule reconciliation does not create duplicate memberships, tags, or run outputs.
- Add tests that inspect persisted album and tag state, not only run counters.

**Non-Goals:**

- No change to schedule API request or response shapes.
- No change to resource library schema or migration behavior.
- No provider behavior change.
- No desktop UI change beyond existing run history behavior.

## Decisions

### 1. Keep post-processing in schedule reconciliation

The daemon schedule runner remains the owner of completed task reconciliation because it has the schedule job context, target album, tags, run id, and linked task detail. Moving this into task execution would couple generic image generation tasks to schedule-specific behavior.

### 2. Use existing idempotent core operations

Album membership uses existing batch add semantics, tag application uses canonical asset tag APIs, and run outputs use `upsert_run_output`. This keeps retry/restart behavior simple and avoids new persistence concepts.

### 3. Test persisted state directly

Counters alone can hide bugs where run output rows are updated but actual library organization is missing. The test must query album contents and gallery/detail tags after reconciliation, then run reconciliation again and verify counters remain stable.

## Risks / Trade-offs

- Album deletion during post-processing may surface as a failed reconcile path -> keep existing core validation/error behavior and avoid creating partial fake success.
- Repeated reconcile may return completed run again -> acceptable as long as persisted outputs, membership and tags remain idempotent.
- Multiple output assets may be produced in the future -> implementation must iterate all task output links, not assume one output.

## Migration Plan

No migration is required. Existing scheduled runs without post-processed outputs remain recoverable through the next schedule reconciliation when their linked task is completed and output links are present.
