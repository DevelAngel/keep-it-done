---
status: accepted
date: 2026-03-29
---

# Timeline-Style Task Detail Expansion

## Context and Problem Statement

The task list is a vertical list of rows. When a user taps a task, additional details must become visible without losing context, without a modal blocking the screen, and without requiring navigation. How should the detail view be designed so that it feels native to the list, communicates the task's properties clearly, and creates a distinctive visual identity?

## Decision Drivers

- In-place expansion — no navigation, no modal
- Mobile-first: works well at 375–428px width
- Visual hierarchy that reflects semantic importance (priority > created)
- Distinctive aesthetic that is recognizable, not generic

## Considered Options

- Timeline with vertical connecting line and color-coded markers
- Icon-enhanced card
- Minimal grid
- Card with shadow elevation
- Flat list (no visual enhancement)

## Decision Outcome

Chosen option: "Timeline with vertical connecting line and color-coded markers", because it communicates that a task is a temporal object (from creation to due date) rather than a static record, creates a clear top-to-bottom reading order that mirrors semantic priority, and gives the app a recognizable visual identity.

**Visual design:**

- Vertical gradient line as spine (`from-cyan-500 via-teal-500 to-cyan-500`)
- Circular markers with 4px `border-slate-900` rings — markers "float" above the line
- Slate-gradient background (`from-slate-900 via-slate-800 to-slate-900`) separates the expanded task from the list
- Consistent spacing: `space-y-4` (16px between nodes)

**Information order** (optional fields shown only when set):

1. Priority (Red/Amber/Sky) — A=Critical, B=Important, C=Routine
2. Due Date (Teal)
3. Start Date (Sky)
4. Time Estimate (Amber, rounded square marker)
5. Context (Teal)
6. Notes (Slate)
7. Created (Cyan) — always present, always last

Reads from actionable/urgent at top to archival/contextual at bottom.

**Color semantics:**

| Color                | Field                     | Meaning                      |
| -------------------- | ------------------------- | ---------------------------- |
| Red `bg-red-500`     | Priority A                | Critical, immediate          |
| Amber `bg-amber-500` | Priority B, Time Estimate | Important, resource cost     |
| Sky `bg-sky-400/700` | Priority C, Start Date    | Routine, informational       |
| Teal `bg-teal-700`   | Due Date, Context         | Temporal boundary, structure |
| Cyan `bg-cyan-700`   | Created                   | Origin, baseline             |
| Slate `bg-slate-600` | Notes                     | Supplementary                |

### Consequences

- Good, because the sequential structure creates spatial memory — users remember "priority was the first point on the line"
- Good, because color coding maps directly to semantic meaning, not decoration
- Good, because the timeline is a visual echo of the header gradient — same identity at a smaller scale
- Good, because screen readers traverse the sequential structure naturally
- Good, because CSS-based transitions run at 60 FPS on tested devices
- Bad, because expanded tasks are 200–300px tall — on small screens, only 2–3 expanded tasks are visible simultaneously
- Bad, because the timeline metaphor is not self-explanatory on first use — the line is not a progress bar
- Bad, because staggered marker animation (planned) adds CSS complexity and browser-specific testing overhead

## Pros and Cons of the Options

### Timeline with vertical connecting line

- Good, because communicates temporal dimension — tasks are journeys, not data rows
- Good, because clear top-to-bottom hierarchy reflects semantic importance
- Good, because visually distinctive — recognizable across the app
- Bad, because consumes more vertical space than alternatives
- Bad, because requires explaining the metaphor to new users

### Icon-enhanced card

- Good, because icons provide fast scanning anchors
- Bad, because icons without structure are decorative, not hierarchical — all properties appear equally important

### Minimal grid

- Good, because maximum information density, fastest scan time (~5 s vs ~8 s for timeline)
- Bad, because no hierarchy — priority and notes appear equally prominent
- Bad, because no visual identity

### Card with shadow elevation

- Good, because clear visual separation
- Bad, because shadows are less effective on touch screens than on desktop
- Bad, because nested card feels heavy on small screens

### Flat list (no visual enhancement)

- Good, because simple, performant, zero implementation cost
- Bad, because no visual identity — interchangeable with any todo app

## More Information

**Expansion model (Leptos):**

```rust
let is_expanded = move || expanded_task_id.get() == Some(task_id);

<Show when=is_expanded>
    <TaskDetails task=id/>
</Show>
```

One task expanded at a time. Clicking the same row collapses. Clicking a different row switches.

**Animation:** Currently `transition-all duration-300` on the container. Staggered marker delays (50ms each, top to bottom) are planned but not yet implemented — all markers appear simultaneously.

**HTML structure:**

```html
<div class="relative pl-8 space-y-4">
  <div
    class="
      absolute left-3 top-0 bottom-0 w-0.5
      bg-gradient-to-b from-cyan-500 via-teal-500 to-cyan-500
    "
  >
  </div>
  <div class="relative">
    <div
      class="
        absolute -left-8 mt-0.5 w-6 h-6
        rounded-full bg-cyan-700 border-4 border-slate-900 shadow
      "
    >
    </div>
    <div class="text-xs font-semibold uppercase tracking-wide text-cyan-400">
      Label
    </div>
    <div class="text-sm text-slate-200">Value</div>
  </div>
</div>
```

**Markers** are currently implemented inline in `TaskDetails`, not as a reusable `TimelineMarker` component. Extraction is a planned refactor.

**Theming:** If a light mode is added, Tailwind v4 `@theme` variables centralize color tokens without changing the HTML structure.
