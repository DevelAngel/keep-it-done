---
status: accepted
date: 2026-03-29
---

# Leptos for Browser-Based UI

## Context and Problem Statement

The task management system needs a browser-based interface for family members to view and interact with tasks. The interface displays tasks as a mobile-first scrollable list with expandable details. Family members access it from desktops, tablets, and smartphones. How should the frontend be implemented to maximize type safety, minimize context switching, and keep deployment simple?

## Decision Drivers

- Full-stack type safety — client and server share types, compile-time verification across the boundary
- Single language (Rust) for both client and server
- Mobile-first responsive design
- Simple deployment alongside the existing `kid-server` process
- Acceptable performance with hundreds of tasks

## Considered Options

- Leptos (Rust WASM framework with server functions)
- Server-rendered HTML with minimal JavaScript (Axum + Askama)
- React or Vue with REST API
- Yew (Rust WASM framework)
- Dioxus (cross-platform Rust UI)
- htmx with server-side rendering

## Decision Outcome

Chosen option: "Leptos", because it provides compile-time type checking across the client–server boundary via server functions, eliminates the context switch between frontend and backend languages, and integrates directly into the existing `kid-server` process — with no separate deployment artifact.

Architecture (fullstack mode):

- Client: `kid-frontend` crate compiles to WASM (`cdylib`), runs in browser
- Server functions: `kid-app` crate with `ssr` feature, compiled into `kid-server`
- Both share `SharedTaskCache` via `use_context::<SharedTaskCache>()`

```rust
#[server]
pub async fn get_tasks() -> Result<Vec<(Uuid, Task)>, ServerFnError> {
    let cache = use_context::<SharedTaskCache>()
        .ok_or_else(|| ServerFnError::new("Storage unavailable"))?;
    Ok(cache.read().await.iter()
        .map(|(id, task)| (*id, task.clone()))
        .collect())
}
```

### Consequences

- Good, because server functions provide compile-time type checking between client and server — integration bugs caught at compile time
- Good, because `Task` struct, `Uuid`, and error types are identical on both sides — no translation layer
- Good, because fine-grained reactivity (signals/effects) rerenders only changed items, not the full list
- Good, because SSR provides HTML on first load — works before WASM is ready
- Good, because `cargo-leptos` handles WASM compilation, Tailwind, and dev reload in one command
- Bad, because Leptos is newer than React/Vue — fewer third-party components, smaller community
- Bad, because WASM binary is larger than equivalent JavaScript for simple apps
- Bad, because full rebuild takes 30+ seconds; incremental is faster but slower than JS hot reload
- Bad, because combining Leptos with certain libraries (notably tonic/gRPC) creates unsolvable wasm-bindgen version conflicts

## Pros and Cons of the Options

### Leptos

- Good, because type-safe client–server boundary with zero boilerplate
- Good, because single language, single ecosystem
- Good, because fine-grained reactivity — efficient for lists with many items
- Bad, because immature ecosystem compared to React/Vue
- Bad, because WASM debugging tooling less mature than browser JS devtools

### Server-rendered HTML with minimal JavaScript (Axum + Askama)

- Good, because simple, no WASM, works without JavaScript
- Bad, because interactive features (expandable detail rows) require substantial JavaScript
- Bad, because splits codebase between Rust templates and vanilla JavaScript

### React or Vue with REST API

- Good, because large ecosystem, mature tooling, widely known
- Bad, because two languages — Rust backend, TypeScript frontend — with manual type synchronization
- Bad, because REST API versioning and documentation overhead
- Bad, because separate build process and deployment artifact

### Yew

- Good, because more mature than Leptos, larger community
- Bad, because React-style virtual DOM diffing — less efficient than fine-grained reactivity for task lists
- Bad, because no built-in server functions — server communication requires manual implementation

### Dioxus

- Good, because cross-platform (web, desktop, mobile) from shared code
- Bad, because web support less mature than Leptos
- Bad, because no server function integration — same manual API problem as Yew

### htmx with server-side rendering

- Good, because progressive enhancement, works without JavaScript
- Bad, because server-side rendering requires a templating language, splitting Rust logic and HTML
- Bad, because a reactive task list with expandable details requires more JavaScript than htmx saves

## More Information

The Leptos application is split across two crates:

- `kid-app` — shared SSR + hydration logic; server functions compiled into `kid-server` via `ssr` feature, and into WASM via `hydrate` feature
- `kid-frontend` — thin `cdylib` entry point that mounts `kid-app`

Build tooling: `cargo-leptos` (not trunk). Handles WASM compilation, Tailwind CSS, and the dev reload server (port 3001). Built site goes to `target/site/`.

`kid-server/src/main.rs` starts both the tarpc TCP listener and the Leptos/Axum HTTP server. Both share `SharedTaskCache` via `Arc<RwLock<TaskCache>>`. Server functions receive the cache per-request via Axum's `Extension` injector.
