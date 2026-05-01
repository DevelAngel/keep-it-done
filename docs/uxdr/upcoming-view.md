---
status: proposed
date: 2026-05-01
---

# Upcoming: Deadline-Grouped View for Time-Sensitive Tasks

## Context and Problem Statement

Tasks can carry a due date (deadline) or a start date (earliest actionable day). In the existing views, these temporal signals are invisible at the list level — MyDay groups by category and treats a task due tomorrow identically to one with no date at all. Users must expand individual tasks to discover looming deadlines or newly actionable start dates.

For ADHD users in particular, *time blindness* means approaching deadlines slip out of awareness unless the system actively surfaces them. How should time-sensitive tasks be presented so that temporal urgency is immediately visible and scannable?

## Decision Drivers

- ADHD-friendly: temporal urgency must be the primary visual axis — not buried inside detail panels
- Forward-looking: the view answers "what's coming at me?" — completed tasks are excluded
- Minimal overlap with other open-task views: category grouping is deliberately absent here; temporal and topical views serve different mental models
- Consistent with existing grouped-view patterns: reuse header styling and context filtering
- Five views remain within the swipe-navigation comfort zone (UXDR: View Switching notes degradation beyond ~6)

## Considered Options

- Deadline-grouped view (dedicated 5th view)
- Urgency badges in MyDay (overlay on existing view)
- Combined timeline view (due + start + completed, chronological)
- Calendar overlay (date picker with task dots)

## Decision Outcome

Chosen option: "Deadline-grouped view", because it isolates the temporal mental model into its own dedicated surface — giving deadlines full salience without cluttering the category-grouped open-tasks view.

### View Identity

| Property | Value |
|---|---|
| Name | Upcoming |
| Color | Rose (`from-rose-500 to-rose-700`) |
| Subtitle | "Open tasks with dates · grouped by urgency" |
| Empty message | "Nothing on the horizon." |

View ordering and position are defined in a separate concept document (see: view-order concept).

### Inclusion Criteria

A task appears in Upcoming if and only if:

1. `status` is `ToDo` (open), **and**
2. at least one of:
   - `due_date` is set (any value — past, present, or future), or
   - `start_date <= today` (the task has become actionable)

Tasks with only a future `start_date` (and no `due_date`) are excluded — they are not yet actionable and have no deadline pressure.

### Temporal Grouping

Tasks are assigned to exactly one group based on their **effective date** — `due_date` if present, otherwise `start_date`:

| Group | Condition | Sort within |
|---|---|---|
| **Overdue** | `due_date < today` | due_date ascending (oldest overdue first) |
| **Today** | `due_date == today` | priority descending (A before B before C) |
| **This Week** | `due_date` within remaining calendar week (tomorrow through Sunday) | due_date ascending |
| **Next Week** | `due_date` within the following Mon–Sun | due_date ascending |
| **Later** | `due_date > next week` | due_date ascending |
| **Ready to Start** | `start_date <= today` and `due_date` is None | start_date ascending (longest-waiting first) |

**Week boundary:** ISO 8601 (Monday = start of week). "This Week" excludes today (today has its own group) and includes up to the coming Sunday.

**Tie-breaking within identical dates:** priority descending, then UUID ascending (creation order).

Only groups containing at least one task are rendered — empty groups are omitted.

### Visual Treatment

**Group headers** follow the established pattern (see UXDR: Quick Wins): `text-sm font-semibold text-slate-400` with `border-t` separators. Non-collapsible.

**Overdue group accent:** the "Overdue" header receives `text-rose-400` instead of `text-slate-400` to signal urgency without adding a new visual channel to individual task rows.

**Priority-A border:** present, using the view's rose accent: `border-l-[3px] border-l-rose-500`.

**Soft-date indicator:** tasks whose date is marked `soft: true` (parsed guess rather than explicit input) display a subtle `~` prefix before the date chip — signalling uncertainty without a new icon.

### Data Model

New `TaskListData` variant:

```rust
enum DeadlineGroup {
    Overdue,
    Today,
    ThisWeek,
    NextWeek,
    Later,
    ReadyToStart,
}

TaskListData::DeadlineGrouped(Vec<(DeadlineGroup, Vec<(Uuid, task::Infos)>)>)
```

Server function: `fetch_upcoming(today: NaiveDate)` — accepts today explicitly (passed from `today_signal`) to ensure midnight-rollover correctness.

### Backlog Counter

