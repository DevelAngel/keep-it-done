---
status: proposed
date: 2026-05-26
---

# Task Change Notification via Guard Callback

## Context and Problem Statement

The application supports two mutation paths:
the Leptos web UI (server functions) and the tarpc CLI.
Both modify the same in-memory task cache.
An SSE push channel already delivers flush status events to connected browsers (see ADR: Server-Sent Events),
but task mutations themselves are not broadcast.
When one user completes, renames, or recategorizes a task, other users see stale data until they navigate or reload.

How should the server detect and broadcast task mutations so that
(a) every mutation path automatically emits a change event,
(b) the types crate stays decoupled from server infrastructure, and
(c) the mechanism is impossible to forget when new mutation endpoints are added?

## Decision Drivers

- Automatic: every mutation must emit an event without requiring the developer to remember a manual `send()` call
- Decoupled: `kid-types` must not depend on `tokio::sync` or the event bus — it is shared between CLI, server, and WASM
- Single responsibility: the existing `TaskMutGuard` already marks tasks as dirty on drop — change notification should follow the same lifecycle
- Scalable to mutation paths: Leptos server functions (~15 endpoints) and RPC handlers (~10 methods) must both emit events without duplicated plumbing
- Low overhead: the callback fires in the drop path — it must not block, allocate excessively, or panic

## Considered Options

- Explicit emit at each mutation site
- Event bus reference inside `TaskMutGuard`
- Guard callback via `Box<dyn FnOnce>`
- Cache-level generation counter

## Decision Outcome

Chosen option: "Guard callback via `Box<dyn FnOnce>`", because it makes event emission automatic (no call site can forget it),
keeps `kid-types` decoupled from the event bus (the callback is injected by the server),
and follows the existing `TaskMutGuard` lifecycle that already handles dirty-marking on drop.

### Mechanism

`TaskMutGuard` gains an optional callback field:

```
TaskMutGuard<'a> {
    id: Uuid,
    task: &'a mut Task,
    dirty: &'a mut ChangeSet,
    actor: String,
    on_drop: Option<Box<dyn FnOnce(Uuid, String) + Send>>,
}
```

On `Drop`, after marking the task dirty and recording the author, the guard calls the callback with `(id, actor)`.
The callback is injected by the caller of `TaskCache::get_mut()`.

### Injection Point

`TaskCache::get_mut()` gains a third parameter for the callback:

```
pub fn get_mut(
    &mut self,
    id: &Uuid,
    actor: impl Into<String>,
    on_change: impl FnOnce(Uuid, String) + Send + 'static,
) -> Option<TaskMutGuard<'_>>
```

The server wraps the event bus in a closure at each call site:

```
let bus = event_bus.clone();
cache.get_mut(&id, &actor, move |id, actor| {
    let _ = bus.send(ServerEvent::TaskChanged { id, actor });
});
```

Because the closure captures only a `broadcast::Sender` clone (which is `Clone + Send`), the types crate never sees the event bus type.

### Event Variant

`ServerEvent` gains a new variant:

```
enum ServerEvent {
    Flush(FlushOutcome),
    TaskChanged { id: Uuid, actor: String },
}
```

The tagged JSON envelope keeps backward compatibility — existing consumers ignore unknown variants.

### `add()` and `remove()`

`TaskCache::add()` and `remove()` do not use the guard pattern.
These methods accept an optional callback directly:

```
pub fn add_with_callback(
    &mut self,
    task: Task,
    on_change: impl FnOnce(Uuid, String) + Send + 'static,
) -> Uuid
```

Alternatively, they can return the ID and let the caller emit explicitly — acceptable because add/remove are single-line operations unlikely to be forgotten.

### Consequences

- Good, because every `get_mut()` call site must provide a callback — the compiler enforces it, making silent omission impossible
- Good, because `kid-types` has no dependency on `tokio::sync`, `broadcast`, or any server crate — the callback is a plain `FnOnce`
- Good, because the existing drop lifecycle is reused — no new mechanism to understand or maintain
- Good, because the same callback pattern works for both Leptos server functions and RPC handlers without code duplication
- Neutral, because `add()` and `remove()` require separate handling since they do not use `TaskMutGuard` — acceptable given their small count (2-3 call sites each)
- Bad, because every `get_mut()` call site must construct a closure — mitigated by a helper method on `SharedTaskCache` that captures the event bus once
- Bad, because `Box<dyn FnOnce>` adds one heap allocation per mutation — negligible at family scale (single-digit mutations per minute)
- Bad, because tests must provide a callback (or `None`) even when they do not care about notifications — mitigated by making the field `Option<Box<...>>` and providing a convenience `get_mut_silent()` for tests

## Pros and Cons of the Options

### Explicit emit at each mutation site

Each Leptos server function and RPC handler calls `event_bus.send(TaskChanged { ... })` after mutating.

- Good, because straightforward — no abstraction to learn
- Good, because `kid-types` remains completely unchanged
- Bad, because ~25 mutation sites must each remember to emit — a single omission silently breaks notifications for that path
- Bad, because the emit call is boilerplate duplicated across every mutation endpoint
- Bad, because new endpoints added in the future can easily forget the emit call with no compiler warning

### Event bus reference inside TaskMutGuard

`TaskMutGuard` holds an `Arc<broadcast::Sender<ServerEvent>>` and sends directly on drop.

- Good, because fully automatic — same as the callback approach
- Bad, because `kid-types` must depend on `tokio::sync` and know the `ServerEvent` type — circular dependency between types and app crates
- Bad, because the guard becomes non-`Send` or requires `Arc` wrapping, complicating the borrow-based design
- Bad, because test code must construct a broadcast channel even when testing pure task logic

### Guard callback via Box\<dyn FnOnce\> (chosen)

Guard receives an opaque callback; server injects a closure over the event bus.

- Good, because automatic emission — no call site can forget
- Good, because `kid-types` stays decoupled — callback is plain Rust, no external types
- Good, because testable — tests pass a no-op closure or `None` to verify mutation logic without event infrastructure
- Bad, because one `Box` allocation per mutation — negligible at family scale
- Bad, because API change on `get_mut()` — all existing call sites must be updated (one-time migration cost)

### Cache-level generation counter

`TaskCache` maintains an `AtomicU64` generation counter bumped on every mutation.
The server periodically checks the counter and broadcasts a generic "data changed" event.

- Good, because trivially simple — one atomic increment
- Good, because no per-task event, no callback plumbing
- Bad, because coarse-grained — client must refetch all tasks even when only one changed
- Bad, because no actor information — UI cannot show who made the change
- Bad, because polling the counter reintroduces latency that SSE was chosen to eliminate

## More Information

The SSE push channel and `ServerEvent` enum are defined in ADR:
[Server-Sent Events for Server-to-Client Push](server-sent-events.md).
The first consumer of the channel is UXDR:
[Flush Status LED](../uxdr/flush-status-led.md).

The client-side reaction to `TaskChanged` events — silent refetch, inline highlight, and conflict warning — is documented in UXDR:
[Task Change Display](../uxdr/task-change-display.md).

The `on_drop` callback must not panic. If `broadcast::send()` fails (no receivers), the error is silently ignored — the event is best-effort.
This matches the existing flush event behavior where lagged receivers skip missed events.
