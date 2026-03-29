---
status: accepted
date: 2026-03-29
---

# tarpc for Rust-Native RPC Communication

## Context and Problem Statement

The CLI tool and the server process need RPC communication. Both are written in Rust and run on the same machine, but TCP transport is required to support SSH tunnelling and flexible deployment. How should the RPC layer be implemented to achieve type-safe, compile-time-verified communication with minimal tooling dependencies?

## Decision Drivers

- Type-safe communication with compile-time verification on both sides
- No external tooling (no protoc, no code generation scripts)
- TCP transport for SSH tunnelling support
- Idiomatic Rust on both client and server
- Minimal dependency complexity

## Considered Options

- tarpc (Rust-native RPC framework)
- Protocol Buffers with prost
- gRPC with tonic
- Cap'n Proto
- JSON-RPC over HTTP
- Direct function calls via shared library

## Decision Outcome

Chosen option: "tarpc", because it defines the service interface directly in Rust using a `#[tarpc::service]` trait, provides compile-time verification that client and server agree on the interface, and requires only `cargo build` — no external tools, no build scripts, no type translation layer.

The service trait lives in `kid-types/src/rpc.rs` (behind the `rpc` feature flag):

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

Transport: `tarpc::serde_transport::tcp` with `tokio_serde::formats::Json`. Default listen address: `127.0.0.1:9000`, configurable via `--listen`.

### Consequences

- Good, because no external tooling — `cargo build` works immediately after clone
- Good, because native Rust types (`Option<T>`, `Result<T,E>`, enums with data) used directly — no translation layer
- Good, because the service trait is living documentation of the RPC interface
- Good, because async/await native — service methods integrate seamlessly with the tokio runtime
- Good, because JSON serialization enables human-readable wire traffic and easy debugging
- Bad, because Rust-only — no polyglot client support (acceptable: CLI and server are both Rust)
- Bad, because smaller community than gRPC/protobuf — fewer third-party tools
- Bad, because macro-generated code is not inspectable as files (use `cargo expand` if needed)
- Bad, because tarpc, serde, and tokio versions must remain compatible — coordinated upgrades required

## Pros and Cons of the Options

### tarpc

- Good, because zero external tooling dependencies
- Good, because service interface is a plain Rust trait — IDEs provide autocomplete across the full call chain
- Good, because flexible serialization (JSON now, bincode later without interface changes)
- Bad, because Rust-only ecosystem
- Bad, because smaller community than gRPC

### Protocol Buffers with prost

- Good, because language-neutral — enables polyglot clients
- Good, because explicit `.proto` schema serves as documentation for non-Rust readers
- Bad, because requires protoc compiler installation and build scripts for code generation
- Bad, because type translation overhead between proto types and Rust types

### gRPC with tonic

- Good, because industry-standard, large ecosystem, streaming support built-in
- Bad, because HTTP/2 overhead for local communication
- Bad, because tonic + wasm-bindgen version conflicts make it incompatible with the Leptos WASM build in the same workspace

### Cap'n Proto

- Good, because zero-copy deserialization
- Bad, because Rust implementation (capnp-rpc) is less mature
- Bad, because zero-copy optimization is irrelevant for task-sized messages (a few KB at most)

### JSON-RPC over HTTP

- Good, because simple, human-readable, extensive tooling
- Bad, because HTTP overhead is unnecessary for a direct TCP channel
- Bad, because conflates the CLI path with the browser HTTP path, adding complexity

### Direct function calls via shared library

- Good, because zero network overhead
- Bad, because both processes would access the filesystem concurrently, requiring complex locking
- Bad, because shared library versioning on Unix/Linux adds symbol resolution complexity

## More Information

The server implements `TaskService` in `kid-server/src/rpc.rs`, delegating to `SharedTaskCache`. The CLI creates a tarpc client in `kid-cli/src/main.rs` and calls methods as regular async functions.

Current service methods do not return `Result` — errors are handled via tarpc's transport-level error and logging. Mutations succeed silently; the dirty-tracking cache handles persistence asynchronously.

For version compatibility, all RPC-related dependencies are pinned at the workspace level in `Cargo.toml` and updated together.
