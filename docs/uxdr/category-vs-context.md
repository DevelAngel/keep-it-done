---
status: accepted
date: 2026-04-06
---

# Category vs. Context — Separation of Classification and Situational Filtering

## Context and Problem Statement

The current data model uses a single `context` field (free text, optional) that is used as a category label — e.g. `KID`, `Haushalt`, `Finanzen`. This works for grouping but conflates two orthogonal concepts: *what kind of task* (category) and *when/where to act on it* (context).

A user cannot answer *"What can I do right now, here, with my current energy level?"* without mentally scanning every task. As the task count grows, the flat list in "My Day" becomes overwhelming. Two distinct filtering dimensions are needed:

- **Category**: stable classification by life domain — enables grouping and visual structure
- **Context**: situational tags describing *when/where/how* a task becomes actionable — enables focus filtering

## Decision Drivers

- Family member is accustomed to categories from MS To Do — familiar mental model must be preserved
- Growing task list requires focus mechanisms beyond view switching
- AI assistant needs machine-readable signals for task recommendation ("What should I do now?")
- Contexts like `@abends` or `@telefon` encode constraints (kids asleep, quiet environment) that no category can express
- Both dimensions must work independently — neither depends on the other

## Considered Options

- Keep single `context` field as-is
- Rename `context` to `category` only (no situational filtering)
- Keep `context` field and add separate `contexts` list alongside it
- Replace `context` with `category` (required) and `contexts` (optional list)

## Decision Outcome

Chosen option: "Replace `context` with `category` + `contexts`", because it cleanly separates two orthogonal dimensions without backward ambiguity. Renaming alone solves the classification problem but leaves situational filtering out. Keeping the old field alongside new ones creates confusion about which to use. The clean replacement gives the AI assistant and the UI unambiguous signals.

| Aspect | Category | Context |
|---|---|---|
| Binding | Intrinsic — belongs to the task itself | Extrinsic — depends on person, time, environment |
| Stability | Fixed after creation (rare changes) | Fluid — may shift as circumstances change |
| Cardinality | Exactly one per task | Zero to many per task |
| Purpose | Classification: *What kind of work is this?* | Actionability filter: *When/where can I do this?* |
| Examples | `Kinder`, `Finanzen`, `Haushalt`, `KID` | `@telefon`, `@abends`, `@unterwegs`, `@kurz`, `@gemeinsam` |
| Interaction | Grouping, sorting, visual sections | Filtering, chip bar, quick-select |

### Data Model

**Before**

```json
{ "context": "KID" }
```

**After**

```json
{
  "category": "KID",
  "contexts": ["@deep-focus", "@büro"]
}
```

- `category` — required string, replaces the former `context` field
- `contexts` — optional list of strings, new field, each prefixed with `@`

Validation rules:
- `category` is mandatory on create — the AI assistant or web UI must set it
- `contexts` defaults to an empty list
- Context values must start with `@` to maintain visual distinction in the UI and CLI output
- Both fields are free text — no predefined enum, to allow organic growth

### Consequences

- Good, because category gives every task a stable home — grouping in views is always possible
- Good, because contexts enable focused filtering without restructuring views
- Good, because the AI assistant can infer category from natural language and suggest contexts based on task properties
- Good, because `@`-prefix makes context values visually distinct in CLI output and chip bars
- Bad, because all existing task files must be migrated (`"context"` → `"category"`, `"contexts": []`)
- Bad, because existing CLI usage of `--details '{"context": "..."}'` breaks — documentation and AI system prompt must be updated

## Pros and Cons of the Options

### Keep single `context` field as-is

- Good, because no migration needed
- Bad, because situational filtering remains impossible without mentally scanning all tasks
- Bad, because the AI assistant cannot distinguish task domain from actionability constraint

### Rename `context` to `category` only

- Good, because simple migration — rename one field
- Good, because resolves the naming confusion
- Bad, because no mechanism for situational filtering — the core problem persists

### Keep `context` and add `contexts` alongside

