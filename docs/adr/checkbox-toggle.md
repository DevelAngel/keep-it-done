# ADR: Checkbox Toggle for Task Completion

## Status

Accepted

## Context

In the task list view, each task has a checkbox. This checkbox must have a clear function: Either it marks the task as completed, or it serves another purpose (e.g., "selection for batch operation").

We must decide:
1. What happens when you click the checkbox?
2. Is this the primary interaction for task completion, or are there other ways?
3. How do checkbox and task row click interact with each other?

This decision influences:
- User muscle memory (from other task apps)
- Number of clicks for frequent actions
- Visual feedback patterns
- State management complexity

## Decision

The checkbox functions as a **direct toggle for task completion**: Clicking the checkbox switches the task status between `todo` and `done`.

**Concretely**:
- Checkbox unchecked (○) → Task status: `todo`
- Checkbox checked (●) → Task status: `done`
- Click on checkbox: Toggle between both states
- Click on task row (except checkbox): Expands/collapses details

This function is **not reversible** via an undo button, but through another click on the checkbox.

## Rationale

### Convention Over Innovation

Almost every task management app uses checkboxes for completion:
- Todoist
- Microsoft To Do
- Apple Reminders
- Google Tasks
- Any.do
- TickTick

You have used these apps. Your children have used these apps. Your brain associates: "Checkbox next to task = mark as done."

If we break this convention – if we use checkboxes for "selection" or "favorite" – we fight against years of habit. That is not impossible. But it must have a very good reason.

Do we have such a reason? No.

### Efficiency for Most Frequent Action

What action do you perform most frequently in a task app?

**Not** "edit task" (occasionally).  
**Not** "delete task" (rarely).  
**Not** "move task" (sometimes).

**But**: "Mark task as completed" (multiple times daily).

This action must be the simplest:
- One click, no confirmation
- Large touch target (checkbox is 20×20px with 4px padding → 28×28px effective)
- Immediate visual feedback (checkbox fills, summary gets strikethrough)

If completion were a secondary workflow ("click on task → details open → button 'mark as done'"), this would complicate the most frequent action.

### Spatial Separation

Checkbox and task row have different functions:
- **Checkbox**: State change (todo ↔ done)
- **Task row**: View change (collapsed ↔ expanded)

This separation is spatially clear:
```
┌────────────────────────────────────┐
│ [○] Research cabinet options       │  ← Checkbox left, row right
└────────────────────────────────────┘
```

When you quickly go through tasks and check them off, you click left. When you want to see details, you click right. Two targets, two spatial areas.

Alternative: "Click on row toggles status, tap-and-hold opens details." This would be confusing. Tap-and-hold is not a discoverable interaction (you must be taught). And it is slower (you must wait).

### Visual Feedback Pattern

When you click a checkbox, you expect immediate visual feedback:
1. Checkbox animation (empty → filled, 150ms)
2. Summary text gets strikethrough
3. Text opacity reduces to 50%

This feedback pattern is universal. You know it from:
- Email clients (mark message as read)
- Shopping lists (check off item)
- Forms (select checkbox)

We use this established pattern. It is not creative. But it is immediately understandable.

### Mobile-First Consideration

On a small screen (iPhone SE: 375px wide), precision matters. You do not want to accidentally mark a task as done when you actually wanted to see details.

The spatial separation – checkbox left (for status change), row right (for detail view) – reduces accidental clicks.

If the entire row toggled status, accidental completions would be more frequent. You tap to see details, but your finger lands 5mm too far left → task is suddenly done.

## Implementation Details

### State Management

```rust
let (checked, set_checked) = signal(false);

// Checkbox handler
on:change=move |_| {
    set_checked.update(|c| *c = !*c);
    // TODO: Persist status change to backend
}

// Prevent event bubbling to row click handler
on:click=|e| e.stop_propagation()
```

The `stop_propagation()` is critical. Without it, a checkbox click would also trigger the row click → task would simultaneously open/close and mark as done.

### Visual States

```rust
// Checkbox CSS
class="
    w-5 h-5
    rounded-full
    border-2 border-slate-600
    cursor-pointer
    appearance-none
    transition-all
    checked:bg-gradient-to-br
    checked:from-cyan-500
    checked:to-teal-600
    checked:border-cyan-500
"

// Summary text CSS
class=move || if checked.get() {
    "text-gray-900 line-through opacity-50"
} else {
    "text-gray-900"
}
```

**Why `rounded-full`?**
Standard checkboxes are square. Round checkboxes are unusual, but not foreign (see Apple Reminders). They are softer, friendlier, fit better with the app's gradient theme.

