---
status: accepted
date: 2026-05-01
---

# Infos/Details boundary and RPC patch strategy

## Context and Problem Statement

`Task` splits into `Infos` (list-level data) and `Details`
(expand-level data). The CLI's batch-update mechanism
(`update --details <JSON>`) is derived from `Details` via
`GeneratePatch`/`Patchable` macros. When a field moves from
`Details` to `Infos` — because it affects list rendering —
the field silently disappears from `DetailsPatch` and the CLI
loses the ability to change it.

Priority was the first field to move (2026-05-01). More will
follow: `time_estimate` (Quick Wins sorts by it),
`due_date` (list-level due indicators). Each move re-creates
the same problem.

## Considered Options

- **A: Dedicated RPC methods** — one method per moved field,
  like `rename`, `recategorize`, `set_priority`
- **B: Unified `TaskPatch`** — decouple the patch struct from
  `Details`; manually maintain a single patch that spans both
  `Infos` and `Details` fields
- **C: Merge Infos and Details** — flatten into one struct,
  one patch, one `update`; the "what the list shows" decision
  moves purely into server functions and frontend

## Decision Outcome

Chosen option: **A** (dedicated RPC methods) for now, because
the pattern is established and the change is minimal. Revisit
when the next field moves.

### Planned evolution

| Trigger | Action |
|---|---|
| Next field moves from Details to Infos | Switch to **B** |
| Details contains only `notes` (+`start_date`) | Consider **C** |

## Pros and Cons of the Options

### A: Dedicated RPC methods

- Good, because follows established pattern (`rename`,
  `recategorize`, `set_priority`)
- Good, because each method has clear semantics and intent
- Good, because minimal implementation effort per field
- Bad, because N moved fields = N new RPC methods + CLI
  subcommands; the command surface grows linearly
- Bad, because batch operations (`set priority + estimate in
  one call`) require multiple round-trips

### B: Unified `TaskPatch`

Replace the derived `DetailsPatch` with a manually maintained
`TaskPatch` that includes all batch-editable fields regardless
of which struct owns them:

```rust
#[skip_serializing_none]
pub struct TaskPatch {
    // from Infos
    priority: Option<Option<Priority>>,
    // from Details
    due_date: Option<Option<Date>>,
    start_date: Option<Option<Date>>,
    time_estimate: Option<Option<TimeEstimate>>,
    notes: Option<Option<String>>,
}
```

The server's `apply_patch` routes each field to the correct
struct. The `GeneratePatch`/`Patchable` derives stay on
`Details` for internal use but `TaskPatch` becomes the RPC
contract.

- Good, because the CLI interface stays stable across field
  moves — one `update` command, one JSON blob
- Good, because batch operations remain single round-trip
- Good, because `kid schema` can generate the full patch
  schema for AI consumers
- Bad, because `TaskPatch` must be kept in sync manually;
  adding a field to a struct without updating the patch is a
  silent omission
- Bad, because fields with special semantics (`status`,
  `summary`) must be explicitly excluded — the patch is not
  a mechanical mirror of the struct
- Neutral, because the existing `DetailsPatch` derives can
  remain for internal server use (`patch_details`)

### C: Merge Infos and Details

Eliminate the struct boundary. `Task` becomes flat with all
fields at one level. A single `TaskPatch` covers everything.
The distinction "what the list shows" is decided by which
fields server functions and frontend components expose.

- Good, because eliminates the field-migration problem
  permanently — there is no boundary to cross
- Good, because one struct, one patch, one `update` — minimal
  conceptual surface
- Bad, because `Infos` as a lightweight list-transfer type
  disappears; server functions must construct ad-hoc projections
  or transfer the full task to the list
- Bad, because larger refactor touching types, server functions,
  frontend components, CLI, and RPC trait
- Bad, because the compile-time guarantee "list views cannot
  access detail fields" is lost

## More Information

Fields that are candidates for moving to Infos (they affect
list-level perception or sorting):

| Field | Reason |
|---|---|
| `time_estimate` | Quick Wins view sorts by it |
| `due_date` | Overdue/upcoming indicators in list |

Fields that should stay in Details (only relevant on expand):

| Field | Reason |
|---|---|
| `notes` | Free text, too heavy for list transfer |
| `start_date` | Rarely displayed in list views |
