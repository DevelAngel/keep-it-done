---
status: accepted
date: 2026-03-29
---

# CLI API Design with JSON Schema

## Context and Problem Statement

An AI assistant needs a programmatic interface to the task management system. The CLI must support all task operations while minimizing token cost per call. Tasks have many optional fields (due date, priority, time estimate, context, notes, …). How should the command structure and argument encoding be designed for an AI consumer?

## Decision Drivers

- Minimal token cost per operation — AI calls this interface hundreds of times
- Type-safe data exchange between AI and server
- Self-documenting API — AI should be able to discover valid inputs
- Extensible without new CLI flags when task fields are added
- Clear separation of distinct user intents

## Considered Options

- Dedicated commands with JSON for optional fields (current design)
- CLI flags for all fields
- Single `update` command for all mutations
- YAML/TOML for optional fields
- Custom DSL (e.g. `@A due:next-week`)
- Positional optional arguments

## Decision Outcome

Chosen option: "Dedicated commands with JSON for optional fields", because it achieves the lowest token cost on the most frequent operation (`complete`), remains extensible without flag proliferation, and leverages JSON which is natural for AI training data.

Command structure:

```bash
kid list     [--json] [--pretty]
kid add      --summary <text> [--details <JSON>]
kid rename   --id <uuid> --summary <text>
kid replace  --id <uuid> --details <JSON>
kid update   --id <uuid> --details <JSON>
kid complete --id <uuid> [--reopen]
kid schema   [--pretty] [--out <file>]
```

`add`, `replace`, and `update` accept JSON for detail fields:

```bash
kid add --summary "Fix leak" --details '{"priority":"A","due_date":{"Guess":"next week"}}'
kid update --id <uuid> --details '{"priority":"B","notes":"Rescheduled"}'
```

`replace` sets all detail fields to exactly the provided JSON (absent fields become `None`). `update` patches only the provided fields; omitted fields are unchanged. Neither can touch `summary` or `status`.

`kid schema` outputs the full JSON Schema for the `Details` struct, generated at runtime from `schemars`. The AI queries this once and caches it in context.

Server address is configured via `--server` or `KID_SERVER_ADDR` (default `127.0.0.1:9000`).

### Consequences

- Good, because `kid complete --id <uuid>` saves ~43% tokens vs a unified update approach on the most frequent operation
- Good, because adding new task fields requires only a schema update, no new CLI flags
- Good, because `kid schema` provides a complete, always-current API reference
- Good, because four update commands (`complete`/`rename`/`replace`/`update`) map to distinct intents — AI learns when to use each
- Good, because explicit `rename` adds friction by design, preventing accidental summary changes
- Bad, because JSON strings require shell quoting, minor friction for AI text generation
- Bad, because `add` has a two-part structure (summary flag + JSON details) — AI must compose both
- Bad, because `DateEstimation`/`TimeEstimation` use `Guess`/`Precise` wrappers, adding nesting depth to the schema

## Pros and Cons of the Options

### Dedicated commands with JSON for optional fields

- Good, because token-efficient on the most common operation
- Good, because extensible without flag proliferation
- Good, because JSON is natural in AI training data
- Bad, because schema complexity (Guess/Precise wrappers)

### CLI flags for all fields

```bash
kid add "Fix leak" --priority A --due-date "next week"
```

- Good, because familiar shell UX
- Bad, because flag proliferation as task fields grow
- Bad, because higher token cost (~30 tokens vs ~24)
- Bad, because less extensible — every new field requires a new flag

### Single `update` command for all mutations

```bash
kid update <uuid> '{"status":"Done","summary":"New","priority":"A"}'
```

- Good, because simpler command surface
- Bad, because no intent separation — status changes (frequent) same cost as metadata changes
- Bad, because summary unprotected from accidental modification

### YAML/TOML for optional fields

- Neutral, because human-readable
- Bad, because multiline strings in shell arguments are awkward
- Bad, because JSON is more natural for AI training data

### Custom DSL

```bash
kid add "Fix leak @A due:next-week ctx:Kitchen"
```

- Good, because compact
- Bad, because requires teaching the DSL in every prompt — token cost of explanation exceeds savings

### Positional optional arguments

```bash
kid add "Fix leak" A "next week" "Kitchen"
```

- Bad, because ambiguous — omitting a middle field requires sentinel values
- Bad, because field order is arbitrary and error-prone

## More Information

**Token cost comparison** (typical creation with 3 optional fields):

- This design: `kid add --summary "Fix leak" --details '{"priority":"A","due_date":{"Guess":"next week"},"context":"Kitchen"}'` ≈ 24 tokens
- Flag-based: `kid add --summary "Fix leak" --priority A --due-date "next week" --context "Kitchen"` ≈ 30 tokens (~20% reduction)

**Complete (most frequent):**

- This design: `kid complete --id <uuid>` ≈ 8 tokens
- Unified update: `kid update --id <uuid> --details '{"status":"Done"}'` ≈ 14 tokens (~43% reduction)

Over 1000 operations: ~6000 token savings.

**Clap structure** (simplified):

```rust
#[derive(Subcommand)]
enum Commands {
    Schema   { pretty: bool, outfile: Option<PathBuf> },
    List     { json: bool, pretty: bool, server: ServerArgs },
    Add      { summary: String, details: Option<String>, server: ServerArgs },
    Rename   { id: Uuid, summary: String, server: ServerArgs },
    Replace  { id: Uuid, details: String, server: ServerArgs },
    Update   { id: Uuid, details: String, server: ServerArgs },
    Complete { id: Uuid, reopen: bool, server: ServerArgs },
}
```

`update` accepts a `TaskDetailsPatch` (only present fields are changed). `replace` accepts a full `TaskDetails` (absent fields become `None`). Schema generation uses `schemars` on the `Details` type and evolves automatically when the struct changes.
