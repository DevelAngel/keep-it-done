# Software Architecture Overview

## Abstract

This document provides a comprehensive overview of the family task management system architecture. The system serves two distinct user groups through separate interfaces: an AI assistant accessing tasks via CLI over IPC, and family members viewing tasks through a browser interface. Both interfaces share a common business logic layer and task storage.

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
│   task-cli      │                    │   Web Browser    │
│   (CLI Tool)    │                    │                  │
└────────┬────────┘                    └────────┬─────────┘
         │                                      │
         │ IPC (Unix Socket)                    │ HTTP
         │ Binary RPC Protocol                  │ REST/Server Functions
         │                                      │
         └──────────────┬───────────────────────┘
                        │
                        ▼
            ┌──────────────────────┐
            │   task-server        │
            │   (Web Server)       │
            │                      │
            │  ┌────────────────┐  │
            │  │  IPC Handler   │  │
            │  └────────┬───────┘  │
            │           │          │
            │  ┌────────▼───────┐  │
            │  │ Business Logic │  │
            │  │ (task-service) │  │
            │  └────────┬───────┘  │
            │           │          │
            │  ┌────────▼───────┐  │
            │  │  Task Storage  │  │
            │  │  (In-Memory    │  │
            │  │   + Files)     │  │
            │  └────────────────┘  │
            └──────────────────────┘
                        │
                        │ File I/O
                        ▼
            ┌──────────────────────┐
            │   Filesystem         │
            │   tasks/*.json       │
            └──────────────────────┘
```

## Component Architecture

### Crate Organization

The system uses a Cargo workspace with four crates, each with a specific responsibility:

```
task-manager/
├── task-types/          # Shared type definitions and RPC service
│   └── src/
│       └── lib.rs       # Task, Status, RPC trait definition
│
├── task-service/        # Pure business logic, transport-agnostic
│   └── src/
│       └── lib.rs       # TaskStorage, domain operations
│
├── task-server/         # Server process with dual interfaces
│   └── src/
│       ├── main.rs      # Server initialization
│       ├── ipc_handler.rs    # Unix Socket RPC server
│       └── http_handler.rs   # HTTP/Leptos endpoints
│
└── task-cli/            # CLI tool for AI assistant
    └── src/
        └── main.rs      # Command parsing, RPC client, JSON output
```

### Dependency Graph

```
    task-cli          task-server
         │                 │
         └────┬────────────┘
              │
              ▼
         task-types
              │
              ▼
        task-service
```

All crates depend on `task-types` for shared data structures and the RPC service definition. The server and CLI have no direct dependency on each other, ensuring clean separation.

## Data Flow

### Read Operation (List Tasks)

```
AI Assistant                                          Server
     │                                                   │
     │  1. Invoke: task-cli list                        │
     ├──────────────────────────────────────────────────┤
     │                                                   │
task-cli                                                │
     │  2. Connect to Unix Socket                       │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │  3. Serialize ListTasksRequest                   │
     │     Send binary RPC message                      │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │                                         ipc_handler
     │                                                   │
     │                                  4. Deserialize request
     │                                     Call task_service
     │                                                   │
     │                                         task_service
     │                                                   │
     │                                  5. Read from in-memory
     │                                     cache (TaskStorage)
     │                                                   │
     │                                  6. Return Vec<Task>
     │                                                   │
     │                                         ipc_handler
     │                                                   │
     │  7. Serialize ListTasksResponse                  │
     │     Send binary RPC message                      │
     <───────────────────────────────────────────────────┤
     │                                                   │
task-cli                                                │
     │  8. Deserialize response                         │
     │     Convert to JSON                              │
     │                                                   │
     │  9. Print JSON to stdout                         │
     ├──────────────────────────────────────────────────┤
     │                                                   │
AI Assistant                                            │
     │  10. Parse JSON output                           │
     │                                                   │
```

### Write Operation (Create Task)

```
AI Assistant                                          Server
     │                                                   │
     │  1. Invoke: task-cli add "Do something"          │
     ├──────────────────────────────────────────────────┤
     │                                                   │
task-cli                                                │
     │  2. CreateTaskRequest via RPC                    │
     ├──────────────────────────────────────────────────>
     │                                                   │
     │                                         ipc_handler
     │                                                   │
     │                                  3. Call task_service
     │                                     create_task()
     │                                                   │
     │                                         task_service
     │                                                   │
     │                                  4. Update in-memory
     │                                     cache
     │                                                   │
     │                                  5. Write task file
     │                                     to filesystem
     │                                     (atomic write)
     │                                                   │
     │                                  6. Return new Task
     │                                                   │
     │                                         ipc_handler
     │                                                   │
     │  7. CreateTaskResponse                           │
     <───────────────────────────────────────────────────┤
     │                                                   │
task-cli                                                │
     │  8. Format as JSON                               │
     │                                                   │
AI Assistant                                            │
     │  9. Receive confirmation                         │
     │                                                   │
```

### Browser Access (Concurrent with CLI)

```
Browser                                              Server
   │                                                    │
   │  1. HTTP GET /                                    │
   ├───────────────────────────────────────────────────>
   │                                                    │
   │                                          http_handler
   │                                                    │
   │  2. Serve Leptos app (WASM)                       │
   <────────────────────────────────────────────────────┤
   │                                                    │
   │  3. Leptos Server Function: get_tasks()           │
   ├───────────────────────────────────────────────────>
   │                                                    │
   │                                          http_handler
   │                                                    │
   │                                  4. Call task_service
   │                                     (same as IPC path)
   │                                                    │
   │                                         task_service
   │                                                    │
   │                                  5. Read from same
   │                                     in-memory cache
   │                                                    │
   │  6. Return tasks as JSON/Response                 │
   <────────────────────────────────────────────────────┤
   │                                                    │
   │  7. Leptos renders UI reactively                  │
   │                                                    │
```

## Storage Architecture

### File Layout

```
tasks/
├── task-2024-001.json    # Individual task files
├── task-2024-002.json
├── task-2024-003.json
└── ...
```

Each task exists as a self-contained JSON file with all properties.

### In-Memory Cache Structure

```
┌─────────────────────────────────────┐
│        TaskStorage                  │
├─────────────────────────────────────┤
│                                     │
│  Primary Cache:                     │
│  ┌──────────────────────────────┐   │
│  │ HashMap<TaskId, Task>        │   │
│  │                              │   │
│  │ "task-001" -> Task { ... }   │   │
│  │ "task-002" -> Task { ... }   │   │
│  │ "task-003" -> Task { ... }   │   │
│  └──────────────────────────────┘   │
│                                     │
│  Derived Indexes (built on demand): │
│  ┌──────────────────────────────┐   │
│  │ Status Index                 │   │
│  │   "todo" -> ["task-002"]     │   │
│  │   "in_progress" -> [...]     │   │
│  │   "done" -> ["task-003"]     │   │
│  └──────────────────────────────┘   │
│                                     │
│  ┌──────────────────────────────┐   │
│  │ Context Index                │   │
│  │   "Work" -> [...]            │   │
│  │   "Personal" -> [...]        │   │
│  └──────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

### Cache Consistency

Since only the server process accesses the task storage directly, cache consistency is guaranteed. The CLI never touches files directly.

```
Write Path:
CLI -> RPC -> Server -> update_cache() -> write_file()
                              │
                              └──> Both operations or neither
                                   (write-through cache)

Read Path:
CLI -> RPC -> Server -> read_cache()
                              │
                              └──> Always consistent with disk
                                   (server is single source of truth)
```

## Communication Protocols

### IPC Protocol (CLI ↔ Server)

Transport: Unix Domain Socket at `/tmp/task-server.sock`

Message Format (with tarpc):
```
┌──────────────────────────────────┐
│  tarpc Frame Header              │
│  (managed by tarpc framework)    │
├──────────────────────────────────┤
│  Method ID (which RPC method)    │
├──────────────────────────────────┤
│  Serialized Arguments            │
│  (bincode binary format)         │
│                                  │
│  Example for list_tasks():       │
│  - No arguments                  │
│                                  │
│  Example for create_task():      │
│  - action: String                │
│  - status: TaskStatus            │
└──────────────────────────────────┘

Response:
┌──────────────────────────────────┐
│  tarpc Frame Header              │
├──────────────────────────────────┤
│  Serialized Return Value         │
│  (bincode binary format)         │
│                                  │
│  Example: Vec<Task> or           │
│           Result<Task, Error>    │
└──────────────────────────────────┘
```

### HTTP Protocol (Browser ↔ Server)

Transport: TCP port 3000 (configurable)

Options:
1. **Leptos Server Functions** (preferred for browser UI):
   - Automatic client-server serialization
   - Type-safe function calls from WASM
   - Internal HTTP POST to generated endpoints

2. **Plain REST API** (for maximum flexibility):
   ```
   GET  /api/tasks           -> List all tasks
   POST /api/tasks           -> Create task
   PUT  /api/tasks/:id       -> Update task
   DELETE /api/tasks/:id     -> Delete task
   ```

Both options call the same `task-service` business logic.

## Concurrency Model

### Server Threading

```
┌─────────────────────────────────────────┐
│          task-server Process            │
├─────────────────────────────────────────┤
│                                         │
│  Tokio Runtime (async multi-threaded)   │
│                                         │
│  ┌─────────────────┐  ┌──────────────┐ │
│  │ IPC Listener    │  │ HTTP Listener│ │
│  │ Task            │  │ Task         │ │
│  └────────┬────────┘  └──────┬───────┘ │
│           │                  │         │
│           ▼                  ▼         │
│  ┌──────────────────────────────────┐  │
│  │  Shared TaskStorage              │  │
│  │  (Arc<RwLock<TaskStorage>>)      │  │
│  │                                  │  │
│  │  - Multiple readers OK           │  │
│  │  - Single writer blocks all      │  │
│  └──────────────────────────────────┘  │
│                                         │
└─────────────────────────────────────────┘
```

Read operations (list tasks) acquire read locks and can run concurrently. Write operations (create/update/delete) acquire write locks and run exclusively.

### CLI Tool

```
┌─────────────────────────┐
│   task-cli Process      │
├─────────────────────────┤
│                         │
│  1. Parse args          │
│  2. Connect to socket   │
│  3. Send RPC request    │
│  4. Wait for response   │
│  5. Format as JSON      │
│  6. Print and exit      │
│                         │
│  (Short-lived process)  │
└─────────────────────────┘
```

Each CLI invocation is a separate process that connects, makes one RPC call, and terminates.

## Error Handling Strategy

### RPC Layer Errors

```
Client (CLI)                Server
    │                          │
    │  Request                 │
    ├─────────────────────────>│
    │                          │
    │                    ┌─────┴─────┐
    │                    │ Validate  │
    │                    │ request   │
    │                    └─────┬─────┘
    │                          │
    │                    Valid │ Invalid
    │                          │
    │              ┌───────────┴──────────┐
    │              ▼                      ▼
    │        Process request      Return Error
    │              │                      │
    │              │                      │
    │  Response    │         Error Response
    │<─────────────┘              │
    │<────────────────────────────┘
    │                          │
```

Errors at different layers:
- **Transport errors**: Connection failed, socket not found → CLI exits with error
- **Serialization errors**: Invalid binary data → Server returns error response
- **Business logic errors**: Task not found, validation failed → Result<T, E> in response
- **Storage errors**: File write failed → Server returns error response

### Browser Error Handling

Leptos server functions return `Result<T, ServerFnError>`. The browser UI displays errors inline without crashing the app.

## Deployment Model

### Development

```
Developer Machine
├── Terminal 1: cargo run --bin task-server
├── Terminal 2: cargo run --bin task-cli -- list
└── Browser: http://localhost:3000
```

All processes on localhost. Socket at `/tmp/task-server.sock`.

### Production (Family Server)

```
Family Server (e.g., Raspberry Pi, NUC)
├── systemd unit: task-server.service
│   └── Socket: /var/run/task-server.sock
│   └── HTTP: 0.0.0.0:3000 (or behind nginx)
│
└── task-cli installed in PATH
    └── Family members can SSH and use CLI
    └── AI assistant runs CLI via automation
```

Nginx or similar can provide HTTPS termination and static asset caching for the browser UI.

## Security Considerations

### IPC Security

Unix Domain Socket permissions control access:
```bash
# Socket owned by task-server user
# Group-readable for family group
-rw-rw---- 1 taskserver family /var/run/task-server.sock
```

Only local processes in the `family` group can connect.

### HTTP Security

For local network deployment:
- No authentication (trusted network)
- HTTPS optional (can use self-signed cert)

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
Operation          Latency     Throughput
────────────────────────────────────────────
List tasks (IPC)   < 1ms       10000/sec
List tasks (HTTP)  < 5ms       1000/sec
Create task        < 10ms      100/sec
Update task        < 10ms      100/sec
Server startup     < 100ms     N/A
```

All operations are memory-bound. No database queries, no network calls except IPC/HTTP.

### Bottlenecks

Current architecture has no bottlenecks at family scale. Theoretical limits:
- Write lock contention if >100 concurrent writers
- File I/O if >1000 writes/second
- Memory if >100k tasks loaded

None of these are realistic for the target use case.

## Future Extensions

### Potential Additions

1. **Real-time browser updates**: Server-Sent Events to push changes to browsers
2. **Multi-device sync**: CRDTs or operational transforms for offline editing
3. **Search**: Full-text search index for task descriptions and notes
4. **Attachments**: File uploads associated with tasks
5. **Recurrence**: Repeating tasks with schedules
6. **Collaboration**: Comments and mentions on tasks

Each extension would follow the same pattern: add to business logic in `task-service`, expose via both IPC and HTTP interfaces.

### Migration Path

If family grows beyond target scale:
- Replace file storage with SQLite (still zero-infrastructure)
- Add connection pooling for IPC
- Implement pagination for task lists
- Consider distributed deployment (multiple servers)

The clean separation between business logic and transport makes these migrations tractable.
