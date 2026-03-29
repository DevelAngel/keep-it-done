# Software Architecture Overview

## Abstract

This document provides a comprehensive overview of the family task management system architecture. The system serves two distinct user groups through separate interfaces: an AI assistant accessing tasks via CLI over RPC, and family members viewing tasks through a browser interface. Both interfaces share a common business logic layer and task storage.

## System Context

```
┌─────────────────┐                    ┌──────────────────┐
│   AI Assistant  │                    │ Family Member    │
│                 │                    │   (Browser)      │
└────────┬────────┘                    └────────┬─────────┘
         │                                      │
         │ Invoke CLI                           │ HTTPS/HTTP
         │                                      │
         ▼                                      ▼
┌─────────────────┐                    ┌──────────────────┐
│   kid (CLI)     │                    │   Web Browser    │
│                 │                    │                  │
└────────┬────────┘                    └────────┬─────────┘
         │                                      │
         │ TCP RPC (tarpc + JSON)               │ HTTP
         │ default: 127.0.0.1:9000             │ Leptos Server Functions
         │                                      │
         └──────────────┬───────────────────────┘
                        │
                        ▼
            ┌──────────────────────┐
            │   kid-server         │
            │   (Web Server)       │
            │                      │
            │  ┌────────────────┐  │
            │  │  RPC Handler   │  │
            │  └────────┬───────┘  │
            │           │          │
            │  ┌────────▼───────┐  │
            │  │ Business Logic │  │
            │  │ (kid-types/    │  │
            │  │  kid-app)      │  │
            │  └────────┬───────┘  │
            │           │          │
            │  ┌────────▼───────┐  │
            │  │  TaskCache     │  │
            │  │  (In-Memory    │  │
            │  │   + Dirty      │  │
            │  │   Tracking)    │  │
            │  └────────────────┘  │
            └──────────────────────┘
                        │
                        │ File I/O (periodic flush)
                        ▼
            ┌──────────────────────┐
            │   Filesystem         │
            │   tasks/*.json       │
            └──────────────────────┘
```

## Component Architecture

### Crate Organization

The system uses a Cargo workspace with the following crates:

```
kid/
├── types/               # kid-types: shared types, RPC trait, storage logic
│   ├── derive/          # kid-types-derive: proc-macro helpers
│   └── src/
│       ├── lib.rs       # Task, Status, traits (TaskId, TaskInfos, TaskDetails)
│       ├── task.rs      # Task struct and related types
│       ├── rpc.rs       # tarpc TaskService trait definition (feature=rpc)
│       └── server.rs    # TaskCache, load/flush, error types (feature=ssr)
│
├── app/                 # kid-app: Leptos app (shared SSR + WASM)
│   └── src/
│       ├── lib.rs       # App component, shell
│       ├── server.rs    # SharedTaskCache, server functions (feature=ssr)
│       ├── cache.rs     # Frontend cache / reactive state
│       └── error_template.rs
│
├── frontend/            # kid-frontend: WASM binary (cdylib)
│   └── src/
│       └── lib.rs       # WASM entry point (feature=hydrate)
│
├── server/              # kid-server: server binary
│   └── src/
│       ├── main.rs      # Server initialization, graceful shutdown
│       ├── builder.rs   # ServerBuilder (composes RPC + HTTP listeners)
│       ├── cache.rs     # SharedTaskCache + background flush logic
│       ├── cli.rs       # CLI args (--listen addr)
│       ├── rpc.rs       # tarpc TCP server (JSON transport)
│       └── http.rs      # Axum + Leptos HTTP server
│
└── cli/                 # kid-cli: CLI binary (`kid`)
    └── src/
        ├── main.rs      # Command parsing, tarpc TCP client, JSON output
        ├── cli.rs       # Clap definitions
        └── task.rs      # Task display / patch helpers
```

### Feature Flags (`kid-types`)

`kid-types` is compiled selectively via feature flags:

| Feature       | Activates                                             |
|---------------|-------------------------------------------------------|
| `rpc`         | `tarpc` service trait                                 |
| `cli`         | `rpc` + `schemars` + `clap` (for CLI JSON schema)    |
| `ssr`         | `rpc` + file storage, `uuid/v7`, `ahash`, `indexmap` |
| `ssr-test-*`  | Test helpers (random data, forced storage failures)   |

### Dependency Graph

```
    kid-cli            kid-server
        │                   │
        │ (feature=cli)      │ (feature=ssr)
        └────────┬───────────┘
                 │
                 ▼
            kid-types
                 │
            kid-types-derive

    kid-server ──(feature=ssr)──> kid-app ──> kid-types
    kid-frontend ─(feature=hydrate)> kid-app
```

## Data Flow

### Read Operation (List Tasks)

