# ADR: Leptos for Browser-Based UI

## Status

Accepted

## Context

The task management system needs a browser-based interface for family members to view and interact with tasks. The interface must display tasks on a Kanban board, support filtering by category, and provide responsive updates when task state changes. Family members will access this interface from various devices including desktops, tablets, and smartphones.

The UI requirements include:
- Interactive Kanban board with drag-and-drop task movement
- Real-time or near-real-time updates when tasks change
- Category filtering and search
- Mobile-responsive design
- Acceptable performance with hundreds of tasks
- Simple deployment alongside the task server

Technology options for web UIs range from server-rendered HTML with minimal JavaScript to fully client-side single-page applications. The choice impacts development experience, performance characteristics, deployment complexity, and maintainability.

## Decision

We will use Leptos (https://www.leptos.dev/) as the framework for the browser-based UI. Leptos is a Rust web framework that compiles to WebAssembly for the client side and provides server functions for backend communication. It offers fine-grained reactivity similar to SolidJS, allowing efficient UI updates without virtual DOM overhead.

The architecture uses Leptos in fullstack mode:
- Client code compiles to WASM (`kid-frontend` crate, `cdylib`) and runs in the browser
- Server functions provide typed API endpoints (`kid-app` crate, feature `ssr`)
- The Leptos server integrates with the existing `kid-server` process
- Server functions access the same `SharedTaskCache` that the RPC interface uses

Server functions are defined as async Rust functions annotated with `#[server]`. The client calls these as if they were local, and Leptos generates the HTTP requests automatically:

```rust
#[server]
pub async fn get_tasks() -> Result<Vec<(Uuid, Task)>, ServerFnError> {
    let cache = use_context::<SharedTaskCache>()
        .ok_or_else(|| ServerFnError::new("Storage unavailable"))?;
    let tasks = cache.read().await.iter()
        .map(|(id, task)| (*id, task.clone()))
        .collect();
    Ok(tasks)
}
```

The client-side components use reactive signals to manage state and automatically re-render when data changes. This provides a responsive user experience without manual DOM manipulation.

## Consequences

### Positive

**Type safety across client-server boundary**: Server functions provide compile-time type checking between client and server. When you call `get_tasks()` from the client, the compiler verifies argument types and return types match the server implementation. This catches entire classes of integration bugs at compile time that would be runtime errors with traditional REST APIs.

**Unified language and ecosystem**: Both client and server code are Rust. Developers can share types, utilities, and logic between frontend and backend. The Task struct definition is identical on both sides. Error handling uses the same Result types. This eliminates context switching between languages and allows full-stack Rust developers to be productive across the entire codebase.

**Fine-grained reactivity**: Leptos uses signals and effects for reactivity. Only components that depend on changed data re-render. This is more efficient than React's virtual DOM diffing for complex UIs with many components. The Kanban board can have hundreds of task cards, but updating one task only re-renders that card, not the entire board.

**WebAssembly performance**: Client-side logic runs as WASM, which executes faster than JavaScript for compute-intensive operations. Filtering large task lists, calculating dependencies, or implementing complex UI interactions benefit from near-native performance. WASM also provides a smaller bundle size compared to equivalent JavaScript frameworks.

**Server-side rendering support**: Leptos can render components on the server for initial page load, sending HTML to the browser. This improves perceived performance and SEO (though SEO is not relevant for this private family app). SSR also means the app works with JavaScript disabled for basic viewing, though interaction requires WASM.

**Reactive primitives feel native to Rust**: Leptos's API design uses Rust idioms like closures and ownership rather than trying to mimic JavaScript frameworks. This makes it feel natural for Rust developers. The learning curve is gentler than trying to use JavaScript frameworks through WASM bindings.

**Active development and modern design**: Leptos is actively developed with a focus on Rust best practices and modern web development patterns. It learns from the experience of React, Vue, and Solid, incorporating their insights while leveraging Rust's type system. The framework continues to improve rapidly.

**Integrated routing**: Leptos provides a router that works with server-side rendering and client-side navigation. URLs map to components, and navigation is instant without full page reloads. This creates a native app-like experience in the browser.

### Negative

**Immature ecosystem**: Leptos is relatively young compared to established JavaScript frameworks. Fewer third-party component libraries exist. Less Stack Overflow content and community resources. Bugs and rough edges remain in less-traveled code paths. The API is still evolving, which may require migration effort during framework updates.

**WASM limitations**: WebAssembly has constraints compared to JavaScript. DOM manipulation goes through JavaScript interop, adding overhead. WASM binaries are larger than minified JavaScript for simple apps. Browser debugging tools for WASM are less mature than for JavaScript. Not all JavaScript libraries can be easily called from WASM.

**Build complexity and compilation time**: Compiling Rust to WASM adds build steps and time compared to JavaScript bundlers. The initial WASM compilation can take 30+ seconds even for small apps. This slows the development feedback loop compared to hot-reloading JavaScript development servers. Incremental compilation helps but doesn't eliminate the issue.

**Learning curve for web developers**: Developers experienced with React or Vue must learn Leptos's reactive model and how it maps to Rust concepts like ownership and borrowing. The mental model differs from JavaScript frameworks. Team members without Rust experience face a steeper learning curve than with JavaScript alternatives.

**Server function limitations**: Server functions are convenient but have constraints. They must serialize all arguments and return values. Complex types or references cannot be passed directly. Each server function call is a separate HTTP request, which can lead to request waterfalls if not carefully designed. There is no built-in batching or caching beyond browser HTTP caching.

**Dependency version conflicts**: As discovered during research, combining Leptos with certain other libraries (notably gRPC/tonic) can create unsolvable dependency conflicts around wasm-bindgen and wasm-streams versions. This limits architectural options and requires careful dependency management. The WASM ecosystem's version sensitivity creates fragility.

**Mobile experience requires additional work**: While Leptos produces responsive web apps, creating a truly mobile-optimized experience requires careful CSS work, touch gesture handling, and performance optimization. Leptos does not provide mobile-specific abstractions. For the family use case where mobile access is secondary, this is acceptable, but it's a constraint for mobile-first applications.

**No offline-first primitives**: Leptos does not include built-in support for offline operation, local-first data sync, or service workers. These must be implemented manually if needed. For the task manager where multiple family members might want to work offline and sync later, this is a limitation.

### Mitigations

For the immature ecosystem, we accept this as the cost of using modern technology that fits our use case well. We contribute to the Leptos community by documenting patterns we discover and reporting bugs we encounter. For missing component libraries, we build custom components, which provides learning opportunities.

For WASM limitations, we keep client-side logic focused on UI concerns. Heavy computation or data processing happens on the server. We use browser developer tools' WASM debugging features where available and rely on logging for cases where they fall short.

For build time, we use `cargo watch` for development to get automatic rebuilds. We structure code to minimize what needs recompilation on changes. The full build runs in CI but developers rarely need to rebuild everything from scratch.

For the learning curve, we provide comprehensive documentation and examples. We start with simple components and gradually introduce advanced patterns. Code reviews ensure Leptos patterns are used idiomatically.

For server function limitations, we design coarse-grained server functions that fetch all data needed for a UI view in one call. We use Leptos resources for caching and automatic refetching. For real-time updates, we plan to add Server-Sent Events to push changes rather than polling with server functions.

For dependency conflicts, we carefully audit dependencies before adding them. We avoid libraries known to conflict with Leptos's WASM compilation. For the gRPC conflict specifically, we keep the RPC layer (tarpc over TCP) entirely separate from the web layer, with both accessing the shared `SharedTaskCache`.

For mobile experience, we use responsive CSS design and test on mobile devices regularly. We accept that the initial version may not be perfectly optimized for mobile, with improvements coming iteratively based on actual family usage patterns.

For offline support, we defer this to a future enhancement. The initial version requires internet connectivity. If offline operation becomes important, we can add service workers and local storage using Leptos's JavaScript interop capabilities.

## Alternatives Considered

### Server-rendered HTML with minimal JavaScript

Use a traditional server-side framework (like Axum with Askama templates) to render HTML on the server, with minimal JavaScript for interactivity. Forms submit via POST, triggering full page reloads.

Rejected because it provides a dated user experience compared to modern single-page apps. Drag-and-drop task movement on the Kanban board would be difficult without substantial JavaScript. Real-time updates require WebSocket or SSE integration with manual DOM manipulation. The development experience splits between Rust server code and vanilla JavaScript client code.

While this approach has deployment simplicity and works without WASM, the user experience disadvantages outweigh these benefits for an interactive app like a task manager.

### React or Vue with REST API

Build the frontend in React or Vue (JavaScript/TypeScript) consuming REST endpoints from the Rust server. This is the conventional approach for web applications with separate frontend and backend.

Rejected because it splits the codebase into two languages. Developers must context-switch between Rust and TypeScript. Type definitions must be manually synchronized between client and server (or use code generation tooling). The Task type exists in both Rust and TypeScript with potential for divergence.

The REST API must be versioned and documented separately. Integration testing requires running both frontend and backend. Deployment becomes more complex with separate build processes.

For a small team or solo developer project, maintaining two language ecosystems adds cognitive overhead. The type safety benefits of full-stack Rust outweigh the larger JavaScript ecosystem.

### Yew (alternative Rust WASM framework)

Use Yew instead of Leptos for the Rust WASM frontend. Yew is more mature and has larger community.

Rejected because Yew's API is modeled after React with virtual DOM diffing, which is less efficient than Leptos's fine-grained reactivity. Yew requires more boilerplate for component definition. Most importantly, Yew does not have server functions, so we would need to manually implement API calls and handle serialization.

Leptos's server functions provide the key advantage of type-safe client-server communication without boilerplate. This architectural benefit is worth the trade-off of using a less mature framework.

### Dioxus (alternative Rust WASM framework)

Use Dioxus, which supports web, desktop, and mobile from shared code. This could provide a native desktop app for power users.

Rejected because Dioxus's web support is less mature than Leptos. The cross-platform capability is appealing but not needed for the initial version focused on browser access. Dioxus also lacks Leptos's server function integration, requiring manual API implementation.

If native desktop or mobile apps become requirements, Dioxus could be reconsidered. For now, a web-first approach with Leptos is simpler.

### htmx with server-side rendering

Use htmx to enhance server-rendered HTML with AJAX capabilities, keeping most logic server-side while providing interactivity without writing JavaScript.

Rejected because htmx still requires server-side rendering in a templating language, splitting concerns between Rust server logic and HTML templates. Complex interactions like drag-and-drop require custom JavaScript anyway. htmx's strength is progressive enhancement, but we want a fully interactive SPA experience.

The approach is interesting for simpler applications but doesn't fit the Kanban board interaction model well.

## Implementation Notes

The Leptos application is split across two crates in the workspace:
- `kid-app` — shared SSR + hydration logic; server functions (compiled into `kid-server` via feature `ssr`) and the WASM client (via feature `hydrate`)
- `kid-frontend` — the `cdylib` WASM binary; thin entry point that mounts `kid-app`

Build tooling uses **`cargo-leptos`** (not trunk). It handles WASM compilation, asset bundling, Tailwind CSS, and the development reload server (port 3001 by default). The built site is emitted to `target/site/`.

`kid-server`'s `main.rs` initializes both the tarpc TCP listener for CLI communication and the Leptos/Axum HTTP server for browser communication. Both share `SharedTaskCache` through `Arc<RwLock<TaskCache>>`.

Server functions access the shared cache via `use_context::<SharedTaskCache>()`, which is injected per-request by the Axum router. This provides typed access without global variables.

CSS uses Tailwind via `style/tailwind.css`, processed by `cargo-leptos` as part of the build pipeline.
