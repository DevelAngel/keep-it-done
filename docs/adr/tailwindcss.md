---
status: accepted
date: 2026-03-29
---

# Tailwind CSS for Browser UI Styling

## Context and Problem Statement

The browser interface needs CSS styling for the task list, task cards, and UI components. The system uses Leptos for the frontend with an SSR/hydration workflow. How should styling be implemented to enable fast iteration, avoid build tool complexity, and integrate cleanly with `cargo-leptos`?

## Decision Drivers

- No separate CSS build process — single command builds everything
- Fast development iteration without context switching to separate CSS files
- Minimal production bundle size
- Consistent visual language without a hand-crafted design system
- Responsive design for mobile and desktop

## Considered Options

- Tailwind CSS
- Plain CSS
- CSS-in-Rust (stylers, styled)
- Component libraries (Leptos UI, Yew UI)
- Bootstrap or Bulma
- No styling framework (inline styles)

## Decision Outcome

Chosen option: "Tailwind CSS", because `cargo-leptos` handles Tailwind compilation automatically — no separate `npx tailwindcss` process, no extra watch command. JIT compilation produces a minimal bundle. Utility classes applied directly in Leptos view macros keep styling co-located with markup.

Configuration: Tailwind v4 CSS-first — custom tokens in `style/tailwind.css` via `@theme`, no `tailwind.config.js`:

```css
@import "tailwindcss";

@theme {
  --color-task-priority-high: #ef4444;
  --color-task-priority-medium: #f59e0b;
  --color-task-priority-low: #10b981;
}
```

### Consequences

- Good, because `cargo leptos build` handles Tailwind — no separate tool to install or run
- Good, because utility classes enable rapid prototyping directly in view macros
- Good, because JIT produces 5–10 KB CSS for the task list UI (instead of megabytes)
- Good, because Tailwind's spacing/color scale ensures consistency without an explicit design system
- Good, because `md:` / `lg:` breakpoint prefixes make responsive layouts trivial
- Bad, because Tailwind class names are plain strings — typos (`bg-bleu-500`) produce no error, just no styling (mitigated by Tailwind IntelliSense)
- Bad, because complex components accumulate long class strings (mitigated by extracting Leptos components)

## Pros and Cons of the Options

### Tailwind CSS

- Good, because integrated into `cargo-leptos` — zero additional tooling
- Good, because JIT ensures unused classes are never shipped
- Good, because v4 CSS-first configuration removes `tailwind.config.js`
- Bad, because no compile-time validation of class names

### Plain CSS

- Neutral, because full control over styles
- Bad, because separate files to manage and keep in sync
- Bad, because no automatic purging of unused styles
- Bad, because naming conventions (BEM, etc.) required to avoid conflicts

### CSS-in-Rust (stylers, styled)

- Neutral, because Rust-native approach
- Bad, because smaller ecosystem, less documentation
- Bad, because no clear advantage over Tailwind for this use case

### Component libraries (Leptos UI, Yew UI)

- Good, because pre-built components
- Bad, because overkill for a simple task list
- Bad, because harder to customize for specific design tokens
- Bad, because adds dependency weight

### Bootstrap or Bulma

- Good, because widely known, extensive documentation
- Bad, because larger bundle sizes even with tree-shaking
- Bad, because opinionated component designs conflict with the app's custom gradient theme

### No styling framework

- Good, because zero dependencies
- Bad, because inconsistent spacing and colors without a design system
- Bad, because responsive layouts require manual media queries

## More Information

Tailwind integrates with the Leptos SSR workflow in three steps:

1. **Development**: `cargo leptos watch` recompiles Tailwind on file changes
2. **Production**: `cargo leptos build --release` generates the optimized CSS bundle
3. **Hydration**: CSS loads before WASM, preventing flash of unstyled content

The CSS bundle is served from `/pkg/` as a static asset, cacheable by browsers. The `tailwind-input-file` key in `Cargo.toml` (under `[package.metadata.leptos]`) points `cargo-leptos` to `style/tailwind.css`.

For conditional class application, use Leptos's `class:` directive:

```rust
<div class:bg-green-100={task.is_done()}
     class:bg-gray-100={!task.is_done()}>
```