```
AI Assistant                                          Server
     │                                                   │
     │  1. Invoke: kid list                             │
     ├──────────────────────────────────────────────────┤
     │                                                   │
   kid                                                  │
     │  2. Connect to TCP socket (127.0.0.1:9000)       │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │  3. tarpc call: list()                           │
     │     JSON over TCP                                │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │                                          rpc handler
     │                                                   │
     │                                  4. Acquire read lock
     │                                     on SharedTaskCache
     │                                                   │
     │                                  5. Return Vec<(Uuid, Task)>
     │                                                   │
     │  6. tarpc response                               │
     <───────────────────────────────────────────────────┤
     │                                                   │
   kid                                                  │
     │  7. Format as JSON / table                       │
     │                                                   │
     │  8. Print to stdout                              │
     ├──────────────────────────────────────────────────┤
     │                                                   │
AI Assistant                                            │
     │  9. Parse output                                 │
     │                                                   │
```

### Write Operation (Add Task)

```
AI Assistant                                          Server
     │                                                   │
     │  1. Invoke: kid add "Do something"               │
     ├──────────────────────────────────────────────────┤
     │                                                   │
   kid                                                  │
     │  2. tarpc call: add(task)                        │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │                                          rpc handler
     │                                                   │
     │                                  3. Acquire write lock
     │                                     on SharedTaskCache
     │                                                   │
     │                                  4. Insert task, mark dirty
     │                                                   │
     │  5. tarpc response (ack)                         │
     <───────────────────────────────────────────────────┤
     │                                                   │
   kid                                                  │
     │  6. Print confirmation JSON                      │
     │                                                   │
```

Dirty tasks are persisted to disk by the background flush task (every 60 s) or on graceful shutdown.

### Browser Access (Concurrent with CLI)

```
Browser                                              Server
   │                                                    │
   │  1. HTTP GET /                                    │
   ├───────────────────────────────────────────────────>
   │                                                    │
   │                                          http handler
   │                                                    │
   │  2. Serve Leptos SSR + WASM hydration             │
   <────────────────────────────────────────────────────┤
   │                                                    │
   │  3. Leptos Server Function (e.g. get_tasks)       │
   ├───────────────────────────────────────────────────>
   │                                                    │
   │                                          http handler
   │                                                    │
   │                                  4. Reads SharedTaskCache
   │                                     (same cache as RPC)
   │                                                    │
   │  5. Return tasks in response                      │
   <────────────────────────────────────────────────────┤
   │                                                    │
   │  6. Leptos renders UI reactively (WASM)           │
   │                                                    │
```

## Storage Architecture

### File Layout

```
tasks/
├── <uuid-v7>.json    # One file per task
├── <uuid-v7>.json
└── ...
```

Each task is a self-contained JSON file. The UUID v7 filename encodes the creation timestamp.

### In-Memory Cache Structure

```
┌───────────────────────────────────────┐
│           TaskCache                   │
├───────────────────────────────────────┤
│                                       │
│  tasks: DataMap                       │
│  ┌────────────────────────────────┐   │
│  │ IndexMap<Uuid, Task>           │   │
│  │ (insertion-ordered, ahash)     │   │
│  │                                │   │
│  │ uuid-001 -> Task { ... }       │   │
│  │ uuid-002 -> Task { ... }       │   │
│  └────────────────────────────────┘   │
│                                       │
│  dirty: ChangeSet                     │
│  ┌────────────────────────────────┐   │
│  │ IndexSet<Uuid>                 │   │
│  │ (tasks modified since last     │   │
│  │  flush)                        │   │
│  └────────────────────────────────┘   │
│                                       │
└───────────────────────────────────────┘
```

### Cache Consistency and Flush Strategy

`SharedTaskCache` is an `Arc<RwLock<TaskCache>>` shared between the RPC and HTTP handlers.

```
Write Path:
CLI/Browser -> RPC/HTTP -> write lock -> mutate cache -> mark dirty
                                                 │
                              background flush (every 60s) or final_flush()
                                                 │
                                          write_file() for each dirty task
                                          (atomic write)

Read Path:
CLI/Browser -> RPC/HTTP -> read lock -> read from IndexMap
                                     (always consistent; server is sole writer)
```

The background flush retries on failure with an exponential back-off interval.

## Communication Protocols

### RPC Protocol (CLI ↔ Server)

Transport: TCP socket, configurable via `--listen` (default `127.0.0.1:9000`)

Framework: `tarpc` with JSON serialization (`tokio_serde::formats::Json`)

```
RPC service (kid-types/src/rpc.rs):

  list()                             -> Vec<(Uuid, Task)>
  add(task: Task)
  rename(id: Uuid, summary: String)
  replace(id: Uuid, details: TaskDetails)
  update(id: Uuid, details: TaskDetailsPatch)
  complete(id: Uuid, reopen: bool)
```

### HTTP Protocol (Browser ↔ Server)

Transport: TCP, port configured via Leptos config (default `127.0.0.1:3000`, reload port `3001`)

Framework: Axum + Leptos (SSR + WASM hydration)

- Leptos Server Functions handle all browser ↔ server data exchange
- Static assets served from `public/`, built site in `target/site/`
- Tailwind CSS built from `style/tailwind.css`

## Concurrency Model

### Server Threading

