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

## Features

- **File-based storage** — tasks as JSON files, no database needed
- **Mobile-first browser view** — scrollable task list with expandable details
- **CLI for AI integration** — typed RPC over TCP, JSON output
- **Privacy first** — everything stays on your home network
- **Zero infrastructure** — runs on a Raspberry Pi or any home server

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

### Task List View

![Task list view: Browser view showing "My Day" with a teal header, an Add button, and a scrollable list of tasks with checkboxes and task summaries.](screenshots/task-list.png "Task list view")

### Task Details Expansion

![Task detail expansion: Same view with one task expanded, revealing labelled fields: priority badge, due date, start date, context badge, and a notes text block.](screenshots/task-detail-expansion.png "Task detail expansion")

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
