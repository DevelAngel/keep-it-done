# Task Storage and Caching Strategy Concept

## Abstract

This concept defines the storage architecture for a family-oriented task management system. The design prioritizes simplicity and zero-infrastructure setup while supporting efficient Kanban board displays and category filtering. The solution uses file-based storage with a complete in-memory cache layer, storing all task properties in single atomic files.

## Context

The system serves small family groups (up to 4 users) and must be deployable without database infrastructure. Tasks need to appear on Kanban boards, support category filtering, and contain the properties defined in the task card concept: action description, due date, time estimate, priority, context/category, dependencies, and notes.

## Storage Strategy

The persistence layer uses plain JSON files on the filesystem. Each task exists as a single, self-contained JSON document within a directory structure. This approach eliminates the need for database setup while remaining human-readable and version-control friendly.

Tasks are stored in a flat directory structure: `tasks/{task-id}.json`. Each file contains all task properties as one atomic unit. This design keeps related information together, avoiding the complexity of splitting task data across multiple files.

The flat structure works well for the expected scale (hundreds to low thousands of tasks for a family). While categories could organize tasks into subdirectories, the flat approach simplifies searching, filtering, and prevents issues when task categories change.

## Single-File Design Rationale

The decision to keep all task properties in a single file deserves careful consideration, as we explored several alternative approaches during design.

We considered splitting tasks into two files: a summary file with board-relevant information (action, status, priority, due date) and a details file with supplementary information (notes, detailed dependencies, timestamps). The theory was that the board view only needs summary data, so loading only summaries would reduce memory usage and improve startup time.

However, this approach introduces several problems. First, maintaining synchronization between two files becomes complex. When a user edits a task, you must determine which file to update, and some changes might affect both files. The modified timestamp, for instance, logically belongs to both files but can only be stored once without risking inconsistency.

Second, the boundary between summary and details proves surprisingly fuzzy. Dependencies seem like details at first, but if you want to highlight tasks that become unblocked when another task completes, you need dependency information in the cache. Time estimates might display as badges on board cards, making them summary data rather than details.

Third, the memory savings are negligible at family scale. A complete task with all properties occupies roughly two to three kilobytes in memory. Even a thousand tasks would consume only two to three megabytes, less than a single medium-resolution photograph. For a family with a hundred active tasks, we are discussing 200 to 300 kilobytes, which is essentially free on modern hardware.

Fourth, two files per task means double the file operations at startup. Instead of reading 100 files, the system reads 200. Modern filesystems handle small files well, but each file access has overhead. The split approach could actually slow down startup while trying to optimize memory usage.

We also considered extracting notes into separate files, as notes can vary significantly in size. Some tasks have no notes, others have several sentences, and occasionally users might paste longer messages. At three to five sentences, a typical note contains 200 to 400 characters, which translates to roughly 0.5 to 1 kilobyte in JSON format.

Even if every task has maximum-length notes, a hundred tasks would contribute only 100 kilobytes to the cache. This is not a meaningful amount of data on any system capable of running a web application. The complexity cost of managing separate note files, handling synchronization, and implementing lazy loading far outweighs any theoretical benefit.

The single-file approach provides atomicity, simplicity, and performance. When you read a task file, you get everything. When you write a task file, one atomic operation updates all properties. There are no partially loaded tasks, no synchronization concerns between related files, and no complex cache invalidation logic.

## File Format

Each task file follows this structure:

```json
{
  "id": "task-2024-001",
  "action": "Draft presentation outline for Project X",
  "dueDate": "2024-03-15",
  "dueDateType": "hard",
  "timeEstimate": "1 hour",
  "priority": "A",
  "context": "Work/ProjectX",
  "status": "todo",
  "dependencies": ["task-2024-000"],
  "notes": "Include sections on architecture and timeline",
  "createdAt": "2024-03-01T10:00:00Z",
  "modifiedAt": "2024-03-05T14:30:00Z"
}
```

The status property supports Kanban columns like "todo", "in-progress", "done". The context property enables category filtering. Timestamps help with synchronization and conflict detection.

## Caching Architecture

Given the small user count and modest data volumes, the system maintains a complete in-memory cache of all tasks with all their properties. When the application starts, it reads all task files into memory in a single operation. This happens once at startup, taking negligible time for family-scale data.

The cache structure is a simple map from task ID to task object. For efficient Kanban display and filtering, the system builds indexes from this map:

- Status index: groups tasks by their Kanban column
- Context index: groups tasks by category
- Dependency graph: tracks which tasks block others

These indexes are derived data, rebuilt whenever the task map changes. Building them is fast because the total task count remains small.

We considered implementing a ring buffer cache for task details, loading full task data only when users open specific tasks. This pattern works well for systems with large per-record data or thousands of concurrent users. However, for family-scale usage with compact task records, it adds complexity without meaningful benefit. The entire task dataset fits comfortably in memory, and loading everything at startup eliminates any latency when users interact with tasks.

## Read and Write Operations

Read operations (displaying the Kanban board, filtering by category) work entirely from memory. No disk access occurs, making the UI instantly responsive.

Write operations (creating, updating, deleting tasks) follow a write-through pattern. The system updates the in-memory cache immediately and writes the changed task file to disk. This ensures the cache and filesystem stay synchronized while keeping the UI responsive.

For write operations, the system uses atomic file writes: write to a temporary file, then rename it over the original. This prevents corruption if the process crashes mid-write. The rename operation is atomic on most filesystems.

## Handling Concurrent Access

With only four users, true concurrent writes to the same task are extremely rare. The system uses file modification timestamps to detect conflicts. When saving a task, it checks whether the file on disk has a newer timestamp than the cached version. If so, the system rejects the write and asks the user to refresh and retry.

This optimistic concurrency approach works well for low-contention scenarios and avoids the complexity of file locking.

## Initial Load and Cache Invalidation

At startup, the application scans the tasks directory and loads all JSON files. It builds the in-memory map and indexes. This happens before the UI becomes available, so users always see complete data.

The system does not need cache invalidation because it typically runs as a single instance. If deploying with multiple application instances (unusual for a family system), a file watcher can detect external changes and reload affected tasks.

## Scalability Considerations

This architecture serves families well but has natural limits. Reading thousands of small files at startup becomes slow. The in-memory cache grows with task count. These constraints are acceptable for the target use case but would need revision for larger deployments.

For a family managing 1000 tasks (an extreme case), startup takes under a second, and memory usage stays under 10 MB. These numbers leave comfortable headroom.

## Backup and Recovery

Plain JSON files integrate naturally with standard backup tools. Users can simply copy the tasks directory. Version control systems like Git work well for tracking task history if desired.

Recovery is trivial: restore the tasks directory from backup. No database restoration or migration steps are needed.