```
┌──────────────────────────────────────────────┐
│             kid-server Process               │
├──────────────────────────────────────────────┤
│                                              │
│  Tokio Runtime (async multi-threaded)        │
│                                              │
│  ┌──────────────┐  ┌─────────────────────┐  │
│  │ RPC Listener │  │ HTTP Listener       │  │
│  │ (TCP :9000)  │  │ (Axum/Leptos :3000) │  │
│  └──────┬───────┘  └──────────┬──────────┘  │
│         │                     │             │
│         ▼                     ▼             │
│  ┌─────────────────────────────────────┐    │
│  │  SharedTaskCache                    │    │
│  │  (Arc<RwLock<TaskCache>>)           │    │
│  │                                     │    │
│  │  - Multiple concurrent readers OK   │    │
│  │  - Single writer blocks all         │    │
│  └─────────────────────────────────────┘    │
│         │                                   │
│  ┌──────▼──────────────────────────────┐    │
│  │  Background flush task              │    │
│  │  (flushes dirty set every 60s)      │    │
│  └─────────────────────────────────────┘    │
│                                              │
└──────────────────────────────────────────────┘
```

### CLI Tool

```
┌─────────────────────────┐
│   kid Process           │
├─────────────────────────┤
│                         │
│  1. Parse args (clap)   │
│  2. Connect TCP socket  │
│  3. tarpc RPC call      │
│  4. Wait for response   │
│  5. Format output       │
│  6. Print and exit      │
│                         │
│  (Short-lived process)  │
└─────────────────────────┘
```

## Error Handling Strategy

### RPC Layer Errors

Errors at different layers:
- **Transport errors**: Connection refused, wrong address → `kid` exits with miette diagnostic
- **Serialization errors**: Invalid JSON data → tarpc returns error
- **Business logic errors**: Task not found, validation failed → `Result<T, E>` in response
- **Storage errors**: File write failed → `FlushErrors` with per-file details, server logs warning and retries

### Browser Error Handling

Leptos server functions return `Result<T, ServerFnError>`. The browser UI renders errors inline without crashing.

## Deployment Model

### Development

```
Developer Machine
├── Terminal 1: cargo leptos watch   (or: cargo run --bin kid-server)
├── Terminal 2: cargo run --bin kid -- list
└── Browser: http://localhost:3000
```

RPC server listens on `127.0.0.1:9000`. HTTP on `127.0.0.1:3000`.

### Production (Family Server)

```
Family Server (e.g., Raspberry Pi, NUC)
├── systemd unit: kid-server.service
│   └── RPC:  127.0.0.1:9000 (local only)
│   └── HTTP: 0.0.0.0:3000 (or behind nginx/HTTPS)
│
└── kid installed in PATH
    └── Family members can SSH and use CLI
    └── AI assistant runs CLI via automation
```

Nginx or similar can provide HTTPS termination and static asset caching for the browser UI.

## Security Considerations

### RPC Security

The RPC server binds to `127.0.0.1` by default, so it is only reachable from the same host. Override with `--listen` if SSH tunnelling is needed.

### HTTP Security

For local network deployment:
- No authentication (trusted network)
- HTTPS optional (can use self-signed cert or nginx termination)

For remote access:
- Add authentication tokens
- Require HTTPS
- Rate limiting on API endpoints

## Scalability and Performance

### Current Constraints

- **Users**: 1-4 family members
- **Tasks**: 100-1000 active tasks
- **Requests**: Single-digit requests per second
- **Memory**: <10 MB for task cache

### Performance Characteristics

```
Operation          Latency     Notes
──────────────────────────────────────────────────
List tasks (RPC)   < 1ms       Read lock, no I/O
List tasks (HTTP)  < 5ms       Same cache path
Add/update task    < 5ms       Write lock, no sync I/O
Flush to disk      < 100ms     Only dirty tasks
Server startup     < 200ms     Load all task files
```

All in-memory operations are sub-millisecond. File I/O is deferred to background flush.

### Bottlenecks

Current architecture has no bottlenecks at family scale. Theoretical limits:
- Write lock contention if >100 concurrent writers
- File I/O if >1000 tasks flushed per cycle
- Memory if >100k tasks loaded

None of these are realistic for the target use case.

## Future Extensions

### Potential Additions

1. **Real-time browser updates**: Server-Sent Events or WebSockets to push changes to browsers
2. **Multi-device sync**: CRDTs or operational transforms for offline editing
3. **Search**: Full-text search index for task descriptions and notes
4. **Attachments**: File uploads associated with tasks
5. **Recurrence**: Repeating tasks with schedules
6. **Collaboration**: Comments and mentions on tasks

Each extension would follow the same pattern: add to business logic in `kid-types`/`kid-app`, expose via both RPC and HTTP interfaces.

### Migration Path

If family grows beyond target scale:
- Replace file storage with SQLite (still zero-infrastructure)
- Add connection pooling for RPC
- Implement pagination for task lists

The clean separation between business logic and transport makes these migrations tractable.
