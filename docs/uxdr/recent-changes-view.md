---
status: accepted
date: 2026-04-25
---

# Recent Changes: Day Grouping and History Pagination

## Context and Problem Statement

The Recent Changes view shows tasks that were recently modified. The initial implementation used a flat 24-hour rolling window with no visual structure. Users could not perceive *when* changes happened, and there was no way to explore older history. At midnight, the view silently shifted its contents without updating labels.

## Decision Drivers

- ADHD-friendly: changes must be scannable at a glance — temporal grouping reduces cognitive load
- No dead clicks: every "load more" must produce visible new content
- Clean server/client separation: server delivers data, client handles presentation
- Midnight correctness: day labels must stay accurate without manual refresh

## Considered Options

- Rolling 24 h window (flat list)
- Fixed calendar-day window with empty-day pagination
- Calendar-day window with data-driven pagination

## Decision Outcome

Chosen option: "Calendar-day window with data-driven pagination", because it combines a predictable initial view (anchored on today) with a load-more action that always yields visible results.

**Initial window:** today + 2 calendar days queried from the server. Only days that contain actual changes are shown — empty days are omitted entirely. When no changes exist in the window, an empty-state message appears with a "load more" button so the user can always look further back.

**Load more:** the server skips empty days and returns only the next N days that actually contain data. This avoids dead clicks where the view expands but nothing new appears. The button is shown both in the populated and in the empty state.

**Day labels** ("Today", "Yesterday", weekday + date) are computed on the client from stored dates, not pre-rendered strings — keeping data separate from presentation.

**Midnight rollover:** the client checks periodically whether the UTC date changed and triggers a refetch with fresh labels.

**AI involvement indicators:** a left border marks AI involvement per task — amber if AI made the most recent change, violet if AI was involved but a human acted last. Because the left border is reserved for this signal, the priority-A accent border used in other views is suppressed here (see UXDR: Priority Visual Weight).

### Consequences

- Good, because day headers give instant temporal orientation when data exists
- Good, because "load more" always surfaces new tasks — no empty-day noise
- Good, because dates in the data model enable midnight-safe label recomputation
- Good, because full refetch on "load more" avoids accumulator complexity
- Bad, because full refetch re-sends the initial 3 days on every "load more" — acceptable at family scale
- Bad, because UTC-based day boundaries may differ from local midnight by a few hours — acceptable for a self-hosted single-timezone deployment

## More Information

**Server pagination:** the client requests a number of extra days beyond the initial window. The server always returns the 3 calendar days, then scans backwards and collects up to that many distinct older days that have data — empty days are skipped.

**Client grouping:** the client groups tasks by date. Only days with data get a header — no pre-populated empty day placeholders. When the entire result is empty, the empty state with the "load more" button is shown instead.
