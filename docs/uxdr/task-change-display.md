---
status: proposed
date: 2026-05-26
---

# Task Change Display: Layered External-Edit Notification

## Context and Problem Statement

When one family member changes a task — via the web UI in another browser or via the CLI — other users currently see stale data until they navigate or reload.
The server will soon broadcast `TaskChanged { id, actor }` events over the existing SSE channel (see ADR: Task Change Notification via Guard Callback).

How should the web UI display these external changes so that
(a) the task list stays current without manual reload,
(b) the user understands that something changed and optionally sees what,
(c) a user editing a task is warned before their changes collide with an external edit, and
(d) the notification does not create cognitive load during normal browsing?

## Decision Drivers

- ADHD-friendly: add salience where it matters (conflict), stay invisible where it does not (routine sync)
- Minimal visual channels: one signal per layer, not three competing signals for the same event
- No toast: recurring toast notifications were explicitly rejected in UXDR Flush Status LED as hostile to ADHD users
- Consistent patterns: reuse existing visual vocabulary (border accents, amber bars) rather than inventing new primitives
- Progressive disclosure: only interrupt the user when action is required — conflict during editing — not during passive browsing

## Considered Options

- Silent refetch only (no visual feedback)
- Sync LED (second indicator, bottom-left)
- Inline row highlight with fade-out
- Conflict banner in detail/edit panel
- Layered approach (silent refetch + highlight + conflict banner)

## Decision Outcome

Chosen option: "Layered approach", because no single mechanism covers all scenarios well.
A silent refetch keeps data current, an inline highlight shows what changed, and a conflict banner protects against data loss — each layer addresses a distinct scenario within the minimum necessary salience.

### Scenarios

| #  | Situation                                                   | Required response                                             |
| -- | ----------------------------------------------------------- | ------------------------------------------------------------- |
| S1 | User browses task list, another user mutates a task         | Data must update; subtle visual hint is helpful               |
| S2 | User has detail panel open, that task is externally changed | User needs to know; stale detail is misleading                |
| S3 | User is in edit mode on a task that is externally changed   | User must be warned; saving would overwrite the external edit |
| S4 | App is idle in background tab                               | Data should be current when user returns                      |

### Layer 1 — Silent Refetch (S1, S4)

When a `TaskChanged` event arrives via SSE, the client bumps an internal version signal (e.g. `external_change_version`).
The task list `Resource` already depends on version signals for local mutations (add, delete, complete, category);
adding one more dependency triggers an automatic refetch for the current view.

No visual feedback is needed for this layer.
The task list updates in place, just as it does after the user's own actions.

For S4 (background tab), the `EventSource` remains connected.
Events accumulate and bump the version signal.
When the user switches back to the tab, the Resource has already refetched or will refetch on the next reactive tick.

### Layer 2 — Inline Row Highlight (S1)

When a `TaskChanged` event arrives, the changed task's ID is added to a reactive `HashSet<Uuid>` of recently-changed IDs.
If the task is visible in the current view, its row receives a temporary left border accent:

```
Normal row:
│  ☐  Buy groceries            @home

Externally changed (highlight active):
┃  ☐  Buy groceries            @home
│
↑ border-l-2 border-l-sky-400, fades after 4 s
```

The highlight uses `border-l-sky-400` — a color not used by any existing visual signal (priority uses amber/rose/transparent, AI-involvement uses amber/violet).
Sky is neutral, distinct, and does not imply urgency or error.

After 4 seconds, the ID is removed from the set and the border fades out via `transition-colors duration-700`.
If the task is not visible in the current view (e.g. it belongs to a different view), the highlight is silently discarded — no deferred queue.

**Batching:** If multiple `TaskChanged` events arrive within a short window (e.g. a CLI batch update), all affected IDs are added to the set simultaneously.
The 4-second timer starts per ID from insertion time, so highlights fade independently.

### Layer 3 — Conflict Banner (S2, S3)

If the user has a task's detail panel open (expanded row) and a `TaskChanged` event arrives for that task's ID, a banner appears inside the panel:

```
┌──────────────────────────────────┐
│  ⚠ Von Papa geaendert.          │
│    Neu laden                     │
│──────────────────────────────────│
│  Erstellt:     2026-05-20       │
│  Kategorie:    Haushalt         │
│  ...                            │
└──────────────────────────────────┘
     ↑ border-t-2 border-t-amber-400
```

The banner uses `border-t-2 border-t-amber-400 bg-amber-950` — consistent with the edit-mode amber bar vocabulary already established in the UI.
The banner contains:

- Warning icon and text: "Von [actor] geaendert."
- Action link: "Neu laden" — refetches the task detail and closes the banner

**Edit-mode escalation:** If the user is actively in edit mode (not just viewing the detail panel), the banner text becomes more urgent:
"Von [actor] geaendert — Aenderungen pruefen." The "Neu laden" action reloads the task data and exits edit mode, discarding local unsaved changes.
This prevents silent overwrites.

**No modal dialog:** The banner is inline, not a modal.
The user can continue reading the panel or dismiss by clicking "Neu laden".
This avoids the cognitive disruption of a modal that demands immediate attention.

