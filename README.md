# Keep It Done

A lightweight, self-hosted task management system designed for families, optimized for natural interaction with AI assistants.

> [!IMPORTANT]
> Work-in-Progress! Not production ready.

## The Idea

Instead of wrestling with complex task apps, just talk naturally to your AI assistant: _"We're planning to renovate the kitchen. Help me organize the tasks."_ The AI breaks it down, sets priorities, tracks dependencies, and keeps you focused on what matters next.

The system provides:

- **Simple file-based storage** - Tasks as JSON files, no database needed
- **Visual Kanban board** - Browser-based UI for the whole family
- **CLI for AI integration** - Command-line tool for AI assistants like [clawdbot](https://clawd.bot/)
- **Zero infrastructure** - Runs on a Raspberry Pi or any home server

## How It Works

```
┌──────────────┐                    ┌──────────────┐
│ AI Assistant │ ──── CLI ───────>  │ Task Server  │
│ (clawdbot)   │    via IPC         │              │
└──────────────┘                    │  ┌────────┐  │
                                    │  │Storage │  │
┌──────────────┐                    │  │(Files) │  │
│ Family       │ ──── HTTP ──────>  │  └────────┘  │
│ (Browser)    │                    └──────────────┘
└──────────────┘
```

### Natural Conversation

**You**: _"I have 30 minutes. What kitchen task can I do?"_

**AI Assistant**: _"Perfect! 'Research cabinet options' is exactly 30 minutes. Want me to suggest some German manufacturers to look at?"_

**You**: _"Yes, and mark it as in progress."_

**AI Assistant**: _"Done. Added Häcker, Nobilia, and Schüller to task notes. Task now in progress."_

The AI handles the complexity - you just describe what needs doing.

### Visual Overview

Open the browser interface to see your Kanban board:

```
┌──────────────┬──────────────┬──────────────┐
│   To Do      │ In Progress  │     Done     │
├──────────────┼──────────────┼──────────────┤
│ Research     │ Get quotes   │ Choose paint │
│ cabinets     │ (2 hours)    │ color        │
│ (30 min)     │ Priority A   │ (15 min)     │
│ Priority B   │              │              │
└──────────────┴──────────────┴──────────────┘
```

Both interfaces work on the same data. Talk to the AI or use the board - your choice.

## Key Features

### For Families

- **Simple deployment** - No database, no cloud accounts, just files
- **Privacy first** - Everything stays on your home network
- **Visual + conversational** - Board for overview, AI for interaction
- **Dependency tracking** - Tasks unlock automatically when blockers complete
- **Time-aware** - Filter by available time: "What can I do in 20 minutes?"

### For AI Assistants

- **IPC-based CLI** - Fast binary protocol over Unix socket
- **JSON output** - Easy parsing for AI processing
- **Complete CRUD** - Create, read, update, delete via command line
- **Rich queries** - Filter by status, priority, context, time estimate
- **Stateless design** - Each command is self-contained

## Architecture

Built with Rust using a clean separation of concerns:

```
task-manager/
├── task-types/      # Shared data structures and RPC definitions
├── task-service/    # Pure business logic (transport-agnostic)
├── task-server/     # Dual interface: IPC + HTTP
└── task-cli/        # Command-line tool for AI assistants
```

Storage is dead simple:

- Each task = one JSON file
- Flat directory structure: `tasks/*.json`
- Complete in-memory cache for instant queries
- Write-through caching for consistency

See [Software Architecture Overview](docs/architecture-overview.md) for details.

## Quick Start

```bash
# Start the server
cargo run --bin task-server

# In another terminal, use the CLI
cargo run --bin task-cli -- add "Test the system" --priority B --estimate "5 min"
cargo run --bin task-cli -- list

# Open browser
open http://localhost:3000
```

## Example: Kitchen Renovation

**You**: _"We're renovating the kitchen. Tasks: research cabinets, get contractor quotes, clear old cabinets, paint walls, install appliances. Help organize this."_

**AI Assistant** (via CLI):

```bash
# AI creates tasks with dependencies
task-cli add "Research cabinet options" --estimate "30 min" --priority B --context "Kitchen"
task-cli add "Get contractor quotes" --estimate "2 hours" --priority A --context "Kitchen"
task-cli add "Install appliances" --priority A --depends-on "Get contractor quotes"
# ... etc
```

**AI Response**: _"Created 5 tasks for kitchen renovation. 'Install appliances' is blocked until you get quotes. Want to tackle 'Research cabinets' first? It's only 30 minutes."_

The family sees the organized board, the AI manages the complexity.

## Use Cases

- **Project planning** - Break down renovations, vacations, events
- **School coordination** - Track kids' homework, projects, deadlines
- **Household management** - Chores, maintenance, shopping
- **Goal tracking** - Exercise routines, learning projects
- **Emergency response** - Quickly organize urgent situations

## AI Assistant Integration

Works with any AI assistant that can execute commands. Example with [clawdbot](https://clawd.bot/):

```bash
# clawdbot internally calls:
task-cli list --context "Kitchen" --status "todo"
task-cli show task-2024-001
task-cli update task-2024-001 --status "in-progress"
```

The CLI outputs JSON, making it easy for AI to parse and reason about tasks.

## Design Principles

### Simple Deployment

No database setup, no cloud accounts. Copy the binary and run it.

### Family Scale

Optimized for 2-4 users managing hundreds of tasks, not thousands of users with millions of tasks.

### Conversation First

Tasks shouldn't require form-filling. Describe what needs doing, let AI handle organization.

### Privacy by Default

Task data never leaves your home network unless you explicitly choose to sync it.

### Zero Lock-In

Tasks are plain JSON files. Read them with any tool, back them up anywhere.

## Documentation

- [User Guide](docs/user-guide.md) - For family members
- [Architecture Overview](docs/architecture-overview.md) - System design
- [Task Storage ADR](docs/task-storage-adr.md) - Storage decisions
- [Task Card Concept](docs/task-card.md) - Data model

## Requirements

- Rust 1.70+
- Unix-like system (Linux, macOS)
- 10MB disk space for typical family usage
- 5MB RAM for in-memory cache

## License

MIT

## Why This Exists

Traditional task apps force you to think like a database. This system lets you think like a human and talk naturally to an AI assistant that handles the organization.

For families who want:

- Task management without the management overhead
- Privacy without complexity
- Intelligence without cloud dependency (for task data)
- Simplicity without sacrificing capability

---

**Talk to your tasks. Let AI do the organizing.**
