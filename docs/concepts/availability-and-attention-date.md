# Availability and Attention Date

## Abstract

Two additions to the task data model — an `Availability` scheduling
constraint and a computed `attention_date` — enable the Upcoming view to
surface tasks based on when work should *begin*, not just when it is
*due*. Combined, they produce a deterministic daily overview without AI
interpretation.

## Context

The Upcoming view groups open tasks by temporal distance to their
`due_date`: Overdue, Today, This Week, Next Week, Later, Ready to Start.
Three problems emerge from this pure-deadline grouping:

1. **"Later" is a dead zone.** A 2-day task due in 10 days and a task
   due in 6 months occupy the same bucket. The system gives no signal
   that the first task needs attention *this week*.

2. **`TimeEstimate` is decorative.** The estimate (15m–2d) is displayed
   in Quick Wins and the detail panel, but has no functional role in
   scheduling. A task estimated at 2 days does not surface earlier than
   a 15-minute task with the same due date.

3. **Weekend is a context, not a constraint.** "Weekend" is an optional
   user-defined context string — topical, not temporal. The system
   cannot distinguish "can only be done on weekends" from "I prefer
   weekends". Since `today.weekday()` is a fact the system already
   knows, weekend-awareness should be structural.

## Core Idea

Give each task two new pieces of information:

- **Availability** — a hard scheduling constraint declaring *which days*
  the task can be worked on. Three values: `Anytime` (default),
  `WeekdayOnly` (Mon–Fri), `WeekendOnly` (Sat–Sun).

- **Attention date** — a computed date answering *"when should I start
  thinking about this?"*. Derived deterministically from `due_date`,
  `time_estimate`, and `availability`. No AI, no heuristics — pure
  date arithmetic.

The Upcoming view uses `attention_date` instead of raw `due_date` for
group assignment. A 2-day weekend-only task due May 25 (Sunday) gets
`attention_date = May 24` (Saturday) and appears in "This Week" on
May 19 — instead of languishing in "Later" until May 18.

## Availability

### Enum

```rust
#[derive(Default)]
enum Availability {
    #[default]
    Anytime,
    WeekdayOnly,
    WeekendOnly,
}
```

### Semantics

| Value | Meaning | Example |
|---|---|---|
| `Anytime` | No day-type constraint | "Review PR" |
| `WeekdayOnly` | Only Mon–Fri | "Call dentist", "School pickup" |
| `WeekendOnly` | Only Sat–Sun | "Mow the lawn", "Family bike ride" |

### Placement

`Availability` lives on `Details` (alongside `due_date`, `start_date`,
`time_estimate`) — it is a scheduling property, not an identity
property.

Serialization: `#[serde(default)]` ensures backward compatibility.
Existing task JSON files without the field deserialize to `Anytime`.

### Distinction from Context

Availability and context serve different purposes:

| | Availability | Context |
|---|---|---|
| Nature | Hard scheduling constraint | Soft filter preference |
| Question answered | "When *can* this be done?" | "When do I *want* to see this?" |
| System use | Attention date computation | View filtering |
| Example | `WeekendOnly` — cannot call on weekday | "Weekend" — prefer to think about it then |

A task can be `Availability::Anytime` *and* carry a "Weekend" context.
The two are orthogonal. Existing "Weekend" contexts are not migrated —
they remain valid filter preferences.

## Attention Date

### Definition

The attention date is the earliest date on which the user should begin
working on a task to finish by its due date, given its estimated
duration and availability constraint.

### Computation

```
fn attention_date(task) -> Option<NaiveDate>:
    // Manual start_date always wins — user override.
    if task.start_date is set:
        return Some(start_date)

    // Need both due_date and time_estimate for computation.
    if task.due_date is None or task.time_estimate is None:
        return task.due_date   // fall back to due_date if present

    lead_days = task.time_estimate.lead_days()
    candidate = task.due_date - lead_days

    // Adjust for availability: shift candidate to an eligible day.
    match task.availability:
        Anytime     -> return candidate
        WeekdayOnly -> return prev_weekday_or_same(candidate)
        WeekendOnly -> return prev_weekend_or_same(candidate)
```