Below the last visible group, a muted footer line shows the number of open tasks that are *not* visible in Upcoming — i.e., tasks without any date (`due_date` is None and `start_date` is None or in the future):

```
 ── 14 tasks without deadline in backlog ──
```

**Styling:** `text-xs text-slate-500 text-center py-3` with thin `border-t border-slate-700` separator. Not an interactive link — pure information.

**Purpose:** prevents the false comfort of an empty or short Upcoming list. A user with 2 upcoming tasks but 40 undated tasks is informed that work exists beyond the temporal horizon — nudging them to assign dates or visit the broader overview.

**Empty-state variant:** when Upcoming shows zero tasks, the empty message ("Nothing on the horizon.") is followed by the counter if backlog > 0: "Nothing on the horizon. 14 tasks in backlog."

### Context Filtering

Context filtering applies identically to other views: `apply_filter()` removes tasks whose contexts do not intersect with the active filter set, then removes empty groups. The filter panel and `active_filters` map gain a `View::Upcoming` key. The backlog counter is unaffected by context filters — it always reflects the total undated open-task count.

### Consequences

- Good, because deadlines are now scannable at a glance — the most urgent tasks are always at the top
- Good, because ADHD time blindness is directly addressed: the system shows *how close* each deadline is, not just *whether* one exists
- Good, because "Ready to Start" surfaces tasks that would otherwise languish in MyDay without urgency signal
- Good, because the view is forward-looking only — no completed tasks, no backward audit (that's Recent Changes)
- Good, because the rose mono-hue gradient is instantly distinguishable from all other views — one color, one identity (see UXDR: View Order mono-hue principle)
- Good, because the swipe navigation handles 5 views comfortably (UXDR: View Switching)
- Good, because the backlog counter prevents false comfort — an empty Upcoming view still communicates that undated work exists
- Neutral, because tasks with both a due_date and start_date <= today appear only in their due_date group — "Ready to Start" is not duplicated
- Bad, because a task with a far-future due_date appears in "Later" even if it just became startable — the due_date dominates, potentially hiding the start signal
- Bad, because "This Week" and "Next Week" are calendar-anchored; a task due Monday night appears in "Next Week" even on Sunday evening — users must learn the boundary

## Pros and Cons of the Options

### Deadline-grouped view (chosen)

Dedicated 5th view showing only date-bearing open tasks, grouped by temporal proximity.

- Good, because full screen dedicated to temporal urgency — maximum salience
- Good, because grouping directly encodes the question "how soon?"
- Good, because no visual or data model conflict with existing views
- Bad, because adds one more view to the swipe sequence (5 total)
- Bad, because tasks appear in both the open-tasks view (by category) and Upcoming (by date) — potential confusion about "where is my task?"

### Urgency badges in MyDay

Add a date chip or colored dot to tasks in MyDay that have approaching deadlines.

- Good, because no new view — zero navigation cost
- Good, because deadline info is co-located with the task in its primary view
- Bad, because adds a second visual signal competing with the priority-A border — violates one-signal principle
- Bad, because "approaching" requires a threshold definition that differs per user
- Bad, because MyDay category grouping scatters deadline tasks across groups — cannot scan by urgency

### Combined timeline view (due + start + completed)

A chronological timeline showing all dated tasks regardless of status, anchored on today.

- Good, because provides complete temporal picture
- Bad, because completed tasks add noise when the goal is forward planning
- Bad, because past-and-future in one view requires complex bidirectional scrolling
- Bad, because fundamentally different from the quick-scan pattern of other views

### Calendar overlay

A mini calendar with dots indicating task density per day, tappable to show that day's tasks.

- Good, because familiar calendar mental model
- Bad, because calendar UI requires significant screen space on mobile
- Bad, because interaction model (tap day → see tasks) is slower than grouped scanning
- Bad, because implementation complexity is disproportionate to the information density needed

## More Information

**Relationship to the open-tasks view:** tasks with dates appear in *both* Upcoming (grouped by urgency) and the category-grouped open-tasks view. This is intentional — the two views answer different questions about the same tasks. Upcoming is the urgency check; the open-tasks view is the comprehensive planning surface.

**Relationship to Quick Wins:** a task can appear in both Quick Wins (has estimate) and Upcoming (has date). No conflict — the views slice the task set along orthogonal axes.

**Future consideration:** if the "Overdue" group grows large for neglectful users, a secondary "overdue for >7 days" subheading or a count badge may help — deferred until real usage data exists.
