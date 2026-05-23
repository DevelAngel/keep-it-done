# E2E Time Simulation Concept

## Abstract

End-to-end tests for the Upcoming view need deterministic control over
what "today" means. A test that verifies Saturday grouping must see
Saturday everywhere — in the server-side rendering, the browser after
hydration, and the seed data — regardless of the actual wall-clock day.

This concept describes an **offset-based time simulation** that shifts
the clock across all three processes (test runner, SSR server, WASM
client) without freezing it.

## Context

The Upcoming view groups tasks into date-relative buckets ("Today",
"This Week", "Next Week", etc.). Which bucket a task lands in depends
on what day it is. Testing these groupings requires the ability to
pretend the current day is something other than the real one.

### The Three-Boundary Problem

Three independent processes must agree on the fake time:

```
Test Runner           Server (SSR)           Browser (WASM)
───────────           ────────────           ──────────────
Seed dates            HTML render            today_signal init
relative to           fetch_upcoming(today)  day_label()
fake today            day_label()            Midnight-rollover
                      to_relative_time()     to_relative_time()
```

The server starts once and stays running across scenarios. Different
scenarios need different days. An environment variable at startup is
therefore insufficient — the override must be switchable per scenario
via RPC.

### The SSR-to-Hydration Consistency Problem

The `today_signal` initialization runs twice — once during SSR on the
server, once during hydration in the browser:

```rust
let today_signal = RwSignal::new(Utc::now().date_naive());
```

If the server renders with fake-Saturday but the browser hydrates with
real-Friday, Leptos detects the signal change and triggers a resource
refetch. The page briefly shows Saturday data, then overwrites it with
Friday data. Tests become flaky.

## Design

### Offset, not fixed date

Store a `chrono::Duration` offset rather than a fixed `NaiveDate`.
Every call to the central `now()` accessor returns `Utc::now() + offset`.

Rationale: a fixed date freezes time. Two calls to `now()` two minutes
apart return identical timestamps. This breaks `to_relative_time()`
(which computes "3 hours ago") and makes the midnight-rollover check
inert. An offset preserves the natural flow of time — the clock ticks
forward, just from a shifted starting point.

| Property                    | Fixed date | Offset          |
| --------------------------- | ---------- | --------------- |
| Time advances between calls | no         | yes             |
| `to_relative_time()` works  | no         | yes             |
| Midnight rollover testable  | no         | yes (in theory) |
| Complexity                  | low        | equally low     |

### Affected call sites

Six independent `Utc::now()` calls exist across the codebase. All must
route through a central accessor that applies the offset.

**Server-side (SSR):**

| Location                | Purpose                                  |
| ----------------------- | ---------------------------------------- |
| `app/src/lib.rs`        | `today_signal` — initial value at render |
| `app/src/lib.rs`        | `day_label()` — "Today"/"Yesterday"      |
| `app/src/lib.rs`        | `to_relative_time()` — "3 hours ago"     |
| `app/src/server/mod.rs` | `fetch_recently_changed()` — date groups |

**Browser-side (WASM after hydration):**

| Location         | Purpose                                    |
| ---------------- | ------------------------------------------ |
| `app/src/lib.rs` | `today_signal` — re-initialized on hydrate |
| `app/src/lib.rs` | Midnight-rollover check (60 s interval)    |
| `app/src/lib.rs` | `day_label()` — runs in WASM too           |
| `app/src/lib.rs` | `to_relative_time()` — runs in WASM too    |

**Test runner:**

| Location                         | Purpose                     |
| -------------------------------- | --------------------------- |
| `end2end/tests/browser/seeds.rs` | `date_from_relative_days()` |

### Server: shared offset state

```rust
// Same pattern as Arc<RwLock<TaskCache>>
Arc<RwLock<Option<chrono::Duration>>>
```

Provided to Leptos components via `provide_context()` so SSR rendering
can read it.