### Lead Days by Estimate

| TimeEstimate | lead_days | Rationale |
|---|---|---|
| Min15–HalfDay | 0 | Completable on the due date itself |
| Day1 | 1 | Need the day before due date |
| Day2 | 2 | Need two days before due date |

Sub-day estimates produce `lead_days = 0`, meaning the attention date
equals the due date. Only multi-day tasks create a meaningful lead
window.

### Day-Type Adjustment

When `availability` restricts eligible days, the candidate date shifts
**backward** to the nearest eligible day:

- `prev_weekday_or_same(date)`: if `date` is Sat → Fri; if Sun → Fri;
  else → date.
- `prev_weekend_or_same(date)`: if Mon → previous Sun; if Tue–Fri →
  previous Sun; if Sat/Sun → date.

For multi-day estimates with availability constraints, the algorithm
must ensure enough eligible days exist between `attention_date` and
`due_date`:

**Example — 2d WeekendOnly, due May 23 (Fri):**

1. `candidate = May 23 - 2 = May 21` (Wed)
2. Need 2 weekend days before May 23
3. Previous weekend: May 17 (Sat) + May 18 (Sun) = 2 eligible days
4. `attention_date = May 17`

**Example — 2d WeekendOnly, due May 25 (Sun):**

1. `candidate = May 25 - 2 = May 23` (Fri)
2. May 24 (Sat) + May 25 (Sun) = 2 eligible days
3. `attention_date = May 24`

### Fallback Hierarchy

| Inputs present | attention_date |
|---|---|
| `start_date` (manual) | `start_date` — always wins |
| `due_date` + `time_estimate` | computed (due - lead, adjusted) |
| `due_date` only | `due_date` — no lead time possible |
| `time_estimate` only | None — no date anchor |
| Neither | None — task has no temporal signal |

## Impact on Upcoming View

### Group Assignment

Currently: group by `due_date`.
Proposed: group by `attention_date` when available, else by `due_date`.

The displayed date on the task card remains the `due_date` — the user
sees the actual deadline. Only the *bucket* changes.

### New Grouping Behavior

| Scenario | Before | After |
|---|---|---|
| 2d task due in 10 days | Later | This Week (attention in 8 days) |
| 15m task due in 10 days | Later | Later (no lead time) |
| 2d weekend task due Fri | Later | This Week (need prev. weekend) |
| Task with manual start_date ≤ today | Ready to Start | Ready to Start (unchanged) |
| Task without time_estimate | Later | Later (unchanged — no lead) |

### Attention Indicator

When a task appears in an earlier group due to its attention date (i.e.,
`attention_date < due_date` group), a subtle label communicates why:

> "start by Thu" · "due May 25"

This prevents confusion when a task labeled "due May 25" appears in the
"This Week" group on May 12. The indicator is derived, not stored.

## Design Principles

**Deterministic over intelligent.** The system does arithmetic, not
interpretation. Given the same inputs, the same output — always.

**Declarative over procedural.** The user declares constraints
(availability, estimate); the system derives consequences (attention
date). No "schedule this for Saturday" workflow.

**Progressive enrichment.** A task with only `due_date` works exactly
as before. Adding `time_estimate` activates lead time. Adding
`availability` activates day-type awareness. Each input unlocks more
precision — nothing breaks without it.

**Manual override wins.** An explicit `start_date` always takes
precedence over the computed `attention_date`. Power users who know
better can override; casual users get sensible defaults.

## Out of Scope

- **Capacity planning** — the system does not know how many hours the
  user has available on a given day. It surfaces tasks, not schedules.
- **Recurring availability patterns** — only the fixed Weekday/Weekend
  split. Custom patterns (e.g., "Tue+Thu only") are deferred.
- **Time-of-day constraints** — "morning only" or "after 6pm" are not
  modeled. The granularity is calendar days.
