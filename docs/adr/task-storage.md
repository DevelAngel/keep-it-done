---
status: accepted
date: 2026-03-29
---

# File-Based Task Storage with Complete In-Memory Caching

## Context and Problem Statement

A family task management system needs a persistence strategy for task cards. The target is 2–4 users managing up to a few hundred tasks. How should tasks be stored and served to meet instant read performance, simple deployment, and data integrity requirements — without database infrastructure?

## Decision Drivers

- Zero-infrastructure deployment — no database installation required
- Instant read performance for task list display
- Human-readable, debuggable data format
- Data integrity under concurrent access from a single process
- Simple backup and recovery story

## Considered Options

- Individual JSON files with complete in-memory cache
- SQLite database
- Single JSON file containing all tasks
- Category-based subdirectory structure
- Split-file approach (summary + details per task)
- Key-value store (Valkey/Redis)
- Cloud storage (Firebase, Supabase)

## Decision Outcome

Chosen option: "Individual JSON files with complete in-memory cache", because it provides instant reads (no disk I/O after startup), eliminates database infrastructure, and keeps task data in a human-readable format that any backup tool understands.

Storage layout:

- One file per task: `tasks/task-{uuid-v7}.json`
- The task ID is the filename — not stored inside the JSON
- Flat directory — no category subdirectories

Cache implementation:

- Full dataset loaded into `IndexMap<Uuid, Task>` at startup (insertion-ordered, `ahash`)
- No derived indexes — direct iteration is fast enough at family scale
- Writes mark UUIDs in a dirty `IndexSet`; a background task flushes every 60 s
- Final flush on graceful shutdown
- Atomic writes via temp file + rename

### Consequences

- Good, because deployment requires only a directory — no setup, no migrations
- Good, because all reads are in-memory — instant response, no I/O on hot path
- Good, because one-file-per-task makes individual writes atomic and isolated
- Good, because task files are human-readable JSON — inspectable, editable, versionable with Git
- Good, because deferred flush decouples write latency from disk I/O
- Bad, because startup time grows with task count (reading many small files) — acceptable at family scale, not for large deployments
- Bad, because memory usage grows with task count — negligible at family scale (~1–3 KB per task)
- Bad, because concurrent writes from multiple application instances are not supported — the single server process owns all writes via `Arc<RwLock<TaskCache>>`

## Pros and Cons of the Options

### Individual JSON files with complete in-memory cache

- Good, because zero infrastructure — copy binary and run
- Good, because each task file is independently atomic
- Good, because human-readable and Git-friendly
- Neutral, because startup reads all files once — O(n) in task count, fast at family scale
- Bad, because not suitable for large deployments or full-text search at scale

### SQLite database

- Good, because proper query language and transaction support
- Good, because single-file database is still simple to back up
- Bad, because requires migration tooling for schema changes
- Bad, because no advantage over file-per-task for the queries actually needed (list all, filter by category)

### Single JSON file containing all tasks

- Good, because trivial to load
- Bad, because concurrent writes from two users conflict on the same file
- Bad, because any partial write corrupts the entire dataset

### Category-based subdirectory structure

- Neutral, because makes browsing by category possible without the app
- Bad, because moving a task to a different category requires a filesystem rename
- Bad, because in-memory filtering is already instant — no benefit at family scale

### Split-file approach (summary + details per task)

- Neutral, because reduces per-read data if details are rarely needed
- Bad, because synchronization between two files is complex when updates can affect both
- Bad, because the boundary between summary and details is ambiguous
- Bad, because memory savings are negligible — a full task is 2–3 KB; 200 tasks = 400–600 KB

### Key-value store (Valkey/Redis)

- Good, because excellent read/write performance
- Bad, because requires infrastructure setup — explicitly out of scope

### Cloud storage (Firebase, Supabase)

- Good, because built-in multi-device sync
- Bad, because requires internet connectivity and cloud account
- Bad, because conflicts with the privacy-first, local-only deployment goal

## More Information

The single-server-process constraint means `Arc<RwLock<TaskCache>>` is the only concurrency mechanism needed. All RPC and HTTP handlers share the same cache instance. If multiple instances become necessary later, a coordination layer (shared lock file or SQLite WAL mode) would be required.

The deferred flush strategy accepts a window of potential data loss (up to 60 s) in exchange for decoupled write latency. The final flush on graceful shutdown closes this window for normal shutdowns. Abnormal termination (kill -9, power loss) can lose up to 60 s of mutations — acceptable for a family task list.

See `kid-types/src/server.rs` for `TaskCache` and `TaskMutGuard`, and `kid-server/src/cache.rs` for the background flush loop.
