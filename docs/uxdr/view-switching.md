---
status: proposed
date: 2026-03-29
---

# Swipe-Based View Switching with Progressive Affordances

## Context and Problem Statement

The task management system exposes multiple filtered views optimized for different user contexts (morning planning, time-window matching, evening reflection, audit review). Users switch views 5–15 times per day, typically on a phone, often in brief moments with limited attention. How should view switching be designed to be discoverable on first use, efficient with daily repetition, and non-intrusive once learned?

## Decision Drivers

- Switching must happen in-place — no screen transitions, no loss of scroll position
- Discoverable without tutorials for family members with varying technical proficiency
- Efficient for expert use (becomes muscle memory)
- Mobile-first: 375–428px primary viewport, no permanent UI chrome beyond what the header already shows

## Considered Options

- Swipe gesture with progressive affordance disclosure
- Tab bar (iOS-style)
- Dropdown menu only
- Filter chips (horizontal scrolling row)
- Card stack with swipe-to-dismiss
- Auto-switching based on time of day

## Decision Outcome

Chosen option: "Swipe gesture with progressive affordance disclosure", because it requires zero permanent UI chrome, becomes instant muscle memory with daily use, and the progressive disclosure of arrow affordances solves discoverability without requiring a tutorial.

**Interaction model:**

- Swipe left/right anywhere in the header area to cycle through views
- Arrow buttons always visible as tap targets and directional cues
- Page indicator dots show current position, individually tappable for direct access

**Progressive affordance levels** (automatic, based on swipe count):

| Phase       | Swipes | Arrow opacity            | Purpose                           |
| ----------- | ------ | ------------------------ | --------------------------------- |
| Learning    | 0–10   | 80%                      | Teach the interaction             |
| Habituation | 11–50  | 40% (80% on interaction) | Clean up while maintaining safety |
| Expert      | 50+    | 20% (80% on interaction) | Minimal chrome, maximum content   |

**Semantic color coding per view:**

- My Day — cyan/teal (active, forward-looking)
- What I Finished — teal/emerald (completion, positive)
- Quick Wins — amber (opportunistic, brief)
- Recent Changes — sky (analytical, audit)

Header title and arrow colors change with the active view.

> [!TIP]
> Color palette aligned with the app's existing cyan/teal accent system.
> Original proposal used purple/green/orange/blue;
> revised to stay within the established palette.

### Consequences

- Good, because zero permanent screen space cost — no tab bar eating 10–12% of viewport height
- Good, because progressive disclosure solves the discovery/efficiency tension without two separate UI modes
- Good, because sequential swipe matches the natural flow: morning planning → during-day → evening reflection
- Bad, because horizontal swipe gesture conflicts with horizontally scrollable content if added later (mitigated by restricting swipe detection to the header area)
- Bad, because linear swipe degrades beyond ~6 views — spatial sense of position is lost
- Bad, because color coding does not help color-blind users (mitigated: view name and page indicator position provide redundant cues)

## Pros and Cons of the Options

### Swipe gesture with progressive affordance disclosure

- Good, because no permanent chrome — maximum content area
- Good, because gesture becomes instant with repetition
- Good, because progressive disclosure avoids both "can't find it" (early) and "visual noise" (later)
- Bad, because initial discoverability depends on arrows being noticed

### Tab bar (iOS-style)

- Good, because instant random access, universally understood
- Bad, because permanent space cost (~10–12% of viewport height)
- Bad, because 4-tab maximum before "More" overflow — defeats simplicity for 5+ views

### Dropdown menu only

- Good, because explicit, always discoverable
- Bad, because 3 taps per switch (open, select, close) — painful at 10+ switches/day
- Bad, because incompatible with the time-window-matching use case (30 seconds to find a task)

### Filter chips

- Good, because maximum flexibility (arbitrary filter combinations)
- Bad, because requires understanding filter composition — expert mental model
- Bad, because horizontal chip row + vertical task list creates scroll direction conflicts

### Card stack with swipe-to-dismiss

- Good, because spatial metaphor is clear
- Bad, because vertical swipe conflicts with list scrolling
- Bad, because "dismiss" is semantically wrong — views are persistent modes, not items to discard

### Auto-switching based on time of day

- Neutral, because reduces switching friction for regular schedules
- Bad, because removes user agency — night-shift workers, irregular schedules break the assumption
- Bad, because unexpected view changes are disorienting

## More Information

**Gesture detection thresholds:** Horizontal movement >10px AND horizontal/vertical ratio >2:1 within first 100ms of touch. Prevents vertical scrolls from triggering view switches.

**Animation:** `transform: translateX()` (GPU-accelerated). Only current and adjacent views rendered.

**Target performance:** ≥60 FPS on iPhone 11, ≥50 FPS on Moto G7.

**Accessibility:** Arrow buttons labeled "Previous view: [name]" / "Next view: [name]". Title is `aria-live` region. Keyboard: arrow keys + number shortcuts.

**Review triggers:** After first user testing round (n ≥ 8), after two weeks of family usage (n ≥ 3 families), after accessibility audit. Arrow visibility thresholds and transition timing are the most likely adjustments.

This decision is an informed hypothesis. It is not final until validated through real use.
