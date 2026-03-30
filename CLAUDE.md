# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Keep It Done (kid)** — self-hosted, file-based task management for families. AI assistants use the `kid` CLI (tarpc/JSON over TCP); family members use a Leptos SSR+WASM web UI.

## Commands

```bash
cargo leptos watch                    # dev loop (hot-reload)
cargo build -p kid-cli --release      # CLI only
cargo leptos build --release ...      # server + WASM frontend
cargo test -p <crate> -- <test_name>  # run tests
```

Formatting uses `leptosfmt` (not `rustfmt`) — already configured in `rustfmt.toml` and `rust-analyzer.toml`.

## Architecture

```
kid-cli  ──→ kid-types (feature=cli)
kid-server ──→ kid-types (feature=ssr) + kid-app (feature=ssr)
kid-frontend ──→ kid-app (feature=hydrate)
```

| Crate | Role |
|---|---|
| `types/` | Shared `Task`, `TaskService` tarpc trait, `TaskCache` — feature-gated per consumer |
| `app/` | Leptos components; same code compiles to SSR and WASM |
| `frontend/` | Thin WASM entry point |
| `server/` | Axum HTTP (`:3000`) + tarpc TCP (`:9000`); owns `Arc<RwLock<TaskCache>>` |
| `cli/` | tarpc TCP client; JSON output for AI consumption |

**Storage:** one JSON file per task (`tasks/task-{uuid-v7}.json`), full in-memory `IndexMap` at runtime, flushed to disk every 60s and on shutdown.

**`kid-types` feature flags** — the crate compiles differently per consumer: `cli` (rpc + clap/schemars), `ssr` (rpc + storage), `ssr-test-*` (test helpers). Always check which feature is active when editing `types/src/`.

**RPC changes** require updating both `kid-server/src/rpc.rs` (impl) and `kid-cli/src/main.rs` (client).

ADRs in `docs/adr/` — consult before changing storage, RPC, or UI framework decisions.
