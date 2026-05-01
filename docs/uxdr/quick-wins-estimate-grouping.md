---
status: accepted
date: 2026-05-01
---

# Quick Wins: Group by Time Estimate Instead of Flat Sort

## Context and Problem Statement

Quick Wins shows open tasks that carry a time estimate, intended
for "I have 15 minutes — what can I knock out?" moments. The
initial implementation sorted tasks by estimate ascending (shortest
first) but rendered them as a flat list. Users scanning the view
had to read individual estimate labels to find where one duration
bucket ended and the next began — the visual structure did not
match the mental model of "pick a time slot, then pick a task."

How should Quick Wins present tasks so that the time-slot decision
is immediate and scanning within a slot is effortless?

## Decision Drivers

- ADHD-friendly: temporal grouping reduces scanning to a two-step
  decision (pick bucket, pick task) rather than a linear search
- Proven pattern: day grouping in Recent Changes already
  demonstrates that grouped headers with a shared data shape
  work well for this app (see UXDR: Recent Changes)
- Non-interactive groups: collapsible sections add a state
  management burden that conflicts with the "glance and grab"
  use case of Quick Wins
- Consistency: group headers should use the same visual treatment
  (font, spacing, separator) as Recent Changes day headers

## Considered Options

- Flat sorted list (status quo)
- Grouped by time estimate, non-collapsible
- Grouped by time estimate, collapsible
- Grouped by category, sorted by estimate within each category

## Decision Outcome

Chosen option: "Grouped by time estimate, non-collapsible",
because it directly maps the view's structure to the user's
decision process — choose a time budget first, then choose a task
within that budget — without adding interaction complexity.

**Group headers** display the full duration label ("15 minutes",
"30 minutes", "1 hour", etc.) using the `Display` trait of
`TimeEstimate`. Only groups containing at least one task are
shown — empty duration buckets are omitted.

**Visual treatment** mirrors the Recent Changes day headers:
`text-sm font-semibold text-slate-400`, with `border-t` separators
between groups. No collapse toggle, no task count badge.

**Server-side grouping:** `fetch_quick_wins` returns
`Vec<(TimeEstimate, Vec<(Uuid, Infos)>)>` — tasks are sorted by
estimate ascending and then chunked into contiguous groups. The
`TaskListData::Flat` variant is replaced by
`TaskListData::EstimateGrouped` since no other view used the flat
variant.

**Subtitle** updated from "shortest first" to "grouped by
duration" to reflect the new layout.

### Consequences

- Good, because the two-step scan (bucket then task) is faster
  than linear scanning — especially when multiple estimate levels
  are populated
- Good, because non-collapsible groups mean zero state to manage;
  the view is always fully visible
- Good, because the pattern reuses the DayGrouped visual language,
  so users familiar with Recent Changes recognise the structure
  instantly
- Good, because empty groups are omitted — no visual noise for
  unused duration buckets
- Good, because context filtering (`apply_filter`) works per group
  and removes empty groups, keeping the view clean after filtering
- Neutral, because groups with a single task still get a header —
  a minor visual overhead justified by consistent structure

## Pros and Cons of the Options

### Flat sorted list (status quo)

Tasks sorted by time estimate ascending, rendered as one
continuous list.

- Good, because simple implementation and rendering
- Bad, because the boundary between "15 min tasks" and "30 min
  tasks" is invisible — users must read each task's estimate
- Bad, because no visual anchor for the "I have X minutes"
  question that motivates opening the view

### Grouped by time estimate, non-collapsible (chosen)

Tasks clustered under duration headers, always expanded.

- Good, because instant visual orientation — headers answer "what
  fits in X minutes?" at a glance
- Good, because no interaction complexity — just scroll
- Bad, because long lists produce many groups and more scrolling
  than a flat list

### Grouped by time estimate, collapsible

Same grouping, but users can collapse individual buckets.

- Good, because users could hide irrelevant buckets
- Bad, because collapse state must be persisted or re-decided on
  each visit — cognitive overhead for a view meant to be
  transient ("grab a task, close")
- Bad, because collapsed groups hide tasks, defeating the
  purpose of a "quick scan" view

### Grouped by category, sorted by estimate within

Tasks grouped by their category (like My Day), with estimate-based
ordering within each group.

- Good, because reuses the existing `Grouped` data model
- Bad, because the primary decision axis in Quick Wins is time,
  not category — grouping by category forces the user to scan
  every category to find all 15-minute tasks
- Bad, because conflicts with the established category grouping
  in My Day and What I Finished, where the grouping matches a
  different mental model

## More Information

**Relationship to time-estimate-input UXDR:** the chip group
input (UXDR: Time Estimate Input) produces the `TimeEstimate`
enum values that this grouping consumes. The fixed variant set
directly enables clean grouping — no bucketing logic needed.

**Relationship to priority-visual-weight UXDR:** within each
estimate group, priority-A tasks still carry the amber left
border per the existing priority accent rule.