- **Auto-assignment of availability** — the system never guesses.
  Default is `Anytime`; the user must explicitly set constraints.
- **Migration of existing "Weekend" contexts** — they remain as filter
  preferences. Users set `availability` explicitly on relevant tasks.

## Implementation Steps

Each step is independently committable and testable.

### Step 1 — Availability enum and Details field

**Crate:** `kid-types`

- Add `Availability` enum with `Default`, `Serialize`, `Deserialize`,
  `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`
- Add `availability: Availability` to `Details` with
  `#[serde(default)]`
- Add to `TaskDetails` trait: `availability()`, `set_availability()`
- Implement on `Details`, `Task`, `(Uuid, Details)`
- Add to `DetailsPatch` (feature-gated `rpc`/`ssr`)

**Test:** round-trip serialize a task with each `Availability` value;
verify existing JSON without the field deserializes to `Anytime`.

### Step 2 — TimeEstimate::lead_days()

**Crate:** `kid-types`

- Add `pub fn lead_days(self) -> u32` to `TimeEstimate`
- Min15–HalfDay → 0, Day1 → 1, Day2 → 2

**Test:** exhaustive match over all variants, assert expected values.

### Step 3 — attention_date() computation

**Crate:** `kid-types` (feature `ssr`)

- Add `pub fn attention_date(&self, today: NaiveDate) -> Option<NaiveDate>`
  to `Details` (or as a free function in a scheduling module)
- Implement the computation described above, including day-type
  adjustment helpers (`prev_weekday_or_same`, `prev_weekend_or_same`,
  and the multi-day availability-aware backtrack)
- `start_date` override: if `start_date` is set, return it directly

**Test:**
- 2d + WeekdayOnly due on Monday → attention_date = previous Thursday
- 2d + WeekendOnly due on Friday → attention_date = previous Saturday
- 1d + Anytime due on Wednesday → attention_date = Tuesday
- HalfDay + Anytime due on Wednesday → attention_date = Wednesday
- Manual start_date set → always returns start_date
- No due_date → returns None

### Step 4 — Update fetch_upcoming grouping

**Crate:** `kid-app` (feature `ssr`)

- In `fetch_upcoming()`, compute `attention_date` per task
- Use it for `DeadlineGroup` assignment instead of raw `due_date`
- Keep `due_date` as `sort_date` for within-group ordering (sort by
  actual deadline, not by when attention starts)

**Test:** create tasks with varying estimates and availabilities; assert
they land in the expected groups for a given `today`.

### Step 5 — CLI support

**Crate:** `kid-cli`

- Add `--availability anytime|weekday|weekend` to `create` and
  `update` subcommands (optional, default omitted)
- JSON output includes `availability` field
- Display in `show` output

**Test:** create task with `--availability weekend`, verify JSON.

### Step 6 — Web UI: availability input

**Crate:** `kid-app`

- Add 3-button toggle in task detail expansion:
  `Anytime` / `Weekdays` / `Weekend`
- Position: below `time_estimate` chips (scheduling block)
- Submit on tap (same pattern as priority toggle)
- Read-only mode: show as text label if not `Anytime`

### Step 7 — Upcoming view: attention indicator

**Crate:** `kid-app`

- When a task's group was determined by `attention_date` (not
  `due_date`), show a secondary label: "start by {attention_date}"
- Styling: `text-xs text-slate-500` — subordinate to the due date chip
- Only shown when `attention_date != due_date` group assignment

## Future Considerations

- **Buffer days:** add a configurable margin (e.g., +1 day) to
  `lead_days` for users who want extra preparation time.
- **"This Weekend" group:** split "This Week" into "This Workweek" and
  "This Weekend" when `today` is Mon–Fri, enabling day-type-aware
  scanning without filtering.
- **Availability in Quick Wins:** filter Quick Wins by today's day type
  — on Saturday, suppress `WeekdayOnly` tasks (or sort them down).
- **Finer availability granularity:** `MorningOnly`, `EveningOnly`,
  or custom day sets — deferred until the day-level model proves
  insufficient.
