# ADR: CLI API Design with JSON Schema

## Status

Accepted

## Context

AI assistant (e.g., clawdbot) needs programmatic interface to task management system. Requirements:

- Minimal token cost per operation
- Type-safe data exchange
- Self-documenting API
- No human usability needed (AI-only interface)
- Support for optional task properties (due_date, priority, time_estimate, context, notes, etc.)

Task struct has many optional fields. Traditional CLI flag explosion (--priority, --due-date, --context, etc.) increases token cost and maintenance burden.

## Decision

### Command Structure

Six operations with distinct intents, all using named flags:

```bash
kid add    --summary <text> [--details <JSON>]     # Create task
kid complete --id <uuid> [--reopen]                # Mark done (or reopen)
kid rename --id <uuid> --summary <text>            # Change task identity
kid replace  --id <uuid> --details <JSON>          # Replace all details
kid update   --id <uuid> --details <JSON>          # Patch some details
kid list     [--json] [--pretty]                   # Output all tasks
kid schema   [--pretty] [--out <file>]             # Get JSON schema for details
```

Server address is configured via `--server` flag or `KID_SERVER_ADDR` env var (default `127.0.0.1:9000`).

### JSON for Optional Fields

`add`, `replace`, and `update` accept JSON for detail fields:

```bash
kid add --summary "Fix leak" --details '{"priority":"A","due_date":{"Guess":"next week"}}'
kid update --id <uuid> --details '{"priority":"B","notes":"Rescheduled"}'
```

Omitting `--details` in `add` uses an empty `Details` (all optional fields absent).

### Command Separation Rationale

**Complete:** Most frequent operation. Dedicated command with `--reopen` flag covers both directions of the `ToDo`/`Done` toggle without JSON overhead.

**Rename:** Summary is task identity. Explicit command prevents accidental changes via `update`. AI must consciously choose rename vs update.

**Replace vs Update:** `replace` sets all detail fields to exactly the provided JSON (missing fields become `None`). `update` patches only the provided fields; omitted fields are left unchanged. Both operate on `Details`; neither can touch `summary` or `status`.

### Schema Provision

```bash
kid schema            # Outputs full JSON Schema for the Details struct
kid schema --pretty   # Pretty-printed
kid schema --out schema.json  # Write to file
```

The schema is generated at runtime from the `Details` struct using `schemars`. It covers all fields accepted by `add`, `replace`, and `update`. AI queries it once and caches in context.

## Consequences

### Positive

**Token efficiency:** Status change `kid complete --id <uuid>` vs `kid update --id <uuid> --details '{"status":"Done"}'` saves ~40% tokens on the most common operation.

**Type safety:** JSON schema validates AI-generated payloads. Catch errors before execution.

**Self-documenting:** Schema command provides complete API reference. No external docs needed.

**Extensibility:** Adding task fields = schema update only. No new CLI flags.

**Intent clarity:** Four update types (complete/rename/replace/update) map to distinct user intents. AI learns when to use each.

**Summary protection:** Explicit rename command prevents accidental identity changes. Adds cognitive friction by design.

### Negative

**JSON escaping:** Shell requires quoting JSON strings. Minor friction for AI text generation.

**Two-part add command:** Summary separate from JSON. AI must structure both parts correctly.

**Schema complexity:** DateEstimation/TimeEstimation use Guess/Precise wrappers. Adds nesting depth.

**Single schema:** `kid schema` outputs one schema covering `add`, `replace`, and `update`. There is no per-command schema. AI must know that `complete` and `rename` use their own flags, not JSON details.

### Mitigations

**Escaping:** AI trained on shell syntax. Single-quote JSON strings natural in LLM output.

**Add structure:** Example in schema output demonstrates correct format. AI follows pattern.

**Schema complexity:** Guess variant accepts freeform strings. AI defaults to Guess for fuzzy data. Precise only when exact values available.

**Single schema:** One schema output covers `add`, `replace`, and `update`. `complete` and `rename` have no JSON input — their intent is expressed entirely through their flags.

## Alternatives Considered

### CLI flags for all fields

```bash
kid add "Fix leak" --priority A --due-date "next week"
```

Rejected: Flag proliferation as task fields grow. Higher token cost (--flag per field). Less extensible.

### Single update command

```bash
kid update <uuid> '{"status":"Done","summary":"New name","priority":"A"}'
```

Rejected: No intent separation. Status changes (frequent) same cost as metadata changes. Summary unprotected from accidental modification.

### YAML/TOML for optional fields

```bash
kid add "Fix leak" --yaml 'priority: A\ndue_date: next week'
```

Rejected: Multiline strings in shell arguments awkward. TOML verbose. JSON ubiquitous in AI training data.

### Custom DSL

```bash
kid add "Fix leak @A due:next-week ctx:Kitchen"
```

Rejected: Custom syntax requires prompt explanation. Token cost of teaching DSL > savings from compact format. JSON natural for AI.

### Positional optional arguments

```bash
kid add "Fix leak" A "next week" "Kitchen"
```

Rejected: Ambiguous field order. What if priority omitted but context provided? Requires sentinel values (null, -, empty).

## Implementation Notes

Clap structure (simplified):

```rust
#[derive(Subcommand)]
enum Commands {
    Schema { pretty: bool, outfile: Option<PathBuf> },
    List   { json: bool, pretty: bool, server: ServerArgs },
    Add    { summary: String, details: Option<String>, server: ServerArgs },
    Rename { id: Uuid, summary: String, server: ServerArgs },
    Replace{ id: Uuid, details: String, server: ServerArgs },
    Update { id: Uuid, details: String, server: ServerArgs },
    Complete{ id: Uuid, reopen: bool, server: ServerArgs },
}
```

All mutating commands use named flags (`--id`, `--summary`, `--details`, `--reopen`). `update` accepts a `TaskDetailsPatch` (partial — only present fields are changed). `replace` accepts a full `TaskDetails` (absent fields become `None`).

Schema generation uses `schemars` on the `Details` type, producing a standard JSON Schema. No hand-crafted schema; the schema evolves automatically when the struct changes.

## Token Cost Analysis

Typical task creation with 3 optional fields:

**This design:**

```bash
kid add --summary "Fix leak" --details '{"priority":"A","due_date":{"Guess":"next week"},"context":"Kitchen"}'
```

≈ 24 tokens

**Flag-based alternative:**

```bash
kid add --summary "Fix leak" --priority A --due-date "next week" --context "Kitchen"
```

≈ 30 tokens

**~20% token reduction** per creation at scale.

Complete (most frequent):

- This design: `kid complete --id <uuid>` ≈ 8 tokens
- Unified update approach: `kid update --id <uuid> --details '{"status":"Done"}'` ≈ 14 tokens
- **~43% reduction**

Over 1000 operations: ~6000 token savings.
