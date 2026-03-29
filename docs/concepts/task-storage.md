# Task Storage and Caching Strategy Concept

## Abstract

This concept defines the storage architecture for a family-oriented task management system. The design prioritizes simplicity and zero-infrastructure setup while supporting efficient task board displays and category filtering. The solution uses file-based storage with a complete in-memory cache layer, storing all task properties in single atomic files.

## Context

The system serves small family groups (up to 4 users) and must be deployable without database infrastructure. Tasks need to appear on task boards, support category filtering, and contain the core properties: action summary, due date, start date, time estimate, priority, context/category, and notes.

## Storage Strategy

The persistence layer uses plain JSON files on the filesystem. Each task exists as a single, self-contained JSON document within a flat directory: `tasks/task-{uuid}.json`. Each file contains all task properties as one atomic unit. This approach eliminates the need for database setup while remaining human-readable and version-control friendly.

The flat structure works well for the expected scale (hundreds to low thousands of tasks for a family). While categories could organize tasks into subdirectories, the flat approach simplifies searching and filtering, and prevents issues when task categories change.

## Task Identifiers

Tasks are identified by **UUID v7**. The UUID is encoded in the filename in simple (non-hyphenated) format:

```
tasks/task-019c500fe59875a19bd9286e3d82cd04.json
```

UUID v7 is time-ordered, so files naturally sort by creation time. The ID is not stored inside the JSON file — the filename is the authoritative identifier. At load time the server parses the UUID from the filename and rejects any file whose name does not contain a valid UUID v7.

## Single-File Design Rationale

All task properties are kept in a single file. We explored splitting tasks into a summary file (board-visible fields) and a details file (notes, extended fields), but rejected it for three reasons:

1. **Synchronization complexity.** When a task changes, determining which file(s) to update is non-trivial. Some changes affect both files (e.g. status transitions that also touch timestamps).

2. **Fuzzy boundary.** The distinction between "summary" and "details" is unstable. A field that seems like a detail today (time estimate) might need to appear on a board card tomorrow. Premature splitting locks in a boundary that may change.

3. **Negligible savings.** A complete task with all optional fields occupies roughly 1–3 KB in memory. Even 1000 tasks total under 3 MB — less than a single medium-resolution photograph. The complexity cost of split files vastly outweighs any theoretical memory benefit.

The single-file approach provides atomicity, simplicity, and predictability. Reading a file gives you everything. Writing a file updates all properties in one operation.

## File Format

Each task file is a flat JSON object. The task ID is the filename, not a field inside the document.

```json
{
  "summary": "Draft presentation outline for Project X",
  "status": { "ToDo": { "since": "2024-03-01T10:00:00Z" } },
  "priority": "A",
  "due_date": { "Precise": "2024-03-15T00:00:00Z" },
  "start_date": { "Guess": "next week" },
  "time_estimate": { "Guess": "1 hour" },
  "context": "Work/ProjectX",
  "notes": "Include sections on architecture and timeline"
}
```

### Field Reference

| Field           | Type                           | Required | Notes                                  |
| --------------- | ------------------------------ | -------- | -------------------------------------- |
| `summary`       | string                         | yes      | Human-readable task description        |
| `status`        | `ToDo`/`Done` + `since`        | yes      | Timestamp records when status was set  |
| `priority`      | `"A"` / `"B"` / `"C"`          | no       | Defaults to `C`                        |
| `due_date`      | `Precise` or `Guess`           | no       | Accepts RFC3339 datetime or free text  |
| `start_date`    | `Precise` or `Guess`           | no       | Earliest date to start working on task |
| `time_estimate` | `Precise` (seconds) or `Guess` | no       | Accepts duration or free text          |
| `context`       | string                         | no       | Category / project label               |
| `notes`         | string                         | no       | Free-form text                         |

### Status Encoding

`status` is an object that captures both the current state and _when it was set_, e.g. `{"ToDo": {"since": "2024-03-01T10:00:00Z"}}` or `{"Done": {"since": "2024-03-10T18:30:00Z"}}`.

Only two states exist: `ToDo` and `Done`. There is no "in-progress" state. The `since` field serves as both a creation timestamp (when status is `ToDo` and the task was just created) and a completion timestamp (when status is `Done`).

### Date and Time Fields

Both `due_date` and `start_date` use a two-variant encoding to accommodate imprecise user input:

- `{"Precise": "<RFC3339 datetime>"}` — machine-readable, enables sorting and filtering
- `{"Guess": "<free text>"}` — human-entered string like `"next week"` or `"sometime in April"`

