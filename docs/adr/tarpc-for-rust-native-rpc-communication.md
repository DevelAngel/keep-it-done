# ADR: tarpc for Rust-native RPC Communication

## Status

Proposed

## Context

The task management system requires RPC communication between the CLI tool and server process. Both components are written in Rust and run on the same machine. The CLI needs to invoke remote operations like listing tasks, creating tasks, and updating tasks, receiving strongly-typed responses.

Initial exploration considered Protocol Buffers with manual IPC implementation over Unix Domain Sockets. This approach provides language-agnostic serialization and explicit interface definitions in `.proto` files. However, it introduces external tooling dependencies (protoc compiler), requires build scripts for code generation, and creates an impedance mismatch between proto types and native Rust types.

The core requirements for the RPC layer are:
- Type-safe communication between CLI and server
- Compile-time verification that both sides agree on interface
- Efficient binary serialization for local IPC
- Support for Unix Domain Sockets as transport
- Minimal external dependencies and tooling
- Idiomatic Rust on both client and server

## Decision

We will use tarpc (https://github.com/google/tarpc) for RPC communication between CLI and server. tarpc is a Rust-native RPC framework that defines service interfaces directly in Rust code using procedural macros. The service definition is a Rust trait annotated with `#[tarpc::service]`, and tarpc generates client and server code automatically at compile time.

The RPC service interface will be defined in the `task-types` crate as a trait. Both server and CLI depend on this crate. The server implements the trait methods with actual business logic. The CLI uses the tarpc-generated client to make remote calls that look like regular async function calls.

Example service definition:
```rust
#[tarpc::service]
pub trait TaskService {
    async fn list_tasks(filter: Option<TaskFilter>) -> Vec<Task>;
    async fn create_task(action: String, status: TaskStatus) -> Result<Task, TaskError>;
    async fn update_task(id: String, updates: TaskUpdate) -> Result<Task, TaskError>;
    async fn delete_task(id: String) -> Result<(), TaskError>;
}
```

tarpc will handle serialization using serde with bincode (efficient binary format) by default. The transport will be Unix Domain Sockets for maximum performance on local IPC. tarpc provides built-in support for this through its transport abstraction layer.

## Consequences

### Positive

**Zero external tooling dependencies**: Developers only need Rust and Cargo. No protoc compiler installation, no version matching between protoc and prost, no build environment setup beyond standard Rust tooling. `cargo build` works immediately after cloning the repository. This significantly lowers the barrier to entry for contributors and simplifies CI/CD pipelines.

**Native Rust type system**: Service methods use idiomatic Rust types directly. `Option<T>` represents optional parameters naturally. `Result<T, E>` handles errors with full type information. Enums can carry associated data like `TaskUpdate::ChangeStatus(TaskStatus)`. Custom error types provide rich error context. There is no translation layer between proto-generated types and internal domain types.

**Simpler build process**: No build scripts required in any crate. tarpc's procedural macros expand during normal compilation as part of the standard Rust build pipeline. The generated code is not a separate artifact but integrated into the compilation process. This eliminates a class of build issues related to generated code being out of sync with definitions.

**Superior error ergonomics**: Methods return `Result<T, E>` where E is a custom error enum. The CLI can match on specific error variants and handle them appropriately. For example, `TaskError::NotFound` can be distinguished from `TaskError::ValidationFailed(String)` at the type level. This is more ergonomic than proto status codes with error message strings.

**Unified codebase**: The service definition lives in Rust code alongside other type definitions. When you change the `Task` struct, you immediately see which RPC methods are affected. The trait serves as living documentation of the RPC interface. IDEs provide autocomplete and type checking across the entire call chain from CLI to server implementation.

**Async/await native support**: tarpc is built on tokio and uses async/await throughout. Service methods are async functions that integrate seamlessly with the server's async runtime. This matches the idiomatic Rust async ecosystem and allows natural composition of async operations within RPC handlers.

**Flexible serialization**: While tarpc defaults to bincode, it can be configured to use JSON, MessagePack, or custom serialization formats. This flexibility allows optimization for different trade-offs (human readability vs binary efficiency) without changing the service interface.

**Transport abstraction**: tarpc supports multiple transports through a trait-based abstraction. Unix Domain Sockets work for local IPC, but if we later need TCP for remote access or in-process channels for testing, the service definition remains unchanged. Only the transport configuration changes.

### Negative

**Rust-only ecosystem**: tarpc works exclusively between Rust programs. If we later need a Python client for scripting or a JavaScript client for direct browser communication without HTTP translation, tarpc cannot help. Protocol Buffers would provide cross-language compatibility. For our current use case where CLI and server are both Rust, this is not a limitation, but it constrains future architectural options.

**Smaller community and ecosystem**: tarpc is less widely adopted than gRPC or other RPC frameworks. Fewer third-party tools exist for monitoring, testing, or debugging. Stack Overflow has fewer questions and answers. The community support is smaller, though the library itself is mature and well-maintained by Google.

**No standardized schema language**: Proto files serve as language-neutral interface documentation that non-programmers can read. tarpc service definitions are Rust code, requiring Rust knowledge to understand the interface. For internal projects this is acceptable, but for public APIs or cross-team coordination, a language-neutral schema can be valuable.

**Generated code not inspectable as files**: The macro-generated code exists only during compilation. While you can use `cargo expand` to see the expansion, it's not as straightforward as inspecting generated `.rs` files in a build directory. This can make debugging macro-related issues more difficult compared to build-script-generated code.

**Version compatibility dependencies**: tarpc, serde, bincode, and tokio versions must be compatible. Upgrading one dependency might require upgrading others. This creates a dependency graph that must be managed carefully. Protocol Buffers isolates proto version from implementation version more cleanly through the wire format stability.

**Learning curve for RPC concepts**: Developers unfamiliar with RPC frameworks must learn tarpc's concepts: service traits, client generation, transport configuration, context propagation. While this is simpler than learning Protocol Buffers + protoc + build integration, it still represents additional knowledge beyond basic Rust.

### Mitigations

For the Rust-only limitation, we accept this as appropriate for the current architecture where all components are Rust. If polyglot support becomes necessary, we can add a parallel Protocol Buffers or gRPC interface that delegates to the same business logic in `task-service`. The RPC layer would be one of multiple interfaces to the core logic.

For the smaller ecosystem, we rely on the fact that tarpc is maintained by Google and used in production systems. The library is stable and well-documented. For debugging, we can enable verbose logging and use tokio-console for async runtime inspection.

For version compatibility, we pin all RPC-related dependencies in the workspace-level `Cargo.toml` and update them together in coordinated releases. We test the dependency graph in CI before merging updates.

For the learning curve, we provide comprehensive documentation with examples of common patterns. The service trait definition serves as clear documentation of available operations. Code reviews ensure RPC patterns are used correctly.

## Alternatives Considered

### Protocol Buffers with prost

Use Protocol Buffers for message definitions with prost for Rust code generation, as explored in the initial proof-of-concept. Manually implement the RPC layer over Unix Domain Sockets with length-prefixed framing.

Rejected because it requires external protoc compiler installation, adds build script complexity, introduces type translation overhead between proto types and Rust types, and provides no advantage for Rust-to-Rust communication. The main benefit of Protocol Buffers is polyglot support, which we do not currently need.

The manual RPC implementation over sockets is error-prone and requires careful framing protocol design. tarpc provides this infrastructure battle-tested and optimized.

### gRPC with tonic

Use gRPC (tonic framework) which builds on Protocol Buffers and provides generated client/server code with streaming support.

Rejected for several reasons. gRPC is designed for HTTP/2 and network communication, adding overhead for local IPC where Unix Domain Sockets are more efficient. The dependency stack is larger (tonic, prost, h2, hyper) increasing build times and potential version conflicts. 

Most critically, gRPC integration with Leptos for the browser interface creates unsolvable dependency conflicts. Leptos uses specific versions of wasm-bindgen and wasm-streams for browser compilation. tonic pulls in different versions of these dependencies. When both exist in the same project, the dependency resolver produces conflicts that cannot be resolved without forking dependencies. This makes the development experience fragile.

### Cap'n Proto

Use Cap'n Proto, another binary serialization format with RPC support, designed for zero-copy deserialization.

Rejected because the Rust implementation (capnp-rpc) is less mature than tarpc. The zero-copy optimization is not meaningful for our small message sizes (tasks are a few kilobytes at most). The complexity of Cap'n Proto's schema language and encoding rules exceeds our needs.

### JSON-RPC over HTTP

Use JSON-RPC protocol over HTTP, which is simple and human-readable with extensive tooling.

Rejected because HTTP adds unnecessary overhead for local IPC. JSON serialization is slower and larger than binary formats for structured data. JSON-RPC is more appropriate for public APIs or browser integration, not for internal process communication where we control both ends.

The browser interface already uses HTTP, so JSON-RPC would make sense there, but for CLI-to-server communication where both are Rust and local, binary RPC over Unix sockets is more efficient.

### Direct function calls with shared library

Compile the business logic as a shared library that both CLI and server link against. The CLI would call functions directly without RPC.

Rejected because it requires both processes to access the filesystem concurrently for the task storage. This creates complex locking requirements and cache invalidation challenges. The clean separation of concerns where the server owns the storage and provides it through RPC is architecturally superior.

Additionally, shared libraries on Unix/Linux have versioning and symbol resolution complexities. The RPC approach keeps the CLI and server as independent executables with a clearly defined interface.

## Implementation Notes

The tarpc integration will be straightforward. The service trait goes in `task-types/src/lib.rs` alongside the Task and related types. The server implements the trait in `task-server/src/rpc_service.rs`, delegating to `task-service` business logic. The CLI creates a tarpc client in `task-cli/src/main.rs` and calls methods as if they were local async functions.

Transport configuration for Unix Domain Sockets uses tarpc's serde_transport module with a UnixStream. The server binds to the socket path, the CLI connects to it. tarpc handles all serialization, framing, and multiplexing automatically.

Error handling benefits from Rust's Result type. Server methods return `Result<T, TaskError>` where TaskError is a custom enum. tarpc serializes errors back to the client where they can be matched and handled appropriately.

This architecture provides the type safety and efficiency we need while remaining idiomatic Rust throughout the stack.
