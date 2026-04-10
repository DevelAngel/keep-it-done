---
name: kid-tasks
description: Manage family tasks using the `kid` CLI. Use when adding, listing, completing, updating or renaming tasks in the household task system. Triggers on: creating tasks, showing task list, marking tasks done/undone, updating task details, renaming tasks, asking about open tasks, assigning priorities, due dates, categories or contexts.
---

# kid-tasks

Family task manager via `kid` CLI (v0.9.0+). Server address configured via `KID_SERVER_ADDR`.

Every task **must** have a category and **should** have at least one context. Always assign both when creating or updating tasks.

## Task data model

`kid list --json` returns one object per task. All detail fields are top-level on `task` (no sub-object). `contexts` omitted when empty.

```json
{
  "id": "019d...",
  "task": {
    "summary": "Fix leaking faucet",
    "status": { "ToDo": { "since": "2026-04-05T10:00:00Z" } },
    "category": "Household",
    "contexts": ["@home", "@shopping"],
    "priority": "A",
    "due_date": { "date": "2026-04-10T09:00:00+02:00", "soft": false },
    "start_date": { "date": "2026-04-06T08:00:00+02:00", "soft": true },
    "time_estimate": "Hours2",
    "notes": "..."
  }
}
```

**`status`:** `{ "ToDo": { "since": "<ISO-8601>" } }` or `{ "Done": { "since": "<ISO-8601>" } }`

**`priority`:** `"A"` high/critical · `"B"` medium/important · `"C"` low/routine

**Dates:** `soft: true` = approximate, `soft: false` = hard deadline

**`time_estimate`:** `Min15` · `Min30` · `Min45` · `Hours1` · `Hours2` · `HalfDay` · `Day1` · `Day2`

**`category`:** Life domain. Examples: `Home`, `Household`, `Finance`, `DIY`, `School`, `Daycare`, `Vacation`, `School Start`, `Secret Santa`, `Office`, `Seasonal`, `Research` — fetch actual values with `kid categories`.

**`contexts`:** When and where? GTD-style, must start with `@`. Examples: `@low-energy` `@weekend` `@evening` `@together` `@outside` `@phone` `@computer` `@shopping` `@selling` `@car` `@calendar` `@office` `@working-hours` `@deep-focus` — fetch actual values with `kid contexts`.

Use the language of the household consistently: if tasks are in German, categories and contexts must be in German too.

## Core commands

```bash
kid list --json                   # list all tasks
kid complete --id <ID>            # mark done
kid complete --id <ID> --reopen   # reopen
```

### Add task

```bash
kid add --summary "Fix leaking faucet" --category "Household"
kid add --summary "File tax return" --category "Finance" -a @computer --details '{"priority":"A","due_date":{"date":"2026-04-30T00:00:00+02:00","soft":false}}'
```

`--category` required. `-a` repeatable for contexts — optional but strongly recommended. Prefer reusing existing categories and contexts (`kid categories`, `kid contexts`). If unclear, suggest options and ask before creating the task.

### Patch details (PATCH — omitted fields unchanged)

```bash
kid update --id <ID> --details '{"priority":"B","time_estimate":"Hours1"}'
kid update --id <ID> --details '{"priority": null}'   # delete field
```

### Replace all details (PUT — omitted fields cleared)

```bash
kid replace --id <ID> --details '{"priority":"B","notes":"..."}'
```

> `category` and `contexts` are not part of details — use dedicated commands.

## Category commands

```bash
kid categories                                     # list all categories in use
kid recategorize --id <ID> --category "Finance"    # change category
```

## Context commands

```bash
kid contexts                                       # list all contexts in use
kid add-contexts     --id <ID> -a @home -a @errands  # add contexts (keeps existing)
kid replace-contexts --id <ID> -a @computer          # replace all contexts
kid replace-contexts --id <ID>                       # remove all contexts
```

## Reporting

Every task has a `since` timestamp in its status. No external state needed.

- **Newly completed** — `status.Done.since` after last report timestamp
- **Newly created** — UUIDv7 timestamp of task ID after last report timestamp (`kid` guarantees UUIDv7)
- **Reopened** — status is `ToDo`, `status.ToDo.since` after last report, UUIDv7 timestamp before last report

> Reopenings indicate a task did not finish as planned — relevant for detailed reports.

## Rarely needed

```bash
kid rename --id <ID> --summary "New title"   # rename summary only — prefer update for all other changes
kid schema                                   # print JSON schema for details
```
