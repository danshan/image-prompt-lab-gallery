# Desktop Refresh Policy

## Purpose

This document records the refresh and polling ownership extracted from `StudioAppController` into `useStudioRefreshPolicy`. It is part of the `systematic-ddd-architecture-refactor` OpenSpec change.

The policy keeps refresh behavior explicit while the UI remains a compact local-first desktop app. The goal is not to remove polling immediately. The goal is to make polling, debounce, and cross-workflow refresh fan-out owned by a focused controller boundary rather than by root shell composition.

## Owner

Code owner:

```text
apps/desktop/src/app/hooks/controllers/refresh-policy.ts
```

Composition caller:

```text
apps/desktop/src/app/StudioAppController.tsx
```

`StudioAppController` passes current workflow state, controller actions, refs, and setters into `useStudioRefreshPolicy`. The hook owns refresh effects and cleanup. Screen components continue to render from props and do not own refresh orchestration.

## Policy

| Workflow concern | Trigger | Refresh behavior | Notes |
| --- | --- | --- | --- |
| Initial Tauri startup | `runningInTauri` becomes true | Refresh libraries and silently check updates | Startup orchestration remains outside individual screens. |
| Gallery query | library path or gallery query changes | Debounced gallery refresh using `GALLERY_REFRESH_DEBOUNCE_MS` | Prevents immediate repeated full gallery reads while filters change. |
| Selected album contents | library path, selected album, album kind, or album list changes | Refresh selected album contents | Albums own add/remove state, but refresh policy coordinates the effect. |
| Album add drawer candidates | drawer opens or add query changes | Refresh album-add candidates | Candidate query stays separate from main gallery query. |
| Library switch cleanup | library path changes | Clear metadata polling timers and completed task keys | Prevents stale task/review timers from crossing library boundaries. |
| Library open | running in Tauri with active library | Refresh albums, suggestions, daemon health, and tasks | Establishes the workspace baseline after a library is selected. |
| Task queue polling | active library and active view | Poll tasks at queue foreground interval or background interval | Queue view uses foreground polling and task detail refresh. Other views use background polling. |
| Task detail selection | selected task changes | Load selected task detail or clear detail | Keeps task detail state tied to selection. |
| Settings logs | active view is settings | Refresh app logs | Logs are settings-owned, not part of Gallery or Queue state. |
| Review suggestion selection | selected suggestion changes | Initialize review form and refresh suggestion history | Review form state follows current suggestion. |
| Gallery asset selection | selected asset or version changes | Load Tauri detail or preview detail, or clear detail | Detail load remains separate from Gallery list query. |

## Follow-Up Candidates

- Replace task polling with daemon events or server-sent updates if the local API grows an event channel.
- Add tests around refresh fan-out once controller hook tests are introduced.
- Reduce full gallery refresh after task completion by refreshing affected assets or read-model deltas.
- Split `useStudioRefreshPolicy` again if it starts mixing unrelated lifecycle, task polling, review, and detail concerns.