**Scope:** The conflict banner only appears for the currently open detail panel.
If the user has no panel open, this layer is inactive — Layer 1 (silent refetch) and Layer 2 (highlight) handle the update.

### Interaction of Layers

```
TaskChanged { id, actor } arrives via SSE
    │
    ├──► Layer 1: bump external_change_version
    │    → Resource refetches task list for current view
    │
    ├──► Layer 2: insert id into highlight set
    │    → task row shows sky border for 4 s (if visible)
    │
    └──► Layer 3: if detail panel is open for this id
         → show amber conflict banner inside panel
         → if edit mode: escalated warning text
```

All three layers fire independently for the same event.
They do not conflict visually because they target different UI regions:
Layer 1 affects data, Layer 2 affects the task row's left border, and Layer 3 affects the detail panel interior.

### Consequences

- Good, because Layer 1 ensures data is always current — no manual reload needed, no stale state
- Good, because Layer 2 provides a visual hint without interrupting the user — sky border is subtle, transient, and fades without requiring interaction
- Good, because Layer 3 prevents data loss during editing — the most critical scenario gets the most prominent signal
- Good, because each layer uses exactly one visual channel (data refresh / border accent / amber banner) — no redundant signals competing for attention
- Good, because the conflict banner reuses the amber vocabulary from edit mode — no new visual language to learn
- Good, because no toast, no modal, no second LED — the design avoids all primitives previously rejected as ADHD-hostile
- Neutral, because Layer 2 highlights are fire-and-forget — if the user is not looking at the list when the highlight appears, they miss it (acceptable: Layer 1 ensures data is correct regardless)
- Bad, because three layers add implementation complexity compared to silent refetch alone — mitigated by each layer being small and independent (a version signal, a reactive set, a conditional banner)
- Bad, because the highlight color (sky-400) adds a new color to the palette that must be documented and maintained — mitigated by it being used for a single, well-defined purpose

## Pros and Cons of the Options

### Silent refetch only

Bump version signal on `TaskChanged`; no visual feedback.

- Good, because zero cognitive load — the list updates like after any local action
- Good, because no new UI elements to design or maintain
- Bad, because the user has no indication that data changed externally — the list may "jump" without explanation
- Bad, because S2/S3 are unprotected — a user editing a task could unknowingly overwrite an external change

### Sync LED (second indicator, bottom-left)

A second status LED, mirroring the flush LED, that briefly flashes on external sync.

- Good, because consistent with the flush LED pattern
- Bad, because two LEDs create two attention points — the user must monitor both corners
- Bad, because the LED shows that _something_ changed but not _what_ — less informative than an inline highlight
- Bad, because it does not protect against edit conflicts (S3)

### Inline row highlight with fade-out

Changed task rows get a temporary border accent.

- Good, because it shows exactly which tasks changed
- Good, because the signal is spatially co-located with the relevant content
- Bad, because it does not update the underlying data — must be combined with a refetch mechanism
- Bad, because it does not protect against edit conflicts (S3)

### Conflict banner in detail/edit panel

An amber warning banner inside the open detail panel when the viewed task is externally changed.

- Good, because it protects the most critical scenario (edit conflict)
- Good, because it reuses existing amber visual vocabulary
- Bad, because it only addresses S2/S3 — the task list itself remains stale without a separate refetch mechanism
- Bad, because it is invisible when no detail panel is open

### Layered approach (chosen)

Combines silent refetch, inline highlight, and conflict banner.

- Good, because each scenario gets the appropriate level of notification — not more, not less
- Good, because layers are independent — any can be deferred without breaking the others
- Good, because the design is incrementally implementable (Layer 1 first, then 2, then 3)
- Bad, because three layers are more complex than one — each must be implemented, tested, and maintained independently

## More Information

### Related Decisions

- ADR: [Server-Sent Events for Server-to-Client Push](../adr/server-sent-events.md) — defines the push channel used to deliver `TaskChanged` events
- ADR: [Task Change Notification via Guard Callback](../adr/task-change-notification.md) — defines how mutations emit `TaskChanged` events automatically
- UXDR: [Flush Status LED](flush-status-led.md) — the first SSE consumer; establishes the no-toast, minimal-salience pattern

### Highlight Color Choice

`sky-400` was chosen because it is unused in the existing palette:

| Signal                | Color                  |
| --------------------- | ---------------------- |
| Priority A (high)     | amber-400              |
| Priority A (critical) | rose-400               |
| AI involvement        | amber-400 / violet-400 |
| Edit mode bars        | amber-400              |
| Flush LED success     | green-500              |
| Flush LED error       | red-500                |
| External change       | sky-400                |

Sky conveys "informational update" without implying urgency, error, or priority — matching the nature of the signal.

### Implementation Order

The layers can be implemented incrementally:

1. **Layer 1** — silent refetch: add `external_change_version` signal, wire SSE listener, add Resource dependency
2. **Layer 2** — inline highlight: add `changed_ids` reactive set, conditional border class on task row, 4 s timeout
3. **Layer 3** — conflict banner: compare open detail panel ID against incoming events, render amber banner conditionally

Each layer is independently useful and testable. Layer 1 is the minimum viable feature; Layers 2 and 3 add progressive polish.
