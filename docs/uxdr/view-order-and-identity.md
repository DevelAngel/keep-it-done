---
status: accepted
date: 2026-05-02
supersedes: 2026-05-01 (reorder: Quick Wins moved to position 2)
---

# View Order, Landing View, and View Identity

## Context and Problem Statement

The app currently has four views (My Day, What I Finished, Quick Wins, Recent Changes) navigated by horizontal swipe. With the addition of "Upcoming" (UXDR: Upcoming View), the total grows to five. Several questions arise simultaneously:

1. Which view should greet the user on app open (landing view)?
2. In what order should the views be arranged for swipe navigation?
3. Does "My Day" still accurately name what it shows, given that Upcoming now owns the temporal "today" question?

These decisions are coupled: the landing view determines position 1, and the rename depends on how the views relate to each other conceptually.

## Decision Drivers

- ADHD-friendly: the most time-critical information must require the least effort to reach (zero swipes)
- Daily rhythm: the swipe sequence should mirror a natural usage flow (urgency check → quick wins → planning → reflection → audit)
- Discoverability: five dots remain legible and tappable on mobile (≥44px tap targets at 375px viewport)
- Naming clarity: each view name should answer "what will I see?" without explanation
- Instant color recognition (ADHD): each view must be identifiable by a single color name — "the pink one", "the blue one", "the green one". This requires mono-hue gradients (no cross-hue blending) and maximum hue separation between views, especially adjacent ones

## Considered Options (Order)

- Upcoming first, Quick Wins second (urgency → dopamine-led) — **chosen**
- Upcoming first, open-tasks second (urgency → planning-led)
- Open-tasks first, Upcoming second (planning-led, status quo spirit)
- Chronological: Upcoming → open-tasks → Quick Wins → What I Finished → Recent Changes

## Decision Outcome

Chosen option: "Upcoming first, Quick Wins second" (urgency → dopamine-led). After checking what's due, the natural ADHD impulse is "what can I knock out quickly?" — the dopamine kick from fast completions builds momentum *before* the potentially overwhelming full overview. Broad planning (All Open) is two swipes away, which is acceptable because it's not time-critical.

### Final View Order

| # | View | Color | One-word ID | Mental model |
|---|---|---|---|---|
| 1 | **Upcoming** | Rose (`from-rose-500 to-rose-700`) | "the pink one" | "What's due or overdue?" |
| 2 | **Quick Wins** | Amber (`from-amber-500 to-amber-700`) | "the orange one" | "What can I quickly knock out?" |
| 3 | **All Open** | Cyan (`from-cyan-500 to-cyan-700`) | "the blue one" | "Everything on my plate" |
| 4 | **What I Finished** | Emerald (`from-emerald-500 to-emerald-700`) | "the green one" | "What did I accomplish?" |
| 5 | **Recent Changes** | Sky (`from-sky-500 to-sky-700`) | "the light blue one" | "What changed lately?" |

### Landing View: Upcoming

The landing view (position 1, shown on app open) is Upcoming. Rationale:

- Overdue tasks and today-deadlines demand *immediate* awareness — they should not require navigation
- Users who have no dated tasks see a brief empty state with a backlog counter (see UXDR: Upcoming View) that naturally points them to swipe right
- The daily rhythm begins with "what's urgent?", followed by "what's quick?" for an early dopamine hit, before broadening to "what's everything?"

### Rename: "My Day" → "All Open"

The former "My Day" showed all open tasks grouped by category — a complete, non-temporal overview. With Upcoming now covering the time-sensitive "today" question, "My Day" is misleading: the view is not scoped to a single day.

**Chosen name: "All Open"**

| Evaluated | Verdict | Reason |
|---|---|---|
| My Day | Rejected | Implies temporal scoping that no longer exists |
| All Open | **Chosen** | Accurate (all open tasks), concise (7 chars), no ambiguity about completion state |
| Open Tasks | Runner-up | Clear but slightly redundant with the subtitle |
| Overview | Rejected | Too generic — could describe any view |
| Everything | Rejected | Implies completed tasks are included |
| Backlog | Rejected | Agile jargon; may confuse non-technical family members |

**Subtitle update:** "Open tasks · ↓ category · oldest first" (unchanged in substance, "My Day" was never in the subtitle).

**Empty message update:** "Nothing left for today." → "No open tasks." (removes temporal implication).

### Color Sequence and ADHD Mono-Hue Principle

**Principle:** each view's gradient stays within a single hue family. Cross-hue gradients (e.g. `rose → teal`) blur identity — ADHD users process "which view am I in?" subconsciously via color, and ambiguous gradients delay that recognition. A mono-hue gradient lets the brain match a single color name to a single view without analysis.

The five-view palette reads left to right:

```
Rose → Amber → Cyan → Emerald → Sky
```

Each pair of adjacent views uses a different hue family:
- Rose → Amber: warm pink to warm orange (both warm, but distinct hues — ~45° separation on the color wheel; "the pink one" vs. "the orange one")
- Amber → Cyan: warm orange to cool blue (maximum temperature jump)
- Cyan → Emerald: blue to green (distinct families, no shared midpoint)
- Emerald → Sky: green to light blue (cool-to-cool, but clearly distinct hues)

