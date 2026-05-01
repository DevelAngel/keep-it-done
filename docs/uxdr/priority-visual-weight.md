---
status: accepted
date: 2026-04-30
---

# Priority Visual Weight: Accent on A Only

## Context and Problem Statement

Tasks have a priority field (A/B/C). List views show many
tasks at once. How should priority be visually communicated
in the list without increasing cognitive load — particularly
for users with ADHD?

## Decision Drivers

- ADHD-friendly: fewer competing visual patterns, not more
- One signal, one meaning — hierarchy through a single
  visual channel
- Equal readability everywhere — no task should become harder
  to read because of its priority
- Mobile-first: subtle cues must still register on small
  screens

## Considered Options

- Accent on A, dim C (opacity + font size reduction)
- Accent on A only (left border), B and C identical
- Color-coded badges on all priorities
- Background tint per priority level

## Decision Outcome

Chosen option: "Accent on A only", because it adds salience
to the one priority level that demands attention without
penalising the readability of everything else.

Implementation: a 3 px left border on the task row, using
the current view's gradient accent color (cyan for My Day,
emerald for What I Finished, amber for Quick Wins, sky for
Recent Changes).

B and C tasks have no visual distinction from each other in
the list. Priority details remain visible in the expanded
detail panel for all levels.

## Pros and Cons of the Options

### Accent on A, dim C

Highlight A tasks (left border or color accent) _and_ reduce
C tasks (lower opacity, smaller font, or indentation).

- Good, because creates a three-tier visual hierarchy
- Bad, because reduced opacity creates visual noise — ADHD
  users notice the irregularity but not necessarily in the
  intended direction
- Bad, because two font sizes force the eye to re-calibrate
  when scanning the list, increasing cognitive load
- Bad, because uses multiple visual channels simultaneously
  (opacity + size + color), violating the one-signal
  principle

### Accent on A only

A-priority tasks get a coloured left border. B and C are
visually identical — the default.

- Good, because the eye is drawn to the accent without the
  rest of the list being "punished"
- Good, because one visual channel (left border), one
  direction (presence vs. absence)
- Good, because equal readability for all tasks — hierarchy
  through addition, not deficit
- Neutral, because B and C are indistinguishable in the
  list; users who care can check the detail panel

### Color-coded badges on all priorities

Small coloured dot or letter badge (A/B/C) on every task row.

- Good, because priority is always visible for all levels
- Bad, because three distinct colours per row create a busy,
  noisy list — exactly what ADHD-friendly design avoids
- Bad, because the information density rarely justifies the
  visual cost; most tasks are B or C

### Background tint per priority level

Subtle background colour per priority (e.g. red tint for A,
neutral for B, blue tint for C).

- Good, because large colour area is easy to perceive
- Bad, because tinted backgrounds interact with the existing
  dark theme and expanded-state highlighting, creating
  complex layering
- Bad, because three background colours in a list feel
  chaotic on small screens
