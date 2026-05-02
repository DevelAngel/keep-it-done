# Keep It Done

A lightweight, self-hosted task management system for families, optimized for natural interaction with AI assistants.

> [!IMPORTANT]
> Work-in-Progress! Not production ready.

## The Idea

Instead of wrestling with complex task apps, talk naturally to your AI assistant: _"We're planning to renovate the kitchen. Help me organize the tasks."_ The AI creates and manages tasks via CLI while the family sees a live browser view.

```
┌──────────────┐                    ┌──────────────┐
│ AI Assistant │ ── TCP/JSON ─────> │  kid-server  │
│ (kid CLI)    │                    │              │
└──────────────┘                    │  ┌────────┐  │
                                    │  │ Tasks  │  │
┌──────────────┐                    │  │(Files) │  │
│ Family       │ ──── HTTP ──────>  │  └────────┘  │
│ (Browser)    │                    └──────────────┘
└──────────────┘
```

See [Mental Load Analysis](docs/analysis/mental-load-analysis-de.pdf) (in German) for the background research behind this idea.

## Features

- **File-based storage** — tasks as JSON files, no database needed
- **Mobile-first browser view** — scrollable task list with expandable details
- **CLI for AI integration** — typed RPC over TCP, JSON output
- **Privacy first** — everything stays on your home network
- **Zero infrastructure** — runs on a Raspberry Pi or any home server

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
# Start the server
cargo run --bin kid-server

# In another terminal
cargo run --bin kid -- add --summary "Test the system"
cargo run --bin kid -- list

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

![Recent Changes: tasks updated within the last 24 hours sorted by most recent change.](screenshots/task-list-recentchanges.png "Recent Changes view")

### Task Details Expansion

![Task detail expansion: task expanded to reveal priority badge, due date, start date, context badge, and notes.](screenshots/task-detail-expansion.png "Task detail expansion")

## Task Fields

Each task can carry: summary, priority (A/B/C), due date, start date, time estimate, context, and notes. Dates accept both precise timestamps and free-text estimates ("next Friday").

## CLI Commands

```bash
kid list
kid add --summary "Buy paint" --priority B --estimate "2h" --context "Kitchen"
kid rename --id <uuid> --summary "New summary"
kid update --id <uuid> --details '{"priority": "A"}'
kid replace --id <uuid> --details '{...}'
kid complete --id <uuid>          # mark done
kid complete --id <uuid> --reopen # reopen
kid schema                        # print JSON Schema for task details
```

## Architecture

```
kid/
├── types/    # kid-types: shared types, RPC trait, storage
├── app/      # kid-app:   Leptos UI (SSR + WASM)
├── frontend/ # kid-frontend: WASM binary
├── server/   # kid-server: Axum + Leptos + tarpc listener
└── cli/      # kid-cli:   `kid` binary
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
