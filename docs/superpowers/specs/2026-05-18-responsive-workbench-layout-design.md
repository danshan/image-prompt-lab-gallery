# Responsive Workbench Layout Design

## Context

The desktop app currently uses a fixed three-column workbench and several page-local grids with hard minimum tracks. This works on wide desktop windows, but compact windows can produce cramped controls, hidden detail surfaces, popover overlap, and panels that are technically present but hard to use.

This design treats `960px` as the first-class compact desktop minimum. The goal is not a phone layout. The goal is for every top-level page to remain usable at compact laptop and tablet-width desktop windows without component overlap, unrecoverable hidden actions, or broken text layout.

## Goals

- Re-evaluate every top-level page: Gallery, Albums, Review, Queue, and Settings.
- Preserve desktop-tool information density on wide windows.
- Make `Workspace` the priority surface when width is constrained.
- Allow both Sidebar and Inspector to collapse.
- Replace page-specific ad hoc breakpoints with shared layout patterns.
- Keep changes reviewable by separating layout behavior from business data flow.

## Non-Goals

- Do not build a full phone-first mobile app.
- Do not redesign the product visual language.
- Do not change Rust core, Tauri command semantics, SQLite schema, or provider behavior.
- Do not refactor unrelated data loading or command orchestration while implementing the layout.
- Do not introduce daemon, IPC, cloud sync, or multi-user scope.

## Selected Approach

Use a shell-first collapse system.

The workbench should define shared responsive behavior first, then each page should adapt its local panels to that shell. This is more maintainable than custom breakpoints per page, and it avoids losing too much desktop density by forcing every page into a single-column layout at `960px`.

## Responsive Shell

### Wide Desktop

At `1280px` and above, the app keeps the full three-column structure:

```text
Library Sidebar | Workspace | Inspector
```

The Sidebar and Inspector can remain visible. Workspace content may use multi-column page layouts where appropriate.

### Compact Desktop

From `960px` to `1279px`, the app enters compact desktop mode:

```text
Collapsed Sidebar Rail | Workspace | Collapsed Inspector Rail or Drawer
```

Rules:

- Workspace is the only stable primary column.
- Sidebar collapses to a narrow rail that preserves library access and top-level navigation.
- Inspector is no longer assumed to be a permanent right column. It opens as a contextual drawer or detail rail.
- Page-local secondary panels must not depend on fixed `280px` or `320px` tracks.
- Page headers, toolbar controls, and row actions must wrap or stack instead of overlapping.

### Below First-Class Minimum

Below `960px`, the app does not need a complete phone-optimized workflow, but it must remain recoverable:

- No incoherent component overlap.
- No primary action should become permanently unreachable.
- Single-column stacking is acceptable.
- Horizontal overflow is acceptable only inside explicit data/code/log preview regions.

## Page Rules

### Gallery

Gallery keeps the same workflow:

```text
query controls -> filter chips -> gallery grid -> selected asset detail
```

Compact desktop behavior:

- Search toolbar becomes multi-row. Text search takes the first row when needed.
- Primary action, filter controls, sort controls, and status text wrap in the second row.
- Gallery cards use container-aware grid sizing rather than a fixed card minimum that can squeeze the main column.
- Selecting an asset opens or updates the Inspector drawer. Asset detail must remain reachable without a permanent right column.
- Long prompts, titles, provider names, and tags must truncate or wrap within their card boundaries.

### Albums

Albums uses a shared split-workspace pattern:

```text
album list | album detail
```

Compact desktop behavior:

- Album detail is the primary surface.
- Album list can collapse into a selector/list panel above the detail surface.
- The create album UI should become an inline panel or drawer-like panel in compact mode. It must not use an absolute popover that covers critical list or header controls.
- Album header actions wrap into a stable action row.
- Album asset cards reuse the Gallery card rules.

### Review

Review also uses the shared split-workspace pattern:

```text
suggestion list | suggestion detail form
```

Compact desktop behavior:

- Review detail is the primary surface.
- Suggestion list can collapse into a selector/list panel.
- Batch actions and add-to-album controls stack or wrap above the list.
- The review form switches from two columns to one column or an `auto-fit` form grid.
- Field-level actions such as regenerate controls may wrap within the label row but must not push inputs outside their container.
- History, confidence, and task mirror sections must use internal wrapping for long text.

