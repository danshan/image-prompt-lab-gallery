# Performance Cleanup Status

> Source review: `docs/PERFORMANCE_REVIEW.md`
> OpenSpec change: `refactor-performance-code-health`

## Finding Map

| Finding | Current status | OpenSpec task coverage | Notes |
|---|---|---|---|
| C-01 Gallery load N+1 | Applies | 1.2, 2.2, 2.6 | Batch preload current version, event, version count, tags and pending review counts. |
| C-02 Search N+1 | Applies | 1.2, 2.5, 2.6 | Search must reuse batch read model or SQL filters. |
| C-03 Album order comparator SQL | Applies | 1.3, 2.4 | Sort orders must be loaded before sorting. |
| C-04 Manual album membership N+1 | Applies | 1.3, 2.3 | Album asset ids must be loaded once. |
| C-05 Synchronous `generate_image` | Applies to legacy path | 4.1 | Daemon path exists, but legacy command remains synchronous. |
| C-06 Monolithic React component | Applies | 4.2, 4.3 | Refactor by workflow boundaries. |
| C-07 Gallery image eager loading | Applies | 4.4 | Immediate fix is lazy loading and async decoding. Thumbnails are deferred unless checkpoint requires them. |
| C-08 Search refetch per keystroke | Applies | 4.5 | Debounce IPC-backed query refresh. |
| H-01 Missing DB indexes | Applies | 2.1 | Add idempotent hot-path indexes. |
| H-02 Full-file hashing buffer | Applies | 1.4, 3.1, 3.2 | Replace with streaming digest. |
| H-03 Full-file image dimensions read | Applies | 1.5, 3.3 | Replace with bounded prefix/header read. |
| H-04 Sequential integrity/repair hashing | Partially deferred | 3.1, 3.2, 6.1, 6.2 | Streaming fixes memory. Parallel hashing/cache requires separate evidence after checkpoint. |
| H-05 Album library lookup opens every DB | Deferred | 6.1, 6.2 | Schema change to add `library_id` to albums is broader than current hot path and needs separate design if still relevant. |
| H-06 Generation job polling cleanup/backoff | Applies | 4.8, 5.4, 5.5 | Implement cleanup/backoff for remaining polling paths. |
| H-07 API refresh waterfall | Applies | 4.7 | Batch independent refreshes only. |
| H-08 Missing React memoization | Applies | 4.2, 4.6 | Prefer component extraction and stable derived values before broad memo wrapping. |
| M-01 Lineage sequential lookup | Deferred | 6.1, 6.2 | Not on current gallery hot path. Consider recursive CTE in a follow-up if lineage depth becomes measurable. |
| M-02 Repeated library lookup by id | Partially applies | 2.5, 6.2 | Gallery/search path still resolves by list in search. Fix where touched, broader registry helper can follow. |
| M-03 `attach_tag` query count | Deferred | 6.1, 6.2 | Useful but outside this change's primary performance path unless tests show review accept bottleneck. |
| M-04 Album batch add max query per asset | Applies | 6.1, 6.2 | Include if album workflow evidence shows material cost; otherwise document as follow-up. |
| M-05 Daemon coarse lock | Applies | 1.7, 5.1, 5.2 | Keep transport stable and reduce lock scope. |
| M-06 Scheduler deep clone | Applies | 1.7, 5.3 | Add cheap no-work path. |
| M-07 Generation jobs map growth | Mostly obsolete | 4.1, 5.x | Current desktop state no longer has `generation_jobs`; verify no equivalent unbounded cache remains. |
| M-08 App logs scans temp | Applies | 1.6, 5.6, 5.7 | Restrict to app-owned roots. |
| M-09 Smart album preview recompute | Applies | 4.5, 4.6 | Memoize or debounce local preview. |
| M-10 Inline available providers | Applies | 4.6 | Memoize derived provider list. |
| L-01 Hex digest uses `format!` per byte | Applies | 3.1, 3.2 | Can be folded into streaming digest cleanup. |
| L-02 `has_task_output` count vs exists | Deferred | 6.1, 6.2 | Low-risk micro-optimization, not needed for current acceptance criteria. |
| L-03 Provider cancel filesystem polling | Deferred | 6.1, 6.2 | Provider control semantics need separate design to avoid breaking cancel behavior. |
| L-04 Daemon client timeout too short | Applies | 5.4, 5.5 | Make timeout context-aware. |
| L-05 Vite code splitting | Deferred | 6.1, 6.2 | Build-time bundle optimization can follow after component split evidence. |

## Current Direction

The first implementation pass will focus on findings marked `Applies`, while deferred findings remain visible in the checkpoint. SQLite replacement is not part of this change unless the sufficiency checkpoint proves the optimized SQLite path cannot meet target workloads.

## Checkpoint

SQLite sufficiency is tracked in `docs/PERFORMANCE_SQLITE_CHECKPOINT.md`. The checkpoint uses an ignored 10k-asset synthetic test so storage replacement can be decided from repeatable evidence after the hot-path fixes land.