**Why gradient when checked?**
The gradient (cyan → teal) matches the app's color theme. It is a visual echo – the app says: "Well done, this task now belongs to the completed world."

### Backend Persistence

```rust
#[server(endpoint = "complete_task")]
pub async fn complete_task(id: Uuid, completed: bool) -> Result<(), ServerFnError> {
    let cache = use_context::<SharedTaskCache>()...;
    let mut cache = cache.write().await;
    if completed {
        cache.get_mut(&id)?.mark_done();
    } else {
        cache.get_mut(&id)?.mark_todo();
    }
    Ok(())
    // dirty flag is set automatically via TaskMutGuard::drop()
    // background flush writes to disk within ~60 s
}
```

The checkbox executes this server function optimistically: the UI signal (`set_checked`) is updated immediately before dispatching. If the server call fails, the signal is reverted (`set_checked.set(!checked)`) and the error is logged. While the call is in flight, the checkbox is disabled (`prop:disabled=pending`) to prevent double-clicks.

## Consequences

### Positive

**Intuitive**: Users do not need to learn how checkboxes work. They already know.

**Efficient**: The most frequent action (checking off task) is the simplest (one click).

**Accessible**: Checkboxes are semantically correct (`<input type="checkbox">`), screen readers understand them natively.

**Mobile-Optimized**: Clear spatial separation reduces accidental interactions.

**Visually Satisfying**: The filling of the checkbox + strikethrough is satisfying. It gives immediate feedback.

### Negative

**Permanent-Feeling**: One click marks the task as done. If you accidentally click, you must click again to revert. No undo button.

**Mitigation**: The checkbox is small enough (20px) that accidental clicks are rare. And: Reverting is simple (same checkbox, click again).

**No Batch Operations**: Checkboxes serve only completion, not multi-select for batch-delete or batch-move.

**Mitigation**: For a family app, batch operations are rarer. If they become necessary later, we can add a separate "Edit Mode" (like iOS Reminders).

**Two states only**: The task model intentionally has only `ToDo` and `Done` — no `in-progress`. The checkbox maps cleanly onto this binary model. There is no hidden third state that could leave the checkbox in an undefined position.

## Alternatives Considered

### Alternative 1: Checkbox for Selection, Button for Completion

Checkbox marks tasks for batch operations. A separate button (e.g., in expanded details) marks as done.

**Rejected** because:
- Completion is the most frequent action, should not be secondary
- Batch operations are rare in family apps (no need for "delete all Kitchen tasks")
- Additional button increases UI complexity

### Alternative 2: Swipe Gesture for Completion

Swipe right over a task → marks as done.

**Rejected** because:
- Swipe gestures are not discoverable (users must learn them)
- Collision with possible other swipe actions (swipe-left for delete?)
- Accidental swipes while scrolling are more frequent than accidental checkbox clicks

### Alternative 3: Long-Press for Completion

Hold a task long → menu appears with "mark as done."

**Rejected** because:
- Slower than direct click (you must wait)
- Hides the most frequent action behind a menu
- Long-press is primarily for context menus (with multiple options), not for single actions

### Alternative 4: Entire Row Toggles Status

Click on the entire task row toggles between `todo` and `done`. No separate detail view.

**Rejected** because:
- Detail view is important (created timestamp, priority, notes)
- Collision between "change status" and "see details" would be inevitable
- Accidental status changes would be more frequent

## Future Considerations

### Multi-Status Support

The task model has exactly two states: `ToDo` ↔ `Done`. An `in-progress` state was explicitly decided against to keep the model simple (see `task-card.md`). The checkbox toggle maps cleanly onto this binary model and requires no future extension for additional states.

### Undo Functionality

If accidental completions become a problem:
- Toast notification after completion: "Task completed. [Undo]"
- Undo button in notification (visible for 3 seconds)
- Click on undo: Reverts status

This would be a later addition, not core functionality.

### Recurring Tasks

When a task is marked as "daily recurring":
- Checkbox click marks today's instance as done
- A new instance automatically appears for tomorrow

The checkbox semantics remain the same: "Mark this instance as done."

## Conclusion

The checkbox as direct toggle for task completion is the right decision for this app because:
1. It follows universal conventions
2. It makes the most frequent action the simplest
3. It uses spatial separation for functional clarity
4. It is optimized for mobile

This is not an innovative decision. It is a deliberately conventional decision. And that is good.

Innovation should take place where it provides added value (see: Timeline detail view). Completion mechanisms are not the place for innovation. They are the place for reliability.

You know how checkboxes work. We use this knowledge, instead of fighting against it.
