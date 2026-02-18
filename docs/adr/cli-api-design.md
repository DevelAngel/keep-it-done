# ADR: CLI API Design with JSON Schema

## Status

Proposed

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

Four operations with distinct intents:

```bash
kid add <summary> [JSON]           # Create task
kid status <uuid> <status>         # Change ToDo/Done
kid rename <uuid> <new-summary>    # Change task identity
kid update <uuid> <JSON>           # Change metadata
kid list                           # Output all tasks as JSON
kid schema <command>               # Get JSON schema for command
```

### JSON for Optional Fields

`add` and `update` accept JSON for optional fields:

```bash
kid add "Fix leak" '{"priority":"A","due_date":{"Guess":"next week"}}'
kid update a1b2c3 '{"priority":"B","notes":"Rescheduled"}'
```

Empty JSON valid:
```bash
kid add "Quick task"  # defaults to '{}'
```

### Command Separation Rationale

**Status:** Most frequent operation. Dedicated command eliminates JSON overhead.

**Rename:** Summary is task identity. Explicit command prevents accidental changes. AI must consciously choose rename vs update.

**Update:** All other metadata changes. Excludes summary/status (enforced by validation).

### Schema Provision

```bash
kid schema add      # Returns schema for add JSON
kid schema update   # Returns schema for update JSON (same as add)
kid schema status   # Returns enum: ToDo | Done
kid schema rename   # Returns: string (new summary)
```

Schema output format:
```json
{
  "optional_fields": {
    "priority": "A | B | C",
    "due_date": {
      "Guess": "string",
      "Precise": "ISO8601 datetime"
    },
    "time_estimate": {
      "Guess": "string",
      "Precise": "ISO8601 duration (PT2H)"
    },
    "context": "string",
    "notes": "string"
  },
  "example": "kid add \"Task\" '{\"priority\":\"A\"}'"
}
```

AI queries schema once, caches in context.

## Consequences

### Positive

**Token efficiency:** Status change `kid status <uuid> Done` vs `kid update <uuid> '{"status":"Done"}'` saves 33% tokens on most common operation.

**Type safety:** JSON schema validates AI-generated payloads. Catch errors before execution.

**Self-documenting:** Schema command provides complete API reference. No external docs needed.

**Extensibility:** Adding task fields = schema update only. No new CLI flags.

**Intent clarity:** Three update types (status/rename/update) map to distinct user intents. AI learns when to use each.

**Summary protection:** Explicit rename command prevents accidental identity changes. Adds cognitive friction by design.

### Negative

**JSON escaping:** Shell requires quoting JSON strings. Minor friction for AI text generation.

**Two-part add command:** Summary separate from JSON. AI must structure both parts correctly.

**Schema complexity:** DateEstimation/TimeEstimation use Guess/Precise wrappers. Adds nesting depth.

**Multiple schemas:** Four schema commands vs single unified schema. AI must track which schema applies to which command.

### Mitigations

**Escaping:** AI trained on shell syntax. Single-quote JSON strings natural in LLM output.

**Add structure:** Example in schema output demonstrates correct format. AI follows pattern.

**Schema complexity:** Guess variant accepts freeform strings. AI defaults to Guess for fuzzy data. Precise only when exact values available.

**Multiple schemas:** Schemas share structure (add == update fields). AI learns once, applies twice. Status/rename trivial (enum/string).

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

Clap structure:
```rust
#[derive(Parser)]
enum Command {
    Add {
        summary: String,
        #[arg(default_value = "{}")]
        json: String,
    },
    Status {
        uuid: Uuid,
        status: Status,
    },
    Rename {
        uuid: Uuid,
        summary: String,
    },
    Update {
        uuid: Uuid,
        json: String,
    },
    List,
    Schema {
        #[arg(value_enum)]
        command: SchemaCommand,
    },
}
```

Update validation:
```rust
fn handle_update(uuid: Uuid, json: String) {
    let input: UpdateInput = serde_json::from_str(&json)?;
    
    if input.summary.is_some() {
        return Err("Use 'kid rename' to change summary");
    }
    if input.status.is_some() {
        return Err("Use 'kid status' to change status");
    }
    
    // proceed
}
```

Schema generation uses hand-crafted JSON for AI optimization (not full JSON Schema spec). Includes examples.

## Token Cost Analysis

Typical task creation with 3 optional fields:

**This design:**
```bash
kid add "Fix leak" '{"priority":"A","due_date":{"Guess":"next week"},"context":"Kitchen"}'
```
≈ 22 tokens

**Flag-based alternative:**
```bash
kid add "Fix leak" --priority A --due-date "next week" --context "Kitchen"
```
≈ 28 tokens

**27% token reduction** per operation at scale.

Status change (most frequent):
- This design: 8 tokens
- Unified update: 12 tokens
- **33% reduction**

Over 1000 operations: ~6000 token savings.