`time_estimate` follows the same pattern:

- `{"Precise": <seconds as integer>}` — machine-readable
- `{"Guess": "<free text>"}` — e.g. `"about an hour"`

### Legacy Format Compatibility

Earlier versions stored `status` as a plain string (`"ToDo"`, `"Done"`). On load, the server detects this legacy format and re-serializes the file using the current format (with embedded `since` timestamp). The file is marked dirty so it will be rewritten on the next flush.

## Caching Architecture

The system maintains a complete in-memory cache of all tasks. At startup it reads every `task-*.json` file from the tasks directory into an `IndexMap<Uuid, Task>` (insertion-ordered, using `ahash` for fast hashing). This happens once before the server accepts requests.

There are **no derived indexes** (no status index, no context index). Reads iterate the `IndexMap` directly. For family-scale data (≤1000 tasks) a linear scan completes in microseconds and is simpler to maintain than a set of indexes that must be kept consistent on every write.

### Dirty Tracking

Mutations are tracked through a companion `IndexSet<Uuid>` called the _change set_ (referred to as `dirty` in the code). Rather than writing to disk on every change (write-through), the cache defers I/O:

```
Mutation path:
  add / get_mut / remove → marks UUID in dirty set

Flush path (background task, every 60 s):
  for each UUID in dirty:
    if task exists  → write_task_file (atomic rename)
    if task deleted → delete_task_file
  clear dirty set on full success
  on partial failure: retain only failed UUIDs in dirty set
```

The deferred approach batches I/O, reduces disk pressure, and keeps mutation latency below 1 ms even on slow storage.

### TaskMutGuard

`get_mut()` returns a `TaskMutGuard` instead of a plain `&mut Task`. The guard holds a mutable reference to the task and a reference to the dirty set. When the guard is dropped (end of scope), it automatically inserts the UUID into the dirty set. This makes it impossible to mutate a task without recording the change — no call site needs to remember to mark dirty manually.

### Shared Cache

The cache is wrapped in `Arc<RwLock<TaskCache>>` (`SharedTaskCache`) and shared between:

- The **RPC handler** (CLI access)
- The **HTTP handler** (browser/Leptos server functions)
- The **background flush task**

Multiple readers can hold the read lock concurrently. Writers acquire an exclusive lock for the duration of the mutation (typically microseconds).

## Atomic Writes

Each task file is written atomically:

1. Serialize task to `task-{uuid}.json.tmp`
2. `rename()` the temp file over the final path

The `rename()` syscall is atomic on POSIX filesystems. If the process crashes mid-write, only the temp file is left behind — the previous version of the task file remains intact. The temp file is ignored at startup (only `task-*.json` files are loaded).

## Flush on Shutdown

On SIGTERM or SIGINT the server performs a `final_flush()` before exiting, draining the dirty set synchronously. This ensures no data is lost for in-flight mutations that have not yet reached the background flush interval.

## Concurrency

`SharedTaskCache` uses `tokio::sync::RwLock`. All mutation operations acquire the write lock, serializing concurrent writes. This is appropriate for the target user count (≤4 users). Concurrent reads are fully parallel. There is no optimistic locking or file modification timestamp checking — the in-memory cache is the single source of truth, and the server is the sole writer to the filesystem.

## Initial Load and Error Handling

At startup the server calls `TaskCache::load()`, which:

1. Creates the `tasks/` directory if it does not exist.
2. Scans the directory for `task-*.json` files.
3. Parses the UUID from each filename; skips files with invalid or non-v7 UUIDs.
4. Reads and deserializes each file.
5. Detects and migrates legacy format files (marks them dirty for rewrite).
6. Returns `(num_loaded, num_to_migrate)` on success, or `LoadErrors` listing every failed file.

Load failures are non-fatal per file: the server can start with a partial cache and log the errors.

## Scalability Considerations

For a family managing 1000 tasks, startup takes under a second and memory usage stays under 10 MB. These numbers leave comfortable headroom.

Natural limits:

- Loading thousands of small files at startup becomes slow beyond ~50k tasks.
- The `IndexMap` grows linearly with task count.

For significantly larger deployments, SQLite would be a natural replacement for the file layer while keeping the same in-memory `IndexMap` cache structure.

## Backup and Recovery

Plain JSON files integrate naturally with standard backup tools. Copying the `tasks/` directory is sufficient for a full backup. Git works well for tracking task history if desired.

Recovery is trivial: restore the `tasks/` directory. No database restoration or migration steps are needed.