### Central `now()` accessor

A single function replaces all `Utc::now()` calls:

```rust
fn now() -> DateTime<Utc> {
    match get_offset() {
        Some(offset) => Utc::now() + offset,
        None         => Utc::now(),
    }
}

fn today() -> NaiveDate {
    now().date_naive()
}
```

On the server, `get_offset()` reads the shared `Arc<RwLock<…>>`.
In the browser, `get_offset()` reads a meta tag injected by SSR.
In production, the offset is always `None` — zero overhead.

### Meta tag bridge (SSR to WASM)

When an offset is active, the server injects it into the HTML:

```html
<meta name="kid-time-offset-seconds" content="86400" />
```

The WASM `now()` reads this once at startup:

```rust
fn get_offset() -> Option<chrono::Duration> {
    // Browser: parse <meta name="kid-time-offset-seconds">
    // SSR:     read from context Arc<RwLock<…>>
}
```

Because SSR and WASM use the same offset value, `today_signal` is
initialized identically on both sides. No signal mismatch, no refetch,
no flicker.

### RPC endpoints

Two new methods on the `TaskService` trait:

```rust
async fn set_time_offset(seconds: i64);
async fn reset_time_offset();
```

The offset is stored in seconds for wire simplicity. The test computes
the offset from the desired day:

```rust
let desired = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap(); // Saturday
let offset = desired - Utc::now().date_naive();               // +1 day
rpc.set_time_offset(offset.num_seconds()).await;
```

### Seed adaptation

`date_from_relative_days()` receives a reference date parameter
instead of calling `Utc::now()` internally:

```rust
fn date_from_relative_days(s: &str, reference: DateTime<Utc>) -> TaskDate {
    let days: i64 = s.parse().unwrap();
    let date = reference.fixed_offset() + TimeDelta::days(days);
    TaskDate { date, soft: false }
}
```

The reference date is derived from the same offset the server uses, so
seed dates and server grouping are always consistent.

### Test flow per scenario

```
Before-Hook:
  1. Create temp_dir
  2. rpc.switch_dir(temp_dir)
  3. rpc.set_time_offset(offset_seconds)   // e.g. +86400 for Saturday
  4. Seed tasks with dates relative to fake today

Scenario:
  5. Browser navigates to /upcoming
  6. SSR renders with offset → correct day buckets
  7. WASM reads <meta> → same offset → no hydration mismatch
  8. Assert: grouping labels match expectations

After-Hook:
  9. rpc.reset_time_offset()
  10. rpc.switch_dir(original)
  11. Cleanup temp_dir
```

## Files to change

| File                             | Change                                                 |
| -------------------------------- | ------------------------------------------------------ |
| `types/src/service.rs`           | Add `set_time_offset` / `reset_time_offset` to trait   |
| `server/src/main.rs`             | `Arc<RwLock<Option<Duration>>>` in app state + context |
| `server/src/rpc.rs`              | Implement new RPC methods                              |
| `app/src/lib.rs`                 | Central `now()`/`today()`, replace all `Utc::now()`    |
| `app/src/server/mod.rs`          | `fetch_recently_changed()` uses `today()`              |
| `cli/src/main.rs`                | Client stubs for new RPC methods                       |
| `end2end/tests/browser/seeds.rs` | `date_from_relative_days()` takes reference param      |
| `end2end/tests/browser/main.rs`  | Before/after hooks call offset RPC                     |
| Shell / HTML template            | Conditional `<meta>` tag injection                     |

## Production safety

In production, no offset is ever set. The `Arc<RwLock<…>>` holds
`None`, and the `now()` accessor falls through to plain `Utc::now()`.
The meta tag is not emitted. The only cost is one `RwLock::read()` per
request — negligible compared to the template render.

The RPC methods are no different from `switch_dir` which already exists
for test isolation. Both are administrative endpoints that a production
deployment does not expose externally.
