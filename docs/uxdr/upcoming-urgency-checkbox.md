---
status: proposed
date: 2026-05-22
---

# Upcoming: Urgency Signal on Checkbox Border

## Context and Problem Statement

The Upcoming view groups tasks by temporal proximity (Today,
Tomorrow, This Week, etc.). Tasks can land in an earlier group
for two independent reasons:

1. **Estimate-driven lead time** — a Day1/Day2 estimate shifts
   the task ahead of its due date so work can start early enough
2. **Start-date pull** — a `start_date` earlier than the computed
   attention date pulls the task forward

The previous design showed a text label ("start by {date}") for
case 1, but nothing for case 2. Users seeing a task in "Today"
with a due date weeks away had no visual clue _why_ it was there
or _how urgent_ it really was. Meanwhile, tasks that genuinely
need completion today looked identical to pulled-forward tasks
with ample slack.

How should the Upcoming view communicate urgency at the scan
level — distinguishing "must finish now" from "can start now
but has time" — without adding text noise?

## Decision Drivers

- ADHD-friendly: add salience to the urgent, don't subtract
  from the rest (UXDR: Priority Visual Weight)
- One signal, one meaning: a single visual channel per
  distinction, not redundant encodings
- Orthogonal to priority: urgency (time pressure) and priority
  (importance) are independent dimensions and must not collide
  visually — priority uses the left border, urgency must use
  a different location
- Progressive disclosure: the scan level needs one bit
  ("urgent or not"); detailed dates live in the expanded panel
- Works for all pull-forward reasons: estimate-driven,
  start-date-driven, and overdue

## Considered Options

- Text label showing due date (replace "start by" with "due")
- Binary symbol (dot or chevron) on flexible tasks
- Accent on due-in-group tasks' checkbox border (chosen)
- Accent on the task summary text weight

## Decision Outcome

Chosen option: "Accent on checkbox border", because it places
the urgency signal directly on the action point (the completion
toggle), is orthogonal to the priority-A left border, and
scales cleanly to a four-tier model without adding text.

### Urgency Model

Each task in the Upcoming view carries one of four urgency
levels, derived from its grouping reason and time estimate:

| Urgency      | Condition                                     | Meaning                           |
| ------------ | --------------------------------------------- | --------------------------------- |
| **None**     | Soft + sub-day estimate (≤ 2 h)               | Quick task with ample buffer      |
|              | OR `ReadyToStart` (no due date)               |                                   |
|              | OR any task in the Later group                |                                   |
| **Standard** | Soft + estimate ≥ HalfDay                     | Has time but needs planning       |
|              | OR Soft + no estimate                         | Has time but scope unknown        |
| **Hard**     | Due in this group + sub-day estimate (≤ 2 h)  | Due soon, completable in a gap    |
| **High**     | Due in this group + estimate ≥ HalfDay        | Due soon, needs dedicated time    |
|              | OR due in this group + no estimate            | Due soon, scope unknown           |
|              | OR Overdue + estimate ≥ HalfDay               | Past due, needs planning          |
|              | OR Overdue + no estimate                      | Past due, scope unknown           |

**Later is always None:** tasks more than two weeks away
don't warrant urgency signals — accents there would
create noise without actionable information. When a task
moves into NextWeek, it receives its proper urgency level.

"Soft" means the task was pulled into an earlier group by
`start_date` rather than by deadline pressure.

**Rationale for the two axes:** The model separates two
independent questions: _Does this task have deadline
pressure?_ (soft vs. hard) and _Can I squeeze it in or does
it need planning?_ (sub-day vs. ≥ HalfDay/unknown). Their
combination yields four distinct user actions: ignore → note
for later → grab in a gap → plan actively.

### Visual Treatment

| Urgency      | Size  | Border          | Extra       |
| ------------ | ----- | --------------- | ----------- |
| **None**     | 16 px | 2 px, slate-600 | —           |
| **Standard** | 20 px | 3 px, slate-600 | —           |
| **Hard**     | 20 px | 3 px, slate-600 | —           |
| **High**     | 20 px | 3 px, slate-600 | subtle glow |