**Change from existing palette:** the former cross-hue gradients `from-cyan-600 to-teal-700` (My Day) and `from-teal-600 to-emerald-700` (What I Finished) shared a teal midpoint, making swipe transitions between them ambiguous. Both are replaced by mono-hue gradients. The corresponding dot-active, checkbox, spinner, and priority-A-border colors update to match the new single-hue base.

**Hue collision check — Cyan vs. Sky:** positions 3 and 5 are both in the blue family. At `cyan-500` (#06b6d4) vs. `sky-500` (#0ea5e9) the hues are close. This is acceptable because the views are never adjacent (2 positions apart, with Emerald between them) and the one-word IDs differ ("the blue one" vs. "the light blue one"). If user testing reveals confusion, Cyan can shift to Indigo (`from-indigo-500 to-indigo-700`, "the purple one") as a fallback.

### Swipe Navigation Impact

**Dot indicators:** five dots at ~8px diameter + 8px gap = ~72px total width. Comfortably fits at 375px with centered alignment. Tap targets remain ≥44px (CSS padding around each dot).

**Progressive affordance:** thresholds (0–10, 11–50, 50+) remain unchanged. The swipe-count counter does not reset — users who already reached "expert" phase with 4 views retain their state.

**Keyboard shortcuts:** number keys 1–5 for direct access (was 1–4). Accessibility labels update: "View 1 of 5: Upcoming", etc.

**Edge behavior:** left arrow hidden on Upcoming (position 1), right arrow hidden on Recent Changes (position 5). No wrapping — linear sequence.

### Consequences

- Good, because the most urgent information requires zero interaction to see
- Good, because "All Open" is self-explanatory and honest about what the view contains
- Good, because the daily rhythm (urgency → quick wins → planning → reflection → audit) maps naturally to left-to-right swipe
- Good, because Quick Wins at position 2 provides an early dopamine hit — ADHD users build momentum from fast completions before facing the full task list
- Good, because five views remain within the comfortable range for linear swipe navigation
- Good, because mono-hue gradients give each view a single-word color identity — instant recognition for ADHD users
- Good, because eliminating the shared teal midpoint removes swipe-transition ambiguity between positions 2 and 3
- Neutral, because users familiar with "My Day" as position 1 must relearn — mitigated by the view name being always visible in the header
- Bad, because users without dated tasks see an empty landing view on every open — mitigated by the backlog counter and single-swipe access to Quick Wins
- Bad, because Cyan (position 3) and Sky (position 5) are both blue-family — mitigated by 2 positions of distance and fallback to Indigo if testing reveals confusion
- Bad, because Rose (position 1) and Amber (position 2) are both warm-toned — mitigated by ~45° hue separation (pink vs. orange) and distinct one-word IDs

## Pros and Cons of the Options

### Upcoming first, Quick Wins second (urgency → dopamine-led) — chosen

- Good, because deadlines and overdue items are the most time-sensitive information in the system
- Good, because Quick Wins at position 2 serves the natural follow-up impulse: "what's urgent?" → "what can I knock out fast?"
- Good, because fast completions generate dopamine before the potentially overwhelming full overview — builds momentum for ADHD users
- Good, because the backlog counter on an empty Upcoming naturally guides users rightward to Quick Wins
- Good, because Recent Changes (AI audit) is correctly deprioritized at position 5 — rarely needed by humans
- Bad, because frequent empty-state for users who rarely use dates
- Bad, because All Open (full overview) is two swipes away instead of one

### Upcoming first, open-tasks second (urgency → planning-led)

- Good, because the full overview is immediately accessible after the urgency check
- Bad, because Quick Wins is buried at position 4 — the dopamine hit comes too late in the sequence
- Bad, because the jump from urgency to full overview can feel overwhelming for ADHD users

### Open-tasks first (planning-led)

- Good, because preserves the current landing behavior — no user relearning
- Good, because always populated (any open task appears)
- Bad, because urgency is one swipe away — for ADHD users, "one swipe" can mean "never seen"
- Bad, because the most time-critical information is not in the most accessible position

### Chronological (time-based sequence)

Upcoming → All Open → Quick Wins → What I Finished → Recent Changes

- Good, because a temporal logic (future → present → past) is conceptually elegant
- Bad, because "What I Finished" (past) being buried at position 4 conflicts with its role as evening reflection — users want it adjacent to the planning views
- Bad, because Quick Wins is not time-based at all; forcing it into a temporal sequence feels arbitrary

## More Information

**Migration path:** the `View` enum order changes from `[Upcoming, AllOpen, WhatIFinished, QuickWins, RecentlyChanged]` to `[Upcoming, QuickWins, AllOpen, WhatIFinished, RecentlyChanged]`. The `current_view` default remains `View::Upcoming`. Stored user preferences (if any) referencing the old enum positions must be handled.

**Relationship to UXDR: View Switching:** the swipe mechanism, progressive affordance, and gesture detection remain unchanged. Only the number of dots and the enum order change. The view-switching UXDR's note about "degradation beyond ~6 views" is not triggered at 5.

**Relationship to UXDR: Upcoming View:** this document defines *where* Upcoming sits; the Upcoming UXDR defines *what* it shows.
