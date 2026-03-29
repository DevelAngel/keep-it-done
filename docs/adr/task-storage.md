# ADR: File-Based Task Storage with Complete In-Memory Caching

## Status

Accepted

## Context

We need a persistence strategy for task cards in a family task management system. The system displays tasks on Kanban boards and supports category filtering. The target audience is families with up to four users. A key requirement is simple deployment without database infrastructure.

Tasks contain multiple properties: summary, due date, start date, time estimate, priority, context, status, and notes. The system must efficiently support queries like "show all open tasks" or "show all tasks in the 'Work' category."

## Decision

We will store tasks as individual JSON files in a flat directory structure and maintain a complete in-memory cache of all tasks with all properties and derived indexes for common queries.

Storage implementation details:

- Each task exists in one file: `tasks/task-{uuid}.json` (UUID v7, simple encoding)
- The task ID is the filename — it is not stored inside the JSON document
- All task properties live in a single JSON document
- No splitting of task data across multiple files
- Flat directory structure instead of category-based subdirectories

Cache implementation details:

- Full task dataset including all properties loaded into memory at startup
- In-memory `IndexMap<Uuid, Task>` (insertion-ordered, fast `ahash` hashing)
- No derived indexes — reads iterate the map directly (fast enough at family scale)
- Deferred flush: mutations mark UUIDs in a dirty set; a background task flushes every 60 s
- Final flush on graceful shutdown to prevent data loss
- Atomic file writes using temp file + rename pattern

## Consequences

### Positive

The deployment story becomes extremely simple. Users can run the application anywhere without installing databases. The entire task dataset fits in a single directory that any backup tool understands.

Read performance is excellent. Displaying Kanban boards or filtering by category requires no disk I/O. The UI responds instantly because all queries hit memory. Opening task details for editing or viewing notes happens with zero latency since all data is already loaded.

Development complexity stays low. No ORM, no migration scripts, no database connection pooling. Reading and writing JSON files is straightforward in any programming language. The single-file-per-task design eliminates synchronization concerns between related data.

Data remains human-readable and debuggable. Users can inspect task files directly, edit them in emergencies, or process them with standard text tools.

Version control integration works naturally. Teams or families who want task history can simply commit the tasks directory to Git.

Atomic operations are straightforward. Each task file update is a single atomic rename operation, preventing partial writes or corrupted states.

### Negative

Startup time depends on task count. Reading hundreds of small files takes measurable time. For the target scale (families), this remains under a second, but the architecture would not scale to thousands of users.

Memory usage grows with task count. Each task consumes a few kilobytes in memory. Again, acceptable for families but limiting for larger deployments. At family scale (100-200 active tasks), total memory consumption stays well under 1 MB, which is negligible.

Concurrent writes from multiple application instances are not supported. The single server process serializes all writes via `Arc<RwLock<TaskCache>>`. Running multiple instances would cause conflicting writes to the filesystem without coordination.

Search performance will degrade as task count grows if we implement full-text search later. The in-memory approach enables only simple indexed lookups. Complex queries might require scanning all tasks.

No built-in query language exists. Advanced filtering beyond status and category requires custom code. A database would provide SQL or similar query capabilities.

### Mitigations

To address startup time, we can implement lazy loading if needed. Load only active tasks initially and fetch archived tasks on demand.

For memory concerns, we can add task archiving. Move completed tasks older than 90 days to an archive directory that does not load at startup.

If multiple instances become necessary, a coordination layer (e.g. a shared lock file, or switching to SQLite with WAL mode) would be required. This adds complexity but remains simpler than full database deployment.

For search, we can add a separate search index file that builds incrementally. This trades some simplicity for search speed if the feature becomes important.

## Alternatives Considered

### SQLite database

Would provide proper querying and transaction support. Rejected because it adds deployment complexity (file permissions, location management) and requires migration tooling for schema changes. The query power is not needed for simple Kanban and category filtering.

### Single JSON file with all tasks

Simpler to load but problematic for concurrent access. Two users updating different tasks would conflict. File corruption risk increases because every write touches the entire dataset. Rejected due to fragility.

### Category-based subdirectories

Structure like `tasks/{category}/{task-id}.json` would make browsing by category easier but complicates category changes. Moving files between directories is more complex than updating a property. Rejected because filtering in-memory is already fast.

### Split-file approach with summary and details

We considered storing tasks as two files per task: `tasks/{task-id}.summary.json` containing board-relevant properties (action, status, priority, due date) and `tasks/{task-id}.details.json` containing supplementary information (notes, detailed dependencies, timestamps). The cache would load all summaries but only recently accessed details in a ring buffer.

This approach was rejected for several reasons. First, synchronization between two files becomes complex when updates can affect both files. Second, the boundary between summary and details is not clear-cut. Dependencies seem like details but are needed for detecting unblocked tasks. Time estimates might display on board cards, making them summary data.

Third, and most critically, the memory savings are negligible. A complete task with all properties occupies two to three kilobytes. Even a thousand tasks consume only two to three megabytes. For a family with a hundred tasks, we are discussing 200 to 300 kilobytes, which is meaningless on modern hardware. The complexity cost of managing two files, implementing lazy loading, and handling cache misses far outweighs any theoretical benefit.

Fourth, startup performance would likely degrade rather than improve. Reading 200 small files (two per task) introduces more filesystem overhead than reading 100 slightly larger files.

### Separate notes files

We considered extracting notes into separate files since note length varies significantly. Some tasks have no notes, others have several sentences or pasted messages. However, typical notes contain 200 to 400 characters, roughly 0.5 to 1 kilobyte in JSON.

Even if every task has maximum-length notes, a hundred tasks contribute only 100 kilobytes to the cache. This is not a meaningful amount of data. The complexity of managing separate note files, handling lazy loading when users open task details, and synchronizing note updates with task updates does not justify the negligible memory savings.

Additionally, separating notes would introduce latency when users open task details. All data in memory means instant response. Requiring a file read for notes creates a small but perceptible delay and necessitates loading indicators in the UI.

### Key-value store (Valkey/Redis)

Explicitly ruled out by requirements. Would provide excellent performance but requires infrastructure setup.

### Cloud storage (Firebase, Supabase)

Provides synchronization across devices but requires internet connectivity and account setup. Conflicts with the simple deployment goal. Suitable for a future enhancement but not the base implementation.

## Implementation Notes

The single-file design with complete caching prioritizes simplicity and performance at the target scale. For family usage with compact task records, loading everything into memory eliminates complexity and provides instant UI response with negligible resource cost.

If the system later needs to support significantly larger deployments or tasks with substantial attachments, the architecture can be revisited. But for the defined use case, optimizing for problems that cannot occur at this scale would be premature optimization with real complexity costs.
