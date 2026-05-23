---
status: accepted
date: 2026-05-23
---

# Central Time Accessor Instead of Direct `Utc::now()`

## Context and Problem Statement

The codebase calls `Utc::now()` in six independent locations across server (SSR), browser (WASM), and test code. Each call is an implicit dependency on wall-clock time. This makes end-to-end tests non-deterministic — the Upcoming view groups tasks by relative day, so a test written on Friday produces different results on Saturday. A central accessor is needed so that all time-dependent code draws from one controllable source.

## Decision Drivers

- E2E tests must simulate arbitrary days (e.g. Saturday) regardless of when they run
- SSR-rendered HTML and WASM-hydrated state must agree on "today" to avoid hydration mismatches
- Production code must not pay a meaningful cost for testability
- The solution must work across process boundaries (server ↔ browser)

## Considered Options

- Direct `Utc::now()` everywhere (status quo)
- Compile-time feature flag with a fixed fake date
- Central `now()` accessor with injectable offset

## Decision Outcome

Chosen option: "Central `now()` accessor with injectable offset", because it satisfies all four decision drivers simultaneously. A `Duration` offset shifts the clock without freezing it, so relative-time calculations ("3 hours ago") and midnight-rollover checks continue to work naturally. The offset is `None` in production — the accessor degrades to a plain `Utc::now()` call.

### The rule

**No code outside the accessor module may call `Utc::now()` directly.** All time-dependent code uses `kid_time::now()` or `kid_time::today()`. This applies to `app/`, `server/`, and test seed code.

### Consequences

- Good, because E2E tests become day-deterministic — each scenario sets its own offset via RPC
- Good, because SSR and WASM hydration see the same offset (transmitted via `<meta>` tag), eliminating signal mismatches
- Good, because the offset is a single `Arc<RwLock<Option<Duration>>>` read per request — negligible overhead
- Good, because relative-time formatting and midnight rollover keep working (time advances, only the origin shifts)
- Bad, because every new `Utc::now()` call must be caught in review — the rule is convention, not compiler-enforced (mitigation: a `grep` in CI can flag direct `Utc::now()` calls outside the accessor module)
- Bad, because the `<meta>` tag bridge adds a cross-process coupling for test infrastructure (mitigation: the tag is only emitted when an offset is active; production HTML is unchanged)

## Pros and Cons of the Options

### Direct `Utc::now()` everywhere (status quo)

- Good, because zero indirection — simple and obvious
- Bad, because untestable — no way to simulate a different day without `libfaketime` or OS-level tricks
- Bad, because SSR and WASM inevitably disagree on the current instant, causing hydration flicker in time-sensitive views

### Compile-time feature flag with a fixed fake date

- Good, because no runtime cost in production
- Bad, because requires recompilation to change the simulated day
- Bad, because a fixed date freezes time — `to_relative_time()` returns the same value forever, midnight rollover never triggers
- Bad, because the server cannot switch days between scenarios without restarting

### Central `now()` accessor with injectable offset

- Good, because offset is per-process state, switchable at runtime via RPC
- Good, because time still advances — only the origin shifts
- Good, because `None` offset compiles down to a plain `Utc::now()` call
- Neutral, because adds one module and one level of indirection to all time access

## More Information

The full design — including the three-boundary problem, meta-tag bridge, RPC endpoints, and per-scenario test flow — is documented in [E2E Time Simulation Concept](../concepts/e2e-time-simulation.md).