Standard and Hard share the same visual treatment — four
semantic levels collapse to three visual levels. This keeps
the scan simple (ADHD-friendly: fewer competing patterns)
while the data model retains the full distinction for
potential future use.

**Form only, no color accent.** Urgency is communicated
purely through checkbox size and border thickness — no color
shift. This avoids visual competition with the view's
identity color and the priority-A left border. The signal
is perceptible but not intrusive.

**Alignment.** All checkboxes sit inside a fixed 20 × 20 px
container with flex centering. The smaller None checkbox
(16 px) is centered within the same column, so all tasks
align regardless of urgency level. The same technique
applies to the priority-A left border: all tasks carry a
3 px left border (transparent when not priority A) to
prevent horizontal shifts.

### Removed: "start by" Text Label

The text label "start by {date}" is removed. It was only shown
for estimate-driven shifts and answered a question ("when to
start?") that is redundant once the task is already in Today's
group. The checkbox accent replaces it with a more scannable,
universally applicable signal.

The `attention-label` data-testid and the `attention_label`
component prop are removed. E2E tests are updated to assert
urgency via checkbox styling instead.

### Data Model Change

The `Option<NaiveDate>` (attention label) in the grouped task
tuple is replaced by a four-variant `Urgency` enum
(`None`, `Standard`, `Hard`, `High`).

### Consequences

- Good, because urgency signal sits on the action point — the
  eye is drawn to exactly where the user clicks to complete
- Good, because orthogonal to priority-A left border: two
  independent axes, two distinct visual locations, no collision
- Good, because the four tiers (None/Standard/Hard/High) map directly
  to the user's decision: ignore → note → grab in a gap → plan
- Good, because start-date-pulled tasks are finally visually
  distinguished from deadline-driven tasks in the same group
- Good, because overdue tasks always get maximum urgency,
  regardless of estimate
- Good, because removing the text label reduces visual noise
  while increasing information density (scannable vs. readable)
- Good, because form-only signalling avoids color competition
  with the view's identity hue and the priority-A accent
- Bad, because the exact due date is no longer visible at the
  scan level — users who want to know _how much_ slack remains
  must expand the task detail panel

## Pros and Cons of the Options

### Text label showing due date

Replace "start by {date}" with "due {date}", color-coded by
urgency (accent for due-in-group, slate for has-slack).

- Good, because provides exact date information at scan level
- Good, because works for all pull-forward reasons
- Bad, because text on every task row creates visual noise,
  especially at 5 tasks per group
- Bad, because two visual channels (text + color) for one
  distinction violates the one-signal principle

### Binary symbol on flexible tasks

A small chevron or dot indicating "this task has slack."

- Good, because minimal visual weight
- Bad, because marking the _flexible_ tasks requires the user
  to mentally invert ("no symbol = urgent") — adds cognitive
  load rather than reducing it
- Bad, because a single bit cannot distinguish "due soon, easy"
  from "due soon, needs planning"

### Accent on checkbox border (chosen)

Three-tier border treatment on the completion toggle.

- Good, because urgency is encoded at the interaction point
- Good, because four tiers match four distinct user actions
- Good, because orthogonal to all existing visual channels
- Bad, because requires learning that a glowing checkbox means
  "needs planning" — not immediately self-explanatory

### Accent on task summary text weight

Bolder or warmer text for urgent tasks.

- Good, because no additional UI element
- Bad, because `slate-100` vs `white` is barely perceptible
  on dark backgrounds
- Bad, because competes with the strikethrough channel used
  for completed tasks in other views

## More Information

**Relationship to Priority Visual Weight UXDR:** Priority uses
the left border (single channel, presence/absence). Urgency
uses the checkbox border (different location, four tiers). The
two signals are fully orthogonal — a Priority-A task can be
Soft, Hard, or High urgency.

**Relationship to Upcoming View UXDR:** This decision modifies
the data model defined there. The `attention_date` field in the
grouped task tuple is replaced by `Urgency`. Grouping logic
(which group a task lands in) is unchanged.
