---
status: accepted
date: 2026-05-26
---

# Search and Filter: Unified Panel with Independent Dimensions

## Context and Problem Statement

The filter panel already provides context-based filtering via a chip bar (see **Category vs. Context** UXDR).
Adding a search input creates a second filtering dimension.
How should search and context filtering coexist in the UI?
Should they share a panel or live in separate controls?
How do they compose — AND or OR?
And which views need search at all?

## Decision Drivers

- ADHD-friendly: one place for "narrow down the list", not two competing controls
- Minimal new affordances — reuse existing UI patterns where possible
- Search and context filtering are independent dimensions — the user should not have to think about interaction order
- Mobile-first: screen space is limited, especially when the on-screen keyboard is open
- The filter button is already a learned interaction — users know where to find it

## Considered Options

- Separate search icon button with its own panel
- Always-visible search bar above the task list
- Search input inside the existing filter panel
- Search as a separate view (dedicated search results page)

## Decision Outcome

Chosen option: "Search input inside the existing filter panel", because it groups all narrowing controls in one place, reuses the existing filter toggle, and avoids a second button in the already compact header toolbar.
The search input appears above the context chips — visually primary because text search is the more powerful tool.

### Scope

Search is available in **All Open** and **What I Finished** only.
The other views have small, time-scoped result sets where search adds little value:

| View             | Search  | Rationale                                                |
| ---------------- | ------- | -------------------------------------------------------- |
| Upcoming         | No      | Already filtered by date — small result set              |
| Quick Wins       | No      | Already filtered by estimate — small result set          |
| All Open         | **Yes** | Largest view, hundreds of tasks, free-text recall needed |
| What I Finished  | **Yes** | Growing archive, "did we already do X?" questions        |
| Recently Changed | No      | Time-scoped to 2–4 days — inherently small               |

### Composition: Search AND Filter

Search and context filtering compose with AND semantics:

1. Server delivers the full task list for the current view
2. Context filter is applied first — removes tasks not matching selected contexts
3. Search is applied second — removes tasks not matching the fuzzy query
4. The resulting set is displayed, grouped by category as usual

Both dimensions reduce independently.
A task must satisfy _all_ active context filters AND _all_ search words to remain visible.
This is consistent with the existing AND logic within context filtering itself (multiple chips = all must match).

**Example flow:**

1. User opens "All Open" — 150 tasks across 8 categories
2. User taps filter button → panel opens with search input and context chips
3. User taps `@abends` → list reduces to 22 tasks tagged `@abends`
4. User types "zahn" in search → list reduces to 2 tasks containing "zahn" in their summary, among those 22
5. User clears search → back to 22 `@abends` tasks
6. User deselects `@abends` → back to 150 tasks

### Filter Panel Layout

When the filter panel is open:

```
┌─────────────────────────────────────────┐
│  🔍 Search tasks…                    ✕  │  ← only for views with search
├─────────────────────────────────────────┤
│  @abends  @telefon  @unterwegs  @kurz   │  ← context chips (all views)
└─────────────────────────────────────────┘
```

- **Search input** — text field with magnifying glass icon (left) and clear button (right, visible only when non-empty)
- **Context chips** — unchanged from existing design, shown below the search input
- On views without search (Upcoming, Quick Wins, Recently Changed), only the context chips appear

### State Management

- Search query is **per-view** — switching from All Open to What I Finished and back preserves each view's query
- Context filters remain per-view as before
- Both are client-side state only — not persisted, not in the URL
- Clearing the search input (via ✕ or manual backspace) immediately restores the unfiltered view

### Filter Button Indicator

The filter toggle button in the header toolbar glows teal when _any_ narrowing is active:

- Active context filter → teal
- Active search query → teal
- Both active → teal
- Neither active → default opacity

This gives the user a single glance signal: "my list is narrowed" — without needing to distinguish which dimension is active.

### Empty State

When search and/or filter produce zero results, the empty state message changes from the view's default ("No open tasks.") to **"No matches."** — signaling that tasks exist but are hidden by the active narrowing, not that the list is truly empty.

### Consequences

- Good, because one toggle controls all narrowing — no cognitive overhead deciding which control to use
- Good, because search input is immediately visible when the panel opens — no extra tap
- Good, because per-view state means search in All Open does not interfere with What I Finished
- Good, because AND composition is predictable — same logic users already know from multi-chip filtering
- Good, because "No matches." prevents the false impression of an empty task list
- Bad, because search requires opening the filter panel — not visible by default, relies on the user knowing about the filter button
- Bad, because no search history or recent queries — every panel open starts with a blank input

## Pros and Cons of the Options

### Separate search icon button with its own panel

- Good, because search gets its own visual identity — magnifying glass icon is universally understood
- Bad, because a third button in the header toolbar (alongside filter and edit) increases visual complexity
- Bad, because two separate panels can be open simultaneously — confusing state

### Always-visible search bar above the task list

- Good, because highest discoverability — always present, no toggle needed
- Good, because best for frequent searchers
- Bad, because consumes vertical space permanently — costly on mobile
- Bad, because visually dominant on views where search is unnecessary (Upcoming, Quick Wins)

### Search input inside the existing filter panel

- Good, because reuses the learned filter toggle interaction
- Good, because groups all narrowing in one place
- Good, because no new header buttons needed
- Neutral, because requires opening the panel to access search — one tap overhead
- Bad, because first-time discoverability is lower than an always-visible bar

### Search as a separate view

- Good, because maximum space for results and input
- Bad, because context switch away from the current view — the user loses their scroll position
- Bad, because a 6th view dot in the header adds complexity to an already full navigation
- Bad, because the user must decide upfront whether to browse or search — a false dichotomy

## More Information

The search input and context chips are orthogonal but complementary filtering tools.
Contexts answer "where/when can I act?" while search answers "which specific task am I looking for?".
The panel groups them because the user's intent is the same: _narrow the list_.

Future extensions that fit naturally into this model:

- **Category filter chips** — a second row of chips for category filtering, below context chips
- **Search highlighting** — matching characters highlighted in the task summary
- **Search scope toggle** — switch between searching summary only vs. summary + notes

See **Client-Side Fuzzy Search** ADR for the matching algorithm and library choice.
