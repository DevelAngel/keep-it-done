---
status: proposed
date: 2026-05-25
---

# Flush Status LED: Persistent Save Indicator

## Context and Problem Statement

The task cache flushes dirty tasks to disk every 60 seconds in a
background loop. Success and failure are logged via `tracing` on
the server — the user sees nothing. There is no server-to-client
push channel; the UI is purely request-response (Leptos server
functions).

How should the flush outcome be communicated to the user so that
(a) successful persistence is acknowledged, (b) flush errors are
noticed and not silently lost, and (c) the notification mechanism
serves as groundwork for future server-pushed events (e.g. "CLI
changed a task")?

## Decision Drivers

- ADHD-friendly: add salience for errors without adding cognitive
  load during normal operation — no recurring toast every 60 s
- Touch-first: hover-dependent interactions are unusable on phones
  and tablets
- Scroll-independent: the indicator must remain visible regardless
  of scroll position
- Non-intrusive: must not overlay input fields, task cards, or the
  edit-mode amber bars
- Extensible: the server-to-client channel should support future
  event types beyond flush
- Minimal infrastructure: avoid building a full toast/snackbar
  system when a simpler primitive suffices

## Considered Options

- Server-Sent Events with toast notifications
- Server-Sent Events with status LED
- Polling via server function with toast
- Piggyback flush status on existing server function responses
- Static status indicator in the page header

## Decision Outcome

Chosen option: "Server-Sent Events with status LED", because it
combines a lightweight, always-visible indicator (no text, no
overlay) with a real-time push channel that can be reused for
future server events. The LED adds zero cognitive load during
normal operation (invisible when idle, briefly visible on
success) and persistent salience only when something is wrong.

### Push Channel: Server-Sent Events

A broadcast channel carries server events from any server-side
producer. An Axum SSE endpoint at `/api/events` streams these
to connected browsers. The event enum is tagged for future
extensibility (e.g. CLI task changes). On the client, an
`EventSource` connects to `/api/events` and feeds a reactive
signal that drives the LED component.

### Visual Design: Status LED

A small circle (8–10 px), fixed-positioned in the bottom-right
corner of the viewport. It functions like a hardware status LED —
presence and color encode state, nothing else.

```
┌──────────────────────────────┐
│                              │
│   Content (scrolls)          │
│                              │
│                              │
│                           ●  │  ← fixed, bottom-right
└──────────────────────────────┘
```

Position: `fixed bottom-4 right-4 z-50`. The LED sits outside the
content flow and never overlaps input fields or task cards.

### Interaction States

| State                           | LED                                               | Click             | Auto-Dismiss             |
| ------------------------------- | ------------------------------------------------- | ----------------- | ------------------------ |
| Idle (nothing dirty)            | invisible                                         | —                 | —                        |
| Flush success (`Ok(n)`, n > 0)  | green                                             | no action         | fades after 3 s          |
| Flush error                     | red, persistent                                   | opens error panel | stays until next success |
| Error panel open                | red, dimmed (`opacity-20`)                        | closes panel      | —                        |
| Retry while panel open          | LED pulses briefly (`opacity-100` → `opacity-20`) | —                 | —                        |
| Retry succeeds (panel was open) | green (3 s), panel closes                         | —                 | panel + LED disappear    |

### Error Panel

Clicking the red LED opens a fixed-position detail panel at the
bottom of the viewport. The panel shows the error message from
the last failed flush.

```
Normal error:                    Edit mode (+ amber bars):
┌──────────────────────────┐     ═══════════════════════════  ← amber top
│   Content             🔴 │     │   Content             🔴 │
├──────────────────────────┤     ═══════════════════════════  ← amber at panel top
│ ⚠ Flush error:           │     │ ⚠ Flush error:           │
│ 2/5 tasks failed to      │     │ 2/5 tasks failed to      │
│ write to disk            │     │ write to disk            │
└──────────────────────────┘     └──────────────────────────┘
```

In edit mode, the existing bottom amber bar moves to the top
edge of the error panel (or the panel receives
`border-top: 3px solid amber-400`), and the `fixed bottom-0`
bar is suppressed while the panel is open. This preserves the
visual boundary between content and chrome.

### LED Dimming While Panel Is Open

When the error panel is open, the LED dims to `opacity-20`. Its
red color is redundant with the panel content. On each failed
retry, the LED briefly pulses to full opacity — a heartbeat
signaling "still trying" — then dims again. When a retry
succeeds, the LED turns green, the panel closes, and both
dismiss after 3 seconds.

## Consequences

- Good, because the LED is invisible during normal operation —
  zero cognitive load when flushes succeed on schedule
- Good, because persistent red on error ensures flush failures
  are never silently lost, even if the user was not looking
- Good, because no hover dependency — all interactions are
  click/tap, working equally on desktop and touchscreens
- Good, because the SSE channel is a general-purpose primitive
  reusable for future events (CLI task changes, multi-user sync)
- Good, because the LED's fixed position at bottom-right avoids
  collision with the top/bottom edit-mode amber bars, the header,
  navigation, and input fields
- Good, because the error panel uses the same amber top-border
  convention as edit mode, maintaining visual consistency
- Neutral, because the SSE connection is always open per browser
  tab — acceptable for a family-sized user base
- Bad, because the LED alone carries no textual detail — the user
  must click to read the error message, adding one interaction
  step — mitigated by the LED's persistent visibility ensuring
  the error is not missed

## Pros and Cons of the Options

### SSE with toast notifications

Server pushes events via SSE; a toast/snackbar slides in with
text describing the flush outcome.

- Good, because toast text is immediately readable without
  extra interaction
- Bad, because a success toast every 60 seconds creates
  recurring visual noise — hostile to ADHD users
- Bad, because toasts overlay content and compete with input
  fields and the edit-mode bars for screen space
- Bad, because a full toast system (stacking, auto-dismiss,
  z-index management, animation) is significant infrastructure
  for a single use case

### SSE with status LED (chosen)

Server pushes events via SSE; a fixed-position LED circle
encodes state via presence and color.

- Good, because the LED is the smallest possible visual
  primitive — minimal cognitive load
- Good, because color alone (green/red) is unambiguous for a
  binary outcome (saved / not saved)
- Good, because the LED can be extended to pulse or change
  color for future event types without layout changes
- Bad, because color-only encoding is not accessible to
  color-blind users — mitigated by the click-to-open panel
  providing textual detail

### Polling via server function with toast

Client polls every 30 seconds via a Leptos server function;
displays result as a toast.

- Good, because it stays within the Leptos server function
  ecosystem — no new endpoint type
- Bad, because polling adds up to 30 seconds delay between
  flush and notification
- Bad, because periodic HTTP requests waste bandwidth when
  nothing has changed
- Bad, because polling does not scale to future event types
  that need low-latency delivery (e.g. CLI task changes)

### Piggyback on existing server function responses

Each server function response includes a flush-status sideband
field.

- Good, because no additional network traffic
- Bad, because the user only sees status during active
  interaction — flush errors between clicks are invisible
- Bad, because it couples flush status to unrelated API
  signatures, violating separation of concerns

### Static indicator in page header

A small icon in the header shows the last flush result, updated
on page navigation.

- Good, because no push channel needed
- Bad, because the header scrolls out of view — the indicator
  is invisible when the user is working in the task list
- Bad, because the indicator only updates on navigation, not
  in real time — flush errors may go unnoticed for minutes

## More Information

### Event Flow

```
background_flush()
    │ cache.flush() → Ok(n>0) or Err(e)
    ▼
SharedEventBus (broadcast channel)
    │
    ▼
SSE Endpoint /api/events (Axum)
    │
    ▼
EventSource (Browser, WASM)
    │
    ▼
Reactive Signal
    │
    ▼
<FlushStatusLed/>  →  click  →  <FlushErrorPanel/>
```

### Accessibility

The LED uses `role="status"` and `aria-live="polite"` with a
visually hidden text label ("Flush successful" / "Flush error —
tap for details"). Screen readers announce state changes without
the user needing to perceive color. The error panel uses
`role="alert"` when opened.

### Color-Blindness Mitigation

The LED encodes state as green (success) vs. red (error). For
users who cannot distinguish these colors, the behavioral
difference provides a secondary signal: the success LED
auto-dismisses (transient), while the error LED persists and is
clickable. The error panel provides full textual detail.

### Future Extension

The `ServerEvent` enum and SSE channel are designed for
additional event types. When CLI-triggered task changes are
implemented, a `TaskChanged { id, actor }` variant can be added
without modifying the push infrastructure. The LED component can
be extended to show a brief indicator for external changes, or a
separate component can subscribe to the same `EventSource`.
