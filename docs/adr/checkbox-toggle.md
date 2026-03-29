---
status: accepted
date: 2026-03-29
---

# Checkbox Toggle for Task Completion

## Context and Problem Statement

Each task in the list view has a checkbox. The checkbox must have one clear, unambiguous function. The decision affects user muscle memory, click count for the most frequent action, visual feedback, and state management complexity.

## Decision Drivers

- Completion is the most frequent action — it must be the simplest
- Spatial separation between state change and detail view
- Mobile-first: precise touch targets, no accidental interactions
- Follow established conventions that users already know

## Considered Options

- Checkbox as direct toggle (todo ↔ done)
- Checkbox for selection, separate button for completion
- Swipe gesture for completion
- Long-press for completion
- Entire row toggles status

## Decision Outcome

Chosen option: "Checkbox as direct toggle", because it matches the universal convention from Todoist, Apple Reminders, Microsoft To Do, and every other task app. No user has to learn it. It makes the most frequent action a single click. And it provides clean spatial separation: checkbox left for status, row right for details.

The toggle is binary: unchecked (○) = `ToDo`, checked (●) = `Done`. Clicking the row (outside the checkbox) expands or collapses the detail view. `stop_propagation()` prevents checkbox clicks from triggering the row handler.

The task model has exactly two states (`ToDo`/`Done`). There is no in-progress state. The checkbox maps cleanly onto this binary model.

### Consequences

- Good, because users know how checkboxes work — zero learning curve
- Good, because one click for the most frequent action
- Good, because `<input type="checkbox">` is semantically correct — screen readers understand it natively
- Good, because spatial separation (checkbox left, row right) reduces accidental interactions on mobile
- Good, because fill animation + strikethrough gives immediate, satisfying feedback
- Bad, because one accidental click marks the task done — no undo button (mitigation: click again to revert; target is small enough that accidents are rare)
- Bad, because checkboxes serve only completion, not multi-select for batch operations (mitigation: batch operations are rare at family scale; an "Edit Mode" can be added later if needed)

## Pros and Cons of the Options

### Checkbox as direct toggle

- Good, because universal convention — no explanation needed
- Good, because large enough touch target (20×20 px + padding → effective ~28×28 px)
- Neutral, because "no undo" feels permanent, but reverting is a second click on the same target

### Checkbox for selection, separate button for completion

- Neutral, because enables future batch operations
- Bad, because completion becomes a secondary action — requires opening details first
- Bad, because increases UI complexity for a rare use case

### Swipe gesture for completion

- Bad, because not discoverable — users must be taught
- Bad, because collision with scroll gestures on mobile
- Bad, because accidental swipes during scrolling are more likely than accidental checkbox clicks

### Long-press for completion

- Bad, because slower — requires waiting for hold duration
- Bad, because long-press is a context menu pattern, not a primary action pattern

### Entire row toggles status

- Bad, because collides with expanding detail view — both actions need the same target
- Bad, because accidental completions would be more frequent (larger hit area)
- Bad, because detail view becomes inaccessible without a separate affordance

## More Information

**Visual states:**

The checkbox uses `rounded-full` (soft, matches the app's gradient theme) and a `checked:bg-gradient-to-br checked:from-cyan-500 checked:to-teal-600` fill. The summary text gets `line-through opacity-50` when checked.

**Optimistic UI with revert on failure:**

The UI signal (`set_checked`) is updated immediately before the server call dispatches. If `complete_task(id, checked)` fails, the signal is reverted (`set_checked.set(!checked)`). While the call is in flight, the checkbox is disabled (`prop:disabled=pending`) to prevent double-clicks.

**Future: undo toast.** If accidental completions become a problem, a toast ("Task completed. [Undo]", visible 3 s) can be added without changing the checkbox semantics.
