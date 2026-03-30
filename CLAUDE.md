# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Keep It Done (kid)** — a self-hosted, file-based task management system for families, designed for natural AI assistant integration. The CLI (`kid`) speaks tarpc/JSON to the server; family members use a Leptos SSR+WASM web UI.

## Build & Development Commands

**Prerequisites:** Rust 1.88.0+, `cargo-leptos`, `wasm32-unknown-unknown` target

```bash
# Add WASM target (once)
rustup target add wasm32-unknown-unknown

# Full dev loop (server + frontend hot-reload)
cargo leptos watch

# Build CLI only
cargo build -p kid-cli --release --locked

# Build server + WASM frontend
cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"
```

**Tests:**
```bash
# Unit tests (no WASM targets needed)
cargo test -p kid-types
cargo test -p kid-server

# Run a single test
cargo test -p kid-types -- <test_name>

# SSR integration tests (uses ssr-test-* features)
cargo test -p kid-types --features ssr-test-add-tasks
```

**Formatting:** Uses `leptosfmt` (wraps `rustfmt` for Leptos component syntax). The `rustfmt.toml` and `rust-analyzer.toml` are already configured for this — run `leptosfmt` rather than `rustfmt` directly.

**Cross-compilation:**
```bash
export LEPTOS_BIN_CARGO_COMMAND="cross"
export LEPTOS_BIN_TARGET_TRIPLE="aarch64-unknown-linux-musl"
cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"
```

## Architecture

### Crate Relationships

```
kid-cli  ──→ kid-types (feature=cli)
kid-server ──→ kid-types (feature=ssr) + kid-app (feature=ssr)
kid-frontend ──→ kid-app (feature=hydrate)
kid-types uses kid-types-derive (internal derive macros)
```

### Five Crates

| Crate | Role |
|---|---|
| `types/` | Shared `Task` struct, `TaskService` tarpc trait, `TaskCache` storage — feature-gated |
| `app/` | Leptos components; compiles to SSR (server-side) and WASM (browser hydration) |
| `frontend/` | Thin WASM entry point; activates `hydrate` feature on `kid-app` |
| `server/` | Axum HTTP + tarpc TCP server; owns `SharedTaskCache: Arc<RwLock<TaskCache>>` |
| `cli/` | `kid` binary; tarpc TCP client; outputs JSON for AI consumption |

### Communication

- **CLI ↔ Server:** TCP tarpc with JSON serialization, default `127.0.0.1:9000`
- **Browser ↔ Server:** HTTP + Leptos server functions, default `127.0.0.1:3000`

### Storage Model

- One JSON file per task: `tasks/task-{uuid-v7}.json`
- Full in-memory `IndexMap<Uuid, Task>` loaded at startup
- Background flush every 60s (dirty set tracked with `IndexSet<Uuid>`)
- Atomic writes: temp file → rename
- Final flush on graceful shutdown (SIGTERM/SIGINT)

### Feature Flags in `kid-types`

`kid-types` is the shared heart of the system and compiles differently per consumer:
- `cli` — enables `rpc` + schemars + clap derives
- `ssr` — enables `rpc` + file storage logic
- `ssr-test-add-tasks` / `ssr-test-*` — test helpers that prepopulate the cache

Always check which feature is active when editing `types/src/`.

### RPC Service Trait

```rust
#[tarpc::service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task);
    async fn rename(id: Uuid, summary: String);
    async fn replace(id: Uuid, details: TaskDetails);
    async fn update(id: Uuid, details: TaskDetailsPatch);
    async fn complete(id: Uuid, reopen: bool);
}
```

Changes here require updating both `kid-server/src/rpc.rs` (impl) and `kid-cli/src/main.rs` (client).

## Key Conventions

- **Unsafe code is forbidden** (`unsafe_code = "forbid"` workspace-wide)
- **Edition 2024** throughout
- ADRs live in `docs/adr/` (MADR 4.0.0 format) — consult before changing storage, RPC, or UI framework decisions
- Task UUIDs are v7 (timestamp-sortable); filenames encode creation order
