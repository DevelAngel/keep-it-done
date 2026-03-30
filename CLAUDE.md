# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Keep It Done (kid)** — self-hosted, file-based family task manager. AI assistants use the `kid` CLI (tarpc/JSON over TCP); family members use a Leptos SSR+WASM web UI.

## Commands

```bash
cargo leptos watch                    # dev loop (hot-reload)
cargo build -p kid-cli --release      # CLI only
cargo leptos build --release ...      # server + WASM frontend
cargo test -p <crate> -- <test_name>
```

Formatting: `leptosfmt`, not `rustfmt` — configured in `rustfmt.toml` and `rust-analyzer.toml`.

## Architecture

```
kid-cli      ──→ kid-types (feature=cli)
kid-server   ──→ kid-types (feature=ssr) + kid-app (feature=ssr)
kid-frontend ──→ kid-app (feature=hydrate)
```

| Crate | Role |
|---|---|
| `types/` | `Task`, `TaskService` tarpc trait, `TaskCache` — feature-gated per consumer |
| `app/` | Leptos components; compiles to both SSR and WASM |
| `frontend/` | WASM entry point |
| `server/` | Axum HTTP (`:3000`) + tarpc TCP (`:9000`); owns `Arc<RwLock<TaskCache>>` |
| `cli/` | tarpc TCP client; JSON output |

**Storage:** `tasks/task-{uuid-v7}.json` per task, full in-memory `IndexMap`, flushed every 60s and on shutdown.

**`kid-types` features:** `cli` (rpc + clap/schemars), `ssr` (rpc + storage), `ssr-test-*` (test helpers). Check active feature when editing `types/src/`.

**RPC changes:** update both `kid-server/src/rpc.rs` (impl) and `kid-cli/src/main.rs` (client).

ADRs in `docs/adr/` — consult before changing storage, RPC, or UI framework.
