---
status: proposed
date: 2026-05-25
---

# Task Deletion: Placement, Trigger, and Safeguard

## Context and Problem Statement

Users cannot currently delete tasks through the Web UI. The server
function `delete_task` exists but has no UI trigger. Deletion is
permanent (file removed from disk on next flush) and irreversible
at the storage layer. How should the delete action be exposed so
that it is discoverable, hard to trigger accidentally, and does not
interrupt the user's flow with excessive confirmation friction?

## Decision Drivers

- ADHD-friendly: destructive actions must not rely on impulse
  control alone — a safeguard is needed, but it must not break
  task-management flow with modal interruptions
- Mobile-first: the trigger must be reachable with one thumb at
  375–428px width without precision targeting
- Discoverability: the action must be findable without memorizing
  gestures or hidden menus
- Consistency: placement should feel natural within the existing
  timeline detail expansion (UXDR: Timeline-Style Task Detail
  Expansion)
- Reversibility gap: unlike completion (toggle), deletion is
  permanent — the safeguard must match the severity
- Low accident rate: scrolling, tapping to expand, or editing
  fields must never accidentally trigger deletion

## Considered Options

- Trash-icon button in detail panel with undo-toast
- Trash-icon button in detail panel with inline confirm
- Swipe-to-delete on collapsed row
- Context menu (long-press / right-click)
- Delete as last timeline node with modal confirmation

## Decision Outcome

Chosen option: "Trash-icon button in detail panel with inline
confirm", because it combines discoverability (visible button),
low accident risk (two deliberate taps), zero modal interruption,
and works within the existing expansion pattern without new UI
infrastructure.

### Placement

The delete button appears **at the bottom of the expanded task
detail panel**, below the "Created" timeline node, separated by
`mt-6`. It is **outside** the timeline spine — no marker, no
connecting line — visually communicating that it is a meta-action,
not a task property.

```
┌─────────────────────────────┐
│  ● Priority                 │
│  │                          │
│  ● Due Date                 │
│  │                          │
│  ● ...                      │
│  │                          │
│  ● Created                  │
│                             │
│         [ 🗑 Delete ]        │  ← idle state
│         [ Confirm? ]        │  ← armed state
│                             │
└─────────────────────────────┘
```

### Interaction States

| State       | Appearance                                                                                                | Behavior                                                           |
| ----------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| **Idle**    | Ghost button: `text-slate-500 border border-slate-700 rounded-lg px-4 py-2`. Trash icon + "Delete" label. | First tap transitions to Armed.                                    |
| **Armed**   | Solid destructive: `bg-red-600 text-white rounded-lg px-4 py-2`. Label changes to "Confirm delete".       | Second tap dispatches `delete_task`. Auto-disarms after 3 seconds. |
| **Pending** | Same as Armed but with opacity pulse (`animate-pulse`). Label: "Deleting…". Pointer events disabled.      | Awaiting server response.                                          |

**Disarm rule:** If the user does not confirm within 3 seconds,
the button reverts to Idle. Tapping anywhere else in the detail
panel also disarms immediately. This prevents "I forgot I armed
it" scenarios — critical for ADHD users who context-switch
frequently.

### Visual Design

- **Idle color:** `text-slate-500` — low salience, does not
  compete with task properties. Visible but not attention-grabbing.
- **Armed color:** `bg-red-600 text-white` — red signals danger,
  demands conscious acknowledgment before the second tap.
- **Size:** `min-h-[44px] min-w-[44px]` — meets touch target
  guidelines (44×44 CSS px).
- **Position:** Centered below timeline, `mt-6 mb-2`. Horizontal
  centering ensures equal distance from both screen edges —
  unreachable during one-handed vertical scrolling.
- **No icon in Armed state:** removing the trash icon and showing
  only text ("Confirm delete") forces the user to re-read,
  preventing muscle-memory double-taps.

### After Deletion

- The expanded panel collapses.
- The task row is removed from the list (reactive refresh via
  `delete_task.version()`).
- No toast, no undo — deletion is permanent and the two-step
  confirm is the safeguard.

### Edit Mode Interaction

The delete button is visible in **both** view mode and edit mode.
Deletion is not an "edit" — it is a lifecycle action available
whenever the detail panel is open. This avoids hiding it behind
an extra mode toggle.

### Consequences

- Good, because two deliberate taps on the same element are nearly
  impossible to trigger accidentally during scrolling or field
  editing
