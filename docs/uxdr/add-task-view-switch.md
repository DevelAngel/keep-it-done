---
status: proposed
date: 2026-05-18
---

# Add Task: Implicit Switch to All Open

## Context and Problem Statement

The "Add Task" button appears in every view when edit mode is active.
A newly created task carries only a summary — no dates, no time
estimate, no category beyond the default "Inbox". Several views
filter by attributes that a fresh task does not possess:

| View | Filter criterion | New task visible? |
|---|---|---|
| All Open | `!is_done()` | Yes — always |
| Recently Changed | within time window | Yes — just created |
| Upcoming | `due_date` or `start_date <= today` | Backlog only (collapsed) |
| Quick Wins | `time_estimate.is_some()` | No — invisible |
| What I Finished | `is_done()` | No — invisible |

When a user creates a task in Upcoming, it lands in the collapsed
backlog section; in Quick Wins, it vanishes without any trace. The
user must switch views to find and continue editing the task they
just created. For ADHD users, this "disappearing context" is
especially disorienting — the result of a deliberate action (tap
Add Task, type, press Enter) produces no visible feedback in the
current view.

How should task creation behave across views so that the new task
is always immediately visible and editable?

## Decision Drivers

- ADHD-friendly: the result of a user-initiated action must be
  visible — no silent disappearance
- Predictability: the same button should produce the same
  observable effect regardless of which view the user is in
- Minimal complexity: no per-view special cases, no new UI
  patterns, no conditional defaults that may surprise
- All Open as planning surface: the existing UXDR (Upcoming View)
  describes All Open as "the comprehensive planning surface" —
  task creation aligns with that role

## Considered Options

- Implicit switch to All Open on tap
- Auto-expand backlog / add backlog per view
- View-contextual defaults (pre-fill missing fields)
- Restrict Add Task to compatible views
- Toast notification with "jump to task" link

## Decision Outcome

Chosen option: "Implicit switch to All Open on tap", because it
guarantees visibility with zero per-view special logic and frames
All Open as the canonical creation surface.

### Behaviour

1. User taps the "Add Task" button (any view, edit mode active)
2. The active view switches to **All Open** immediately
3. The text input field opens and receives focus
4. User types summary, presses Enter
5. Task appears in All Open's "Inbox" category group
6. Task detail expansion opens automatically (existing behaviour)
7. User edits fields (dates, estimate, etc.) as needed
8. User navigates back to the previous view when ready — the task
   now satisfies that view's filter criteria if the relevant fields
   were set

The view switch happens **on tap** (before the input field opens),
not on submit. This ensures the user sees the destination context
before typing. If the user cancels (Escape or blur with empty
input), they remain in All Open — an acceptable side effect, since
the cancellation was intentional.

### Why Not the Other Options

**Auto-expand backlog / add backlog per view:** Solves Upcoming
(auto-expand the collapsed backlog) but not Quick Wins, which has
no backlog concept. Adding a backlog to Quick Wins introduces a
new structural element for one edge case. Each view would need its
own fallback logic — complexity scales with the number of views.

**View-contextual defaults:** Setting `due_date = today` in
Upcoming works well, but Quick Wins would need a `time_estimate`
default, and no universal default exists for "how long does this
take?" Wrong defaults that the user must immediately correct add
friction rather than reducing it.

**Restrict Add Task to compatible views:** Hides the button in
Upcoming, Quick Wins, and What I Finished. Eliminates the problem
but reduces capability — a user working in Upcoming who thinks of
a new task must first navigate to All Open manually. The implicit
switch achieves the same result with one fewer mental step.

**Toast notification:** A toast saying "Task created — in Backlog"
with a tap target introduces a new UI pattern (toasts), requires
the user to process an interruption, and still leaves the task
outside the visible list until the user acts on the toast.

### Consequences

- Good, because the new task is always visible — no disappearing
  context, no confusion
- Good, because behaviour is identical in all five views — fully
  predictable
- Good, because zero new UI patterns: view switching and Add Task
  both exist already
- Good, because All Open's role as planning surface is reinforced
- Good, because implementation is trivial — one signal update
- Neutral, because cancelling leaves the user in All Open — a
  minor unexpected position, but not disorienting
- Bad, because users lose their place in the previous view
  (scroll position, expanded task) — mitigated by the fact that
  the user initiated the action and can navigate back

## More Information

This decision interacts with:

- **UXDR: Upcoming View** — describes All Open as "the
  comprehensive planning surface" alongside Upcoming as the
  urgency check
- **UXDR: Upcoming Backlog Disclosure** — the backlog remains
  collapsed by default; this decision removes the need to
  auto-expand it on task creation
- **Concept: Web Edit** — originally scoped task creation as
  "out of scope (CLI only)"; the web Add Task feature was added
  later without updating the concept document
