---
status: proposed
date: 2026-04-01
---

# Time Estimate Input as a Fixed Chip Group

## Context and Problem Statement

Tasks can carry a time estimate to support the Quick Wins view, which surfaces short tasks that fit into an available window. The estimate needs to be set quickly on a phone, displayed clearly in the task detail view, and sortable so Quick Wins can rank tasks by effort.

The previous data model used `TimeEstimation { Guess(String), Precise(Duration) }` — arbitrary free text or any duration. This is unsortable (guesses) and produces unbounded UI surface (free text entry, no natural chip set).

How should a user set a time estimate on a mobile device in a way that is fast, sortable, and maps cleanly to a fixed data model?

## Decision Drivers

- Must be operable with one thumb, no keyboard
- Must produce a sortable value for Quick Wins ranking
- Small fixed option set — all choices visible at once without scrolling
- Matches natural language people use for family tasks ("half a day", "two days")
- Simplifies the data model: no arbitrary strings, no `Duration` math in UI code

## Considered Options

- Chip group (segmented pill row) with fixed named variants
- Stepper (`−` / `+` buttons) over a fixed sequence
- Free-text input (keep `Guess(String)`)
- Native `<input type="time">` or duration field
- Dropdown / select element

## Decision Outcome

Chosen option: **chip group with fixed named variants**, because all options fit on one screen, selection is a single tap, and the fixed set maps directly to a sortable enum with no validation logic needed.

**Fixed variants (in sort order):**

| Chip label | Enum variant | Duration |
|---|---|---|
| 15 min | `Min15` | 15 minutes |
| 30 min | `Min30` | 30 minutes |
| 45 min | `Min45` | 45 minutes |
| 1 h | `Hours1` | 1 hour |
| 2 h | `Hours2` | 2 hours |
| ½ day | `HalfDay` | 4 hours |
| 1 d | `Day1` | 1 day |
| 2 d | `Day2` | 2 days |

**Interaction model:**

- Chips displayed as a wrapping row of tappable pills in the edit view
- Currently selected chip is highlighted (amber, matching the estimate accent color)
- Tapping the active chip deselects it (clears the estimate)
- No keyboard, no confirmation step

**Quick Wins sort:** tasks are ranked by `TimeEstimate` variant order (ascending). The enum derives `Ord` over declaration order, which matches duration order.

**Free-text guesses (e.g. "a weekend") are intentionally not supported as a new input.** Tasks with qualitative effort descriptions should use the `notes` field.

### Migration from old format

The previous model stored `TimeEstimation::Guess(String)` and `TimeEstimation::Precise(Duration)`. The backward-compat deserializer maps these without data loss:

| Old value | New value | Rationale |
|---|---|---|
| `Precise(d)` | nearest variant by midpoint rounding | preserve effort magnitude |
| `Guess(_)` | `Day2` | a guess implies uncertain large effort — sorts last in Quick Wins |

Midpoint boundaries for `Precise` mapping:

| Range (minutes) | Variant |
|---|---|
| 0 – 22 | `Min15` |
| 23 – 37 | `Min30` |
| 38 – 52 | `Min45` |
| 53 – 90 | `Hours1` |
| 91 – 180 | `Hours2` |
| 181 – 360 | `HalfDay` |
| 361 – 720 | `Day1` |
| 721 + | `Day2` |

`Guess` mapping to `Day2` is intentional: a vague estimate signals a task that is unlikely to fit into a short opportunistic window, so it correctly sorts to the bottom of Quick Wins.

### Consequences

- Good, because single tap to set — no keyboard on mobile
- Good, because all options visible simultaneously, no scroll needed
- Good, because produces a sortable value — Quick Wins ranking works without special-casing
- Good, because data model simplifies: `TimeEstimation`/`TimeEstimationRef` collapse into one flat `Copy` enum
- Good, because no data loss on migration — every old estimate maps to a valid variant
- Bad, because "2 days" is the hard cap — tasks estimated beyond that cannot express a time estimate (acceptable: tasks that take more than 2 days are projects, not Quick Wins candidates)

## Pros and Cons of the Options

### Chip group (chosen)

- Good, because all options visible, no interaction to reveal them
- Good, because large tap targets — no precision required
- Good, because maps 1:1 to enum variants, zero parsing
- Bad, because 8 chips may wrap to two rows on very narrow screens (375px) — acceptable with wrapping layout

### Stepper (`−` / `+`)

- Good, because compact — always one line
- Bad, because requires multiple taps to reach distant values (15 min → 2 days = 7 taps)
- Bad, because current value not immediately obvious without reading the label

### Free-text input (keep `Guess`)

- Good, because expressive — any estimate is possible
- Bad, because opens keyboard on mobile — breaks one-thumb flow
- Bad, because produces unsortable values — Quick Wins cannot rank

### Native time/duration input

- Bad, because no browser-native duration input exists
- Bad, because `<input type="time">` means time-of-day, not duration

### Dropdown / select

- Good, because compact
- Bad, because tap → scroll → tap interaction on mobile — more friction than chips for ≤8 options

## More Information

**Chip layout:** `flex-wrap: wrap`, each chip `min-width: fit-content`, gap `0.5rem`. On 375px with 8 chips at ~52px average width the set fits in two rows of four.

**Accessibility:** each chip is a `<button>` with `aria-pressed` reflecting selection state. Chips are keyboard-navigable with arrow keys within the group.

**Review triggers:** after first family usage round — check whether "2 days" cap is ever hit in practice, and whether any common estimate is missing from the set.
