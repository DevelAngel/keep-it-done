# Keep It Done

Stop wrestling with task apps. Talk to your family's AI assistant instead — it turns a conversation into organized tasks everyone can see.

> [!IMPORTANT]
> Work-in-Progress! Not production ready.

## The Idea

Planning is the easy part; keeping track of who's doing what is the mental load. kid removes that load: talk naturally to your AI assistant — _"We're planning to renovate the kitchen. Help me organize the tasks."_ — and it creates and manages the tasks for you via MCP, while the whole family sees a live, shared view in the browser. No app to install, no manual syncing, no one person left holding the whole list in their head.

```
┌──────────────┐                    ┌──────────────┐
│ AI Assistant │ ── HTTP/MCP ─────> │  kid-server  │
│              │      (OAuth)       │              │
└──────────────┘                    │  ┌────────┐  │
                                    │  │ Tasks  │  │
┌──────────────┐                    │  │(Files) │  │
│ Family       │ ──── HTTP ──────>  │  └────────┘  │
│ (Browser)    │                    └──────────────┘
└──────────────┘
```

See [Mental Load Analysis](docs/analysis/mental-load-analysis-de.pdf) (in German) for the background research behind this idea.

## Features

- **No app for the family to learn** — the AI does the data entry, the browser view is read-and-check
- **Works on the kitchen tablet** — mobile-first, scrollable list, tap to expand details
- **Any AI assistant, not locked to one vendor** — standard MCP, OAuth-secured
- **Your family's data stays in your house** — no cloud database, no account required
- **Runs on hardware you already own** — a Raspberry Pi is enough, JSON files instead of a database

### A note on privacy

kid itself keeps all data on your home network.
However, task content is shared with whichever LLM your assistant uses —
cloud models (Claude, ChatGPT) send it to an external API,
while local models via [Ollama](https://ollama.com) keep everything on your own hardware.

[OpenClaw](https://github.com/openclaw/openclaw) is a self-hosted personal assistant
(WhatsApp, Telegram, Signal, and many more channels)
that can front either type of LLM.

## Quick Start

```bash
# Start the server (browser HTTP + MCP, on separate ports)
cargo run --bin kid-server

# MCP server listens on http://127.0.0.1:9100/mcp by default.
# Configure allowed OAuth clients via --mcp-clients-file
# (see server/mcp-clients.example.toml).

# Open browser
open http://localhost:3000
```

## Browser View

Five views, switchable by swipe or tap:

### Upcoming

![Upcoming: list of open tasks with checkboxes sorted by creation date.](screenshots/task-list-upcoming.png "Upcoming view")

### Quick Wins

![Quick Wins: open tasks with a time estimate sorted by age.](screenshots/task-list-quickwins.png "Quick Wins view")

### All Open

![All Open: list of open tasks with checkboxes sorted by creation date.](screenshots/task-list-allopen.png "All Open view")

### What I Finished

![What I Finished: completed tasks sorted by completion date.](screenshots/task-list-whatifinished.png "What I Finished view")

### Recent Changes

![Recent Changes: tasks updated within the last days sorted by most recent change.](screenshots/task-list-recentchanges.png "Recent Changes view")

### Task Details Expansion

![Task detail expansion: task expanded to reveal priority badge, due date, start date, context badge, and notes.](screenshots/task-detail-expansion.png "Task detail expansion")

### Auth Session Expiration Error

![Auth session expiration error: a frendly error message is shown](screenshots/session-expired.png "Auth session expiration error")

## Task Fields

Each task can carry: summary, priority (A/B/C), due date, start date, time estimate, context, and notes. Dates accept both precise timestamps and free-text estimates ("next Friday").

## MCP Tools & Resources

Tools (mutating and read operations):

```
list             — list tasks, filter by status and/or fuzzy search
add              — add a task (summary, category, contexts, details)
rename           — rename a task's summary
replace          — replace all task details (PUT semantics)
update           — patch task details (PATCH semantics)
complete         — complete or reopen a task
recategorize     — change a task's category
add_contexts     — add contexts, keeping existing ones
replace_contexts — replace all contexts
set_priority     — set or clear priority
set_assignee     — set or clear assignee
```

Read-only resources:

```
kid://categories       — categories currently in use
kid://contexts         — contexts currently in use
kid://report/daily     — open tasks grouped by due date
kid://report/backlog   — open tasks with no due date, not ready to start
kid://report/quick_wins — open tasks with a time estimate, shortest first
kid://report/weekly    — narrative review of the last 7 days
```

Each report also has a per-assignee variant, e.g. `kid://report/daily/alice` or `kid://report/daily/unassigned`.

## Architecture

```
kid/
├── types/    # kid-types: shared types, storage
├── app/      # kid-app:   Leptos UI (SSR + WASM)
├── frontend/ # kid-frontend: WASM binary
└── server/   # kid-server: Axum + Leptos + MCP server (OAuth-secured)
```

See [Architecture Overview](docs/architecture-overview.md) for details.

## Documentation

- [User Guide](docs/user-guide.md)
- [Architecture Overview](docs/architecture-overview.md)
- [Task Card Concept](docs/concepts/task-card.md)
- [ADRs](docs/adr/)

## Requirements

- Rust (stable)
- Unix-like system (Linux, macOS)

## License

AGPL-3.0
