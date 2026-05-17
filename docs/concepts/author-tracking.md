# Author Tracking Concept

## Abstract

Each task records an audit trail of its editors — who changed it and
when. The trail is structured as a sequence of **turns**: a turn is a
contiguous period where one actor is the most recent editor. Consecutive
edits by the same actor collapse into a single timestamp; a new
timestamp is only appended when a _different_ actor takes over. This
turn-based model exists primarily to support the AI-involvement
indicators in the Recent Changes view.

## Context

The Recent Changes view shows two AI signals per task (see UXDR: Recent
Changes):

- **amber left border** — AI made the most recent change
- **violet left border** — AI was involved, but a human acted last

Both signals require knowing _who acted last_ and _whether any AI actor
ever participated_. A simple "last modified" timestamp cannot answer
these questions; the system needs a per-actor edit history that
faithfully records handoffs between actors.

Actors are identified by name. Any name starting with `ai:` is
treated as an AI actor; all other names are treated as human.

### Actor naming convention

- **Human actors** use a plain name (e.g. `angelos`). The web UI
  derives the name from the authenticated session.
- **AI actors** use the format `ai:<assistant>:<human>` — the AI
  assistant's name followed by the human who initiated the action.
  For example, `ai:navi:angelos` means the AI assistant Navi acted
  on behalf of Angelos.

The on-behalf-of segment preserves accountability: the author trail
shows not only that an AI made a change, but who instructed it to do
so. The `ai:` prefix is the only part the system inspects for
indicator logic — the remaining segments are for human readers.

## Data Model

Each task stores a map from actor name to a list of timestamps. Each
key is an actor name. Each value contains one timestamp per turn that
actor took. The map preserves insertion order; timestamps within each
value are chronologically ascending.

### What is a turn?

A turn begins when an actor becomes the most recent editor and ends
when a different actor edits. The number of timestamps per actor
equals the number of distinct turns that actor took — not the number
of individual edits.

### Example: A, B, A, A → A, B, A

| Step | Actor | Global last | Action                                      |
| ---- | ----- | ----------- | ------------------------------------------- |
| 1    | A     | (none)      | Append t₁ to A                              |
| 2    | B     | A           | Append t₂ to B (different actor → new turn) |
| 3    | A     | B           | Append t₃ to A (different actor → new turn) |
| 4    | A     | A           | Update t₃→t₄ in A (same actor → collapse)   |

Final state: `{A: [t₁, t₄], B: [t₂]}`

Chronological turn sequence: **A, B, A** — three turns, not four edits.

### Why not A, B?

Collapsing A's second turn into the first would lose the fact that A
returned _after_ B's involvement. For AI indicators this is critical:
a returning AI actor must be recognised as the most recent editor, not
hidden behind an earlier human turn.

### Why not B, A?

Dropping A's initial turn would lose the information that A started the
work before B's involvement. The turn sequence is chronological — it
records the order of handoffs, not just the set of participants.

## Why Debounce Was Removed

An earlier version used a 5-minute time window instead of the
turn-based rule: if the same actor's last edit was less than 5 minutes
ago, update the timestamp in place; otherwise append a new one.

This had two problems:

1. **False turn boundaries.** A single actor editing at t=0 and t=6min
   (no other actor in between) produced two timestamps — implying a
   handoff that never happened. The history grew with entries that
   carried no semantic meaning.

2. **Per-actor instead of global.** The debounce checked only the
   actor's own last timestamp, not who made the most recent edit
   globally. This meant the collapsing decision was independent of
   whether an actual handoff occurred.

The replacement checks a single condition: _is the most recent global
edit by the same actor?_ If yes, update the existing timestamp in
place. If no, append a new timestamp. Time gaps are irrelevant — only
actor switches create turn boundaries.

## Deriving AI Indicators

At query time the turn history is reduced to two booleans:

- **ai_involved** — true if any actor name starts with `ai:`
- **ai_last** — true if the actor with the most recent timestamp
  starts with `ai:`

The turn-based model ensures these flags reflect actual handoffs. A
human reviewing and saving after an AI edit flips `ai_last` to false
while `ai_involved` remains true — exactly the semantic difference
between the amber and violet border.

### Reduction from three to two indicators

The design originally considered three levels of AI involvement:

| Level           | Meaning                                                 | Implication                             |
| --------------- | ------------------------------------------------------- | --------------------------------------- |
| **AI-only**     | No human has ever edited this task                      | Urgently review for errors              |
| **AI-last**     | AI made the most recent edit, human was involved before | AI changed something after human review |
| **AI-involved** | AI participated, human acted last                       | Human has reviewed AI's work            |

Three distinct colors on a single visual channel would force ADHD
users to maintain a mental legend ("what was the color for…?") —
exactly the cognitive overhead the project's UX principles prohibit.

The reduction merges AI-only into AI-last: if only AI has ever edited
a task, AI is necessarily the most recent actor, so `ai_last` is true.
The amber border covers both cases. A user scanning the list sees two
states: amber means "AI had the last word — check this", violet means
"AI helped but a human finished it." No legend required beyond that
single axis.

The priority-A accent border, which other views use for A-priority
tasks, is suppressed in Recent Changes to keep the left border
exclusively for AI indicators (see UXDR: Priority Visual Weight).
This was not a deliberate three-to-two reduction but an unrelated
side effect cleaned up separately.

## Transfer Format

For transfer across the server-function boundary, the per-actor
timestamp map is flattened into a list of (name, timestamp) pairs
sorted by timestamp descending — most recent first. The Recent
Changes UI consumes this directly to render per-author relative
timestamps in the detail panel.

## Relationship to Other Documents

- **UXDR: Recent Changes** — defines the amber/violet border semantics
  that this tracking model supports
- **UXDR: Priority Visual Weight** — explains why the left border is
  reserved for AI indicators in Recent Changes (priority-A accent
  suppressed)
- **Concept: Task Storage** — the mutation guard (`TaskMutGuard`)
  calls `add_author(actor)` on drop, connecting the dirty-tracking
  mechanism to the author trail