- Good, because the 3-second auto-disarm protects distracted users
  without requiring them to manually cancel
- Good, because no modal — flow stays within the task list, no
  overlay to dismiss
- Good, because placement below the timeline avoids any change to
  the established timeline structure or reading order
- Good, because the Armed state's red background provides an
  unmistakable visual signal that a destructive action is pending
- Good, because the server function and reactive refresh
  infrastructure already exist — implementation is purely UI
- Neutral, because permanent deletion (no trash/archive) is
  acceptable for a family task manager where tasks are lightweight
  and re-creatable
- Bad, because users who want bulk deletion must delete one at a
  time — acceptable given the use case (family tasks, not
  enterprise backlogs)
- Bad, because no undo means a misconfirmed delete requires
  manual re-creation — mitigated by the deliberate two-step

## Pros and Cons of the Options

### Trash-icon button with undo-toast

Button in detail panel; single tap deletes immediately; toast
with "Undo" appears for 5 seconds.

- Good, because single-tap deletion is fast for intentional bulk
  cleanup
- Good, because undo provides a safety net without upfront friction
- Bad, because single-tap deletion with no upfront gate relies
  entirely on the user noticing and acting on the toast — poor
  fit for ADHD users who may have already scrolled away
- Bad, because toast infrastructure does not exist in the app and
  must be built (z-index layering, animation, auto-dismiss timer)
- Bad, because on mobile, toasts compete with the bottom
  navigation area and may be obscured

### Trash-icon button with inline confirm (chosen)

Two-step: tap to arm, tap again to confirm. Auto-disarm after
3 seconds.

- Good, because both taps happen on the same element — no eye
  movement to a separate confirmation target
- Good, because auto-disarm prevents stale armed state
- Good, because zero new infrastructure (no toast, no modal)
- Bad, because slightly slower than single-tap + undo for users
  who are certain they want to delete
- Bad, because the color transition from slate to red may be
  missed if the user is not looking at the button — mitigated
  by label text change

### Swipe-to-delete on collapsed row

Horizontal swipe reveals delete action behind the row.

- Good, because familiar from iOS/Android native apps
- Bad, because swipe gestures conflict with horizontal scrolling
  and browser back-navigation on mobile
- Bad, because gesture is invisible — zero discoverability for
  users who don't already know the pattern
- Bad, because Leptos/web touch event handling for swipe is
  complex and fragile across browsers
- Bad, because accidental swipes during vertical scrolling are
  common on small screens

### Context menu (long-press / right-click)

Long-press on mobile or right-click on desktop opens a menu
containing "Delete".

- Good, because hides destructive action behind an intentional
  gesture
- Bad, because long-press discoverability is near zero — users
  must be told it exists
- Bad, because long-press timing conflicts with text selection
  and browser context menus
- Bad, because requires building a context menu component that
  does not yet exist

### Delete as last timeline node with modal

A red timeline marker at the bottom labeled "Delete"; tapping it
opens a centered confirmation modal.

- Good, because integrates visually into the timeline
- Bad, because modals are flow-breaking — the user's attention is
  hijacked and must be returned, which is hostile to ADHD users
- Bad, because modals on mobile require careful positioning,
  backdrop handling, and focus trapping
- Bad, because placing "Delete" on the timeline implies it is a
  task property rather than a meta-action — semantically misleading

## More Information

**Existing server function** (`app/src/server/mod.rs`):

```rust
#[server(endpoint = "delete_task")]
pub async fn delete_task(id: Uuid) -> Result<(), ServerFnError> { ... }
```

**Existing action** (`app/src/lib.rs`):

```rust
let delete_task = ServerAction::<server::DeleteTask>::new();
```

The action is already wired into the reactive dependency chain
that triggers list refreshes. Implementation requires only:

1. A `DeleteButton` component with `RwSignal<ButtonState>` (Idle,
   Armed, Pending)
2. A `set_timeout` for 3-second auto-disarm
3. Dispatching `delete_task.dispatch(...)` on confirmed tap
4. Collapsing the detail panel after successful deletion

**Accessibility:** The button uses `<button>` with
`aria-label="Delete task"` in Idle and `aria-label="Confirm
deletion"` in Armed. Screen readers announce the state change.
The Armed state also sets `aria-live="assertive"` on a visually
hidden status region to announce "Press again to confirm deletion".

**Future extension:** If an archive/soft-delete is introduced
later, the inline-confirm pattern transfers directly — only the
server action and label text change. The interaction model remains
identical.
