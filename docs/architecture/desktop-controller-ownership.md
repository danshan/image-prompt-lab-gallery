# Desktop Controller Ownership

## Purpose

This document records the `StudioAppController.tsx` ownership split for the systematic DDD architecture refactor.

The controller remains the desktop composition root. It should wire workflow state, derived state, actions, and screens. It should not become the primary owner of refresh policy, workflow state semantics, or domain decisions.

## Current Boundaries

| Boundary | Owner | Responsibility |
| --- | --- | --- |
| Library and settings state | `workflows/settings` controller hooks | Library selection, settings sections, library form inputs, pending library actions, app operation state. |
| Gallery selection state | `hooks/controllers` plus `workflows/gallery` state helpers | Gallery query, selection, inspector, lightbox, detail load state. |
| Album workflow state | `hooks/controllers` plus `workflows/albums` helpers | Album list state, selected album, add-to-album drawer query and selection. |
| Review workflow state | `hooks/controllers` plus `workflows/review` helpers | Suggestion selection, review form, suggestion history, regeneration state. |
| Task generation state | `hooks/controllers` plus task workflow actions | Draft generation tasks, queue detail, composer inputs, pending task actions. |
| Refresh and polling policy | `hooks/controllers/refresh-policy.ts` | Startup refresh, gallery debounce, album refresh, metadata polling cleanup, queue polling, logs refresh, selected detail loads. |

## Root Controller Rule

`StudioAppController.tsx` may coordinate cross-workflow actions, but each recurring policy must have a named owner:

- Workflow state changes stay in workflow modules or workflow controller hooks.
- Runtime calls stay in action hooks or Tauri adapters.
- Repeated refresh/polling effects stay in `useStudioRefreshPolicy`.
- Pure state transformations stay in workflow helper modules.

If a new feature adds another cluster of `useEffect` calls or multi-step state transitions to the root controller, it should first add a workflow-owned controller hook or extend an existing one.

## Remaining Risk

The root controller is still large because it composes all workbench screens and actions. This is acceptable for this change because the high-risk refresh policy has been extracted and documented. The next meaningful split should be based on workflow ownership, not file length alone.
