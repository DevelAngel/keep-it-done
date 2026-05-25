---
status: proposed
date: 2026-05-25
---

# Server-Sent Events for Server-to-Client Push

## Context and Problem Statement

The application has two client paths — a Leptos web UI and a
tarpc CLI — that read and write the same in-memory task cache.
Communication is purely request-response: the browser uses
Leptos server functions, the CLI uses tarpc RPC calls. Neither
path learns about changes made by the other, and background
server processes (e.g. the periodic flush) have no way to
communicate outcomes to the browser at all.

How should the server push events to connected browsers so that
(a) background processes can report status, (b) changes made
via CLI are visible in the web UI without manual reload, and
(c) the push channel is lightweight enough for a family-scale
deployment?

## Decision Drivers

- Unidirectional: the server produces events, the browser
  consumes them — no client-to-server messaging needed on this
  channel
- Low infrastructure: must work with the existing Axum HTTP
  server without additional services or protocols
- Browser-native: the client-side consumer runs in WASM —
  standard browser APIs are preferred over custom protocols
- Scalable to event types: the channel must support multiple
  event categories (flush status now, task changes later)
  without per-type infrastructure
- Family scale: 2–4 concurrent connections, not thousands —
  simplicity over throughput optimization

## Considered Options

- Server-Sent Events (SSE) via Axum
- WebSocket via Axum
- Long polling via Leptos server functions
- gRPC server streaming via tonic

## Decision Outcome

Chosen option: "Server-Sent Events via Axum", because SSE is
a unidirectional, HTTP-native push protocol that matches the
data flow exactly (server → browser), requires no upgrade
handshake, works with standard browser `EventSource` API, and
is supported natively by Axum without additional dependencies.

### Architecture

The server maintains a `tokio::sync::broadcast` channel that
carries typed `ServerEvent` values. Any server-side producer
(background flush, RPC handler, future watchers) can send
events. An Axum handler at `/api/events` subscribes to the
channel and streams events as SSE to each connected browser.

```
Producers:                    Consumer:
  background_flush ──┐
  rpc_handler ───────┼──► broadcast::channel
  (future) ──────────┘         │
                               ▼
                       Axum SSE endpoint
                       /api/events
                               │
                        ┌──────┼──────┐
                        ▼      ▼      ▼
                      Tab 1  Tab 2  Tab 3
```

Each browser tab opens its own `EventSource` connection and
receives all events independently. The broadcast channel
handles fan-out; lagged receivers (slow tabs) skip missed
events rather than blocking producers.

### Event Envelope

Events use a tagged enum serialized as JSON. The tag enables
future event types without breaking existing consumers:

```json
{"type":"Flush","count":5}
{"type":"Flush","error":"2/5 tasks failed to write"}
```

### Reconnection

`EventSource` reconnects automatically on connection loss
(browser-native behavior). The server sends keep-alive
comments to prevent proxy timeouts. No event replay or
last-event-id tracking is needed — flush status and task
changes are idempotent state updates, not ordered commands.

### Consequences

- Good, because SSE is HTTP/1.1 native — no upgrade handshake,
  works through all proxies and reverse proxies without special
  configuration
- Good, because `EventSource` is a standard browser API
  available via `web_sys` — no JavaScript glue code needed
- Good, because Axum provides `axum::response::sse::Sse` in
  its core crate — no additional dependencies
- Good, because the broadcast channel decouples producers from
  consumers — adding a new event type requires only a new enum
  variant and a `send()` call
- Good, because unidirectional design matches the data flow —
  no unused server-bound channel to maintain
- Neutral, because each open tab holds one HTTP connection —
  acceptable at family scale (2–4 tabs), not suitable for
  thousands of concurrent users
- Bad, because SSE does not support binary payloads — all
  events must be JSON-serializable (acceptable: events are
  small status messages)
- Bad, because SSE is unidirectional — if bidirectional push
  is ever needed (e.g. collaborative editing), a different
  channel would be required

## Pros and Cons of the Options

### Server-Sent Events via Axum (chosen)

Standard HTTP streaming endpoint; browser uses `EventSource`.

- Good, because matches the unidirectional data flow exactly
- Good, because browser-native API with automatic reconnection
- Good, because no additional dependencies beyond Axum core
- Good, because works through HTTP proxies without
  configuration
- Bad, because unidirectional only — no client-to-server
  messages on the same channel

### WebSocket via Axum

Full-duplex connection with message framing.

- Good, because supports bidirectional communication
- Good, because Axum provides WebSocket support via
  `axum::extract::ws`
- Bad, because bidirectional capability is unused — no
  client-to-server messages needed on this channel
- Bad, because requires upgrade handshake — some proxies
  need explicit WebSocket configuration
- Bad, because no automatic reconnection — client must
  implement reconnection logic manually
- Bad, because more complex connection lifecycle (ping/pong,
  close frames) for no benefit in this use case

### Long polling via Leptos server functions

Client repeatedly calls a server function that blocks until
an event is available.

- Good, because stays within the Leptos server function model
- Bad, because each poll is a full HTTP request-response cycle
  with connection setup overhead
- Bad, because scaling to multiple event types requires either
  a multiplexed polling function or separate polls per type
- Bad, because the blocking server function holds a thread
  (or task) per waiting client — wasteful compared to SSE's
  streaming model

### gRPC server streaming via tonic

Server streams events over a gRPC connection.

- Good, because strongly typed with protobuf schemas
- Bad, because tonic + wasm-bindgen version conflicts make it
  incompatible with the Leptos WASM build (same issue as noted
  in the tarpc ADR)
- Bad, because HTTP/2 requirement adds complexity without
  benefit for local/family deployment
- Bad, because introduces protobuf tooling dependency that the
  project explicitly avoids (see tarpc ADR)

## More Information

The initial consumer of this channel is the flush status LED
(see UXDR: Flush Status LED). The background flush loop sends
an event after each flush attempt that found dirty tasks.

The broadcast channel capacity is small (16 slots). At one
flush event per 60 seconds, this provides over 15 minutes of
buffer before a lagged receiver would miss an event. Missed
events are acceptable — the next event carries the current
state.

See `kid-server/src/cache.rs` for the background flush loop
and `kid-server/src/http.rs` for the Axum HTTP server where
the SSE endpoint will be added.