- Neutral, because preserves backward compatibility during transition
- Bad, because two fields for classification creates permanent confusion about which to use
- Bad, because AI and UI must handle two overlapping signals

### Replace `context` with `category` + `contexts`

- Good, because clean data model — each dimension has exactly one field
- Good, because `category` being required prevents uncategorised tasks piling up
- Good, because optional `contexts` allows gradual adoption — tasks without contexts work fine
- Bad, because breaking change — migration of all existing task files required

## More Information

### Web UI — Category grouping

Categories create visual structure inside any view (My Day, Quick Wins, etc.):

- Tasks are grouped by category with collapsible section headers
- Section headers show category name and task count
- Collapsed sections hide all tasks but remain visible as anchors
- Sort order within each group: Priority (A→C) → Due Date (overdue first) → Created

This transforms a flat list of 20 tasks into 4–5 named groups of 3–5 tasks each.

### Web UI — Context chip bar

A horizontal scrollable chip bar sits between the view header and the task list:

- Each chip represents one context: `@abends`, `@telefon`, `@unterwegs`, `@kurz`, `@gemeinsam`
- Single tap toggles a context filter — only matching tasks remain visible
- Multiple contexts can be active simultaneously (AND logic)
- Active chips are visually highlighted; inactive chips are muted
- The chip bar only shows contexts that exist on at least one visible task — no dead filters
- Clearing all chips restores the unfiltered view

Contexts are not visible in the task list row by default — they are a filter mechanism, not a display field. They appear in the detail expansion when a task is tapped.

**Interaction flow:**
1. User opens "My Day" — sees tasks grouped by category
2. It is 21:00, kids are asleep → user taps `@abends`
3. List reduces to tasks tagged `@abends`, still grouped by category
4. User sees 4 tasks across 2 categories — clear, actionable, focused

### AI Assistant

**Category inference:** the AI infers from natural language and sets `category` via `--details` JSON. If uncertain, it asks.

**Context suggestions:** the AI *suggests* contexts, it does not silently assign them. Context is personal.

- Task involves a phone call → suggest `@telefon`
- Time estimate ≤ 15 min → suggest `@kurz`
- Task requires partner involvement → suggest `@gemeinsam`
- Task involves errands or physical locations → suggest `@unterwegs`

**CLI filtering** (to be added):

```bash
kid list --category "Kinder"
kid list --context "@abends"
kid list --category "Finanzen" --context "@telefon"
```

This enables AI-driven recommendations: *"Was kann ich jetzt tun?"* → AI checks time of day, infers available contexts, queries filtered list.

### Migration Steps

1. Rename `context` → `category` in the JSON schema and storage layer
2. Add `contexts` (optional array) to the Details struct
3. Migrate existing task files: `"context": "X"` → `"category": "X"`, `"contexts": []`
4. Update `kid schema` output
5. Update CLI `--details` documentation and AI assistant system prompt
6. Update web UI: replace single context badge with category display + context chips
7. AI assistant starts suggesting contexts for new tasks after migration

Steps 1–3 are a single atomic migration. Steps 4–7 can be rolled out incrementally.

### Examples

```json
{
  "summary": "Call pediatrician for routine check-up",
  "category": "Children",
  "contexts": ["@phone", "@business-hours"],
  "priority": "B",
  "time_estimate": { "Guess": "10 min" }
}
```

Visible under **Children**; filtered out when `@evenings` is active; AI recommends on a weekday morning.

```json
{
  "summary": "Update dependency versions",
  "category": "KID",
  "contexts": ["@evenings", "@quick"],
  "priority": "C",
  "time_estimate": { "Guess": "15 min" }
}
```

Visible under **KID**; AI recommends when user says *"I have 15 minutes and I'm tired"*.

```json
{
  "summary": "Take out new accident insurance for the kids",
  "category": "Finance",
  "contexts": ["@together", "@deep-focus"],
  "priority": "A"
}
```

Category is **Finance**, not Children — the domain is finance even though children are involved. AI will not recommend this for quick evening slots.