### Queue

Queue has three substantial panels:

```text
Batch Composer | Tasks Queue | Task Detail
```

Wide desktop can keep all three visible. Compact desktop should not simply stack all three panels vertically, because the page becomes too long and hard to operate.

Compact desktop behavior:

- Use a local segmented control or tabs for `Compose`, `Queue`, and `Detail`.
- The selected panel is the primary workspace content.
- Selecting a task should make `Detail` available and may switch to it when appropriate.
- Queue row actions wrap inside the row.
- JSON input, log tail, and output previews use internal scroll regions.

### Settings

Settings keeps separate `Libraries` and `Logs` sections.

Compact desktop behavior:

- Settings tabs remain visible at the top and may wrap if needed.
- Libraries table becomes row cards instead of a fixed grid table.
- Each library row card shows name, path, status, and actions in stacked regions.
- Long paths use truncation plus full value availability through title or copy-friendly text.
- Icon actions remain grouped and reachable.
- Logs browser stacks list and preview vertically. The preview gets a bounded height and internal scrolling.

## Shared Layout Components

Implementation should introduce small layout-only components or class patterns rather than rewriting data flow:

- `WorkbenchShell`: owns top-level responsive shell structure.
- `SidebarRail`: compact navigation representation.
- `InspectorDrawer`: contextual detail surface for compact mode.
- `ResponsiveSplit`: shared two-panel layout for Albums and Review.
- `ResponsiveToolbar`: shared wrapping toolbar behavior.
- `ResponsiveTableCards`: table-to-card pattern for Settings libraries.

These components should not own Rust command calls, gallery query semantics, review mutation logic, or task orchestration.

## CSS Tokens And Rules

Centralize responsive constants:

- compact breakpoint: `960px`
- wide breakpoint: `1280px`
- sidebar full width
- sidebar collapsed width
- inspector full width
- inspector collapsed width
- minimum panel width
- minimum card width
- toolbar control minimum width

Global layout rules:

- Every grid/flex child that can contain text must set `min-width: 0` where needed.
- Long paths, checksums, prompt text, JSON, and logs must have explicit wrapping, truncation, or internal scroll behavior.
- Drawers and popovers must have viewport-constrained width and height.
- Buttons and controls may wrap, but must not overlap adjacent content.
- Page sections should use unframed layouts or single-level panels. Avoid nested card structures.

## Error And Empty States

Responsive changes must preserve existing loading, empty, and recoverable error states:

- Gallery no assets.
- No asset selected.
- Detail loading or unavailable.
- No albums.
- No pending suggestions.
- No tasks.
- No registered libraries.
- No logs.

In compact mode, empty states should appear in the active panel area and should not require opening Inspector unless the state is specifically detail-related.

## Validation

Static validation:

```text
npm run build
npm test
```

Visual validation:

- Capture each top-level view at `1440px`, `1180px`, `960px`, and `900px`.
- Verify no overlap, clipped controls, or unreachable primary actions.
- Verify text handling for long library paths, prompts, schema prompt JSON, checksums, log paths, and task IDs.

Interaction validation:

- Open and close the Sidebar in compact mode.
- Open and close the Inspector drawer in compact mode.
- Select a Gallery asset and inspect detail without a permanent right column.
- Create album UI does not cover critical actions.
- Review field regeneration controls remain reachable.
- Queue `Compose`, `Queue`, and `Detail` panels are all reachable.
- Settings library row actions and logs preview are reachable.

## Implementation Notes

The current `apps/desktop/src/main.tsx` and `apps/desktop/src/styles.css` hold most desktop rendering and styling. Implementation should keep the first patch focused on layout:

- Extract layout-only pieces where it reduces risk.
- Avoid changing command inputs or DTO shapes.
- Avoid mixing layout fixes with business behavior changes.
- Prefer shared class patterns over duplicating page-specific media queries.

The expected outcome is a maintainable responsive foundation that can absorb future Gallery, Albums, Review, Queue, and Settings improvements without reintroducing fixed-width layout failures.
