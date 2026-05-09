---
status: proposed
date: 2026-05-09
---

# Upcoming: Collapsible Backlog Disclosure

## Context and Problem Statement

The Upcoming view shows a muted footer line when open tasks exist
without a date: "── 3 tasks without deadline in backlog ──". This
communicates *how many* undated tasks exist but not *which* ones.
Users who notice the counter must switch to All Open and mentally
filter for undated tasks to identify them — a context switch that
is especially costly for ADHD users.

How should Upcoming surface backlog task identities without adding
a new view or cluttering the timeline-focused layout?

## Decision Drivers

- ADHD-friendly: the answer to "which tasks?" must appear where
  the question arises — no view switching, no mental search
- Minimal disruption: Upcoming remains a timeline view; backlog
  tasks are secondary information, not the primary content
- Existing patterns: the category-collapse toggle in All Open
  provides a proven disclosure pattern (chevron + toggle)
- Five-view limit: no new view may be added (UXDR: View Switching)
- Consistent with UXDR: Upcoming — backlog count is unaffected by
  context filters; the disclosed task list follows the same rule

## Considered Options

- Collapsible backlog section in Upcoming (inline disclosure)
- Backlog badge as shortcut to filtered All Open view
- Tooltip / popover on hover or long-press

## Decision Outcome

Chosen option: "Collapsible backlog section", because the answer
appears exactly where the question arises, reuses an established
interaction pattern, and requires the least cognitive effort.

### Behaviour

The static footer text is replaced by an interactive disclosure
group:

- **Collapsed (default):** a single-line button showing
  `▸ Backlog · 3 tasks`. Styled consistently with deadline-group
  headers: `text-sm font-semibold text-slate-400` with a
  `border-t border-slate-700` separator above.
- **Expanded:** the chevron rotates to `▾`, and backlog tasks
  render below the header using the standard `<TaskItem>`
  component — fully interactive (expandable, completable).
- **State lifetime:** the collapsed/expanded signal is local to
  the component render cycle. Switching away from Upcoming and
  back resets it to collapsed. No persistence needed.

### Data Changes

`fetch_upcoming` returns the full backlog task list instead of
only a count:

```rust
// Before
(Vec<(DeadlineGroup, Vec<(Uuid, task::Infos)>)>, usize)

// After
(Vec<(DeadlineGroup, Vec<(Uuid, task::Infos)>)>, Vec<(Uuid, task::Infos)>)
```

`TaskListData::DeadlineGrouped` mirrors this change.

Backlog tasks are sorted by priority descending (A first), then
UUID ascending (creation order) — matching the "Today" group sort
so that the most important undated tasks surface first.

### Context Filtering

Context filters apply to backlog tasks: only tasks matching the
active filter appear when the section is expanded. The toggle
label adapts to show both visible and total count —
`▸ Backlog · 2 of 5 tasks` — so the user always knows how many
undated tasks exist overall, even when a filter hides some.

This amends the original UXDR rule that the backlog count is
"unaffected by context filters." The total count still appears
(as the denominator), but the disclosed list respects filters to
avoid showing irrelevant tasks.

### Empty-State Variant

When Upcoming has zero deadline-grouped tasks but backlog exists,
the empty message reads "Nothing on the horizon." and the
collapsible backlog section appears below it — giving users a path
to their undated tasks even from an otherwise empty view.

### Consequences

- Good, because "which tasks?" is answered with one tap, right
  where the question arises
- Good, because the collapsed default keeps Upcoming focused on
  its temporal purpose — backlog is opt-in detail
- Good, because backlog tasks are fully interactive: users can
  expand, complete, or inspect them without leaving Upcoming
- Good, because the disclosure pattern matches All Open's
  category collapse — no new interaction to learn
- Neutral, because expanding a large backlog may push deadline
  groups out of view — acceptable since the user explicitly chose
  to expand
- Bad, because backlog tasks lack temporal grouping, making them
  visually distinct from the rest of Upcoming — mitigated by the
  clear "Backlog" header label

## Pros and Cons of the Options

### Collapsible backlog section (chosen)

Inline disclosure below deadline groups, collapsed by default.

- Good, because zero context switch — answer where the question is
- Good, because reuses existing collapse pattern and TaskItem
- Good, because collapsed default preserves Upcoming's focus
- Bad, because adds interaction state to a previously static
  element

### Backlog badge as shortcut to filtered All Open

Clicking the backlog counter navigates to All Open with an
"undated" filter pre-applied.

- Good, because All Open's category grouping adds structure to
  backlog tasks
- Bad, because context switch breaks flow and requires a new
  filter type
- Bad, because the user must navigate back to Upcoming afterward
- Bad, because a temporary filter state needs visual marking to
  avoid confusion

### Tooltip / popover on hover or long-press

Hovering or long-pressing the counter shows task titles in a
floating panel.

- Good, because no layout change to Upcoming
- Bad, because hover is unavailable on mobile
- Bad, because long-press is undiscoverable
- Bad, because the popover is read-only — no task interaction
- Bad, because introduces a new UI pattern with no precedent in
  the app

## More Information

This decision amends the backlog-counter section of
UXDR: Upcoming (upcoming-view.md), which defined the counter as
"not an interactive link — pure information." The disclosure
replaces the static counter while preserving its informational
purpose.
