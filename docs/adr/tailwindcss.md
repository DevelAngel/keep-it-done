# ADR: Tailwind CSS for Browser UI Styling

## Status

Accepted

## Context

The browser interface requires CSS styling for the task list, task cards, and UI components. The interface serves families who need a clean, intuitive visual experience without complexity. The system uses Leptos for the frontend with SSR/hydration workflow.

Key requirements:
- Fast development iteration
- Maintainable styling as features grow
- No complex build toolchain
- Integration with Leptos SSR workflow
- Responsive design for desktop and mobile
- Consistent visual language

## Decision

We will use Tailwind CSS as the primary styling solution, integrated directly into the Leptos build process via `cargo-leptos`.

Implementation details:

- Tailwind CSS v4 with **CSS-first configuration** (`style/tailwind.css` contains `@import 'tailwindcss'`)
- No `tailwind.config.js` — custom design tokens are defined via `@theme` blocks in CSS
- No separate Tailwind CLI process needed
- `cargo leptos build` handles CSS compilation automatically
- Utility-first classes applied directly in Leptos view macros
- JIT compilation for minimal CSS bundle size

## Consequences

### Positive

**Zero Build Tool Complexity**

Leptos's `cargo-leptos` handles Tailwind compilation automatically. No need to run separate `npx tailwindcss` commands, configure watch processes, or coordinate multiple build tools. Single command: `cargo leptos build`.

**Fast Development Iteration**

Utility classes enable rapid UI prototyping without context switching between Rust and CSS files. Change `class="bg-blue-500"` to `class="bg-green-500"` directly in the view macro.

**Minimal CSS Bundle**

JIT compilation includes only classes actually used in the codebase. A typical task list UI compiles to 5-10KB of CSS instead of megabytes. Critical for instant page loads on family home networks.

**No CSS Naming Conflicts**

Utility classes eliminate the need for naming conventions like BEM or CSS modules. No cognitive overhead deciding whether a class should be `.task-card__title` or `.card-title`.

**Responsive Design Built-In**

Breakpoint prefixes (`md:`, `lg:`) make responsive layouts trivial. No media query boilerplate. Example: `class="px-4 md:px-6"` adjusts padding for larger screens without a media query.

**Consistent Visual Language**

Tailwind's design system (spacing scale, color palette, typography) ensures consistency across components without explicit style guides. `p-4` always means 1rem padding.

**Easy Customization**

Define family-specific design tokens in `style/tailwind.css` once (Tailwind v4 CSS-first syntax):
```css
@import 'tailwindcss';

@theme {
  --color-task-priority-high: #ef4444;
  --color-task-priority-medium: #f59e0b;
  --color-task-priority-low: #10b981;
}
```
Then use `bg-task-priority-high` throughout the app.

**Type Safety with Leptos**

Tailwind classes are plain strings in Rust, but Leptos's compile-time view macro catches syntax errors. Typos in class names fail compilation, not at runtime.

### Negative

**String-Based Classes**

No compile-time validation of Tailwind class names themselves. Typing `bg-bleu-500` instead of `bg-blue-500` silently produces no styling. Mitigated by Tailwind VSCode extension with IntelliSense.

**Verbose Class Strings**

Complex components can accumulate long class strings, reducing readability. Mitigated by extracting reusable components or using Leptos's `class:` directive for conditional classes.

**Learning Curve for CSS Experts**

Developers comfortable with traditional CSS must learn Tailwind's utility class naming. However, for a family-scale project with occasional contributions, utility classes are easier to grasp than custom CSS architectures.

**Initial Configuration**

Requires `style/tailwind.css` with `@import 'tailwindcss'` and `cargo-leptos` configured with `tailwind-input-file`. One-time setup cost, documented in project README.

### Mitigations

For verbose classes, extract common patterns into Leptos components that encapsulate the class string once and reuse it everywhere.

For class name validation, use Tailwind IntelliSense in VSCode/IDE.

For conditional styling, leverage Leptos's `class:` directive:
```rust
<div class:bg-green-100={task.is_done()}
     class:bg-gray-100={!task.is_done()}>
```

## Alternatives Considered

### Plain CSS

Traditional approach with custom stylesheets. Rejected for several reasons:
- Requires managing separate CSS files
- No automatic purging of unused styles
- Naming conventions needed to avoid conflicts
- Harder to maintain as features grow
- Slower development iteration

### CSS-in-Rust (stylers, styled)

Rust crates that generate CSS from Rust code. Rejected because:
- Smaller ecosystem compared to Tailwind
- Less documentation and community support
- Still requires learning crate-specific APIs
- No clear advantage over Tailwind for this use case

### Component Libraries (Leptos UI, Yew UI)

Pre-built component libraries with styling included. Rejected because:
- Overkill for a simple task list
- Harder to customize for family-specific needs
- Adds dependency weight
- Tailwind provides same consistency with more flexibility

### Bootstrap or Bulma

CSS frameworks with pre-designed components. Rejected because:
- Larger bundle sizes (even with tree-shaking)
- Less customization flexibility
- Opinionated component designs may not fit family aesthetic
- Tailwind's utility-first approach more aligned with Leptos's component model

### No Styling Framework

Inline styles or minimal custom CSS. Rejected because:
- Inconsistent spacing and colors without design system
- Harder to maintain responsive layouts
- No automatic optimization
- Slower development for UI features

## Implementation Notes

Tailwind integrates seamlessly with Leptos's SSR workflow:

1. **Development**: `cargo leptos watch` compiles Tailwind on file changes
2. **Production**: `cargo leptos build --release` generates optimized CSS bundle
3. **Hydration**: CSS loads before WASM, preventing flash of unstyled content

The CSS bundle is served as a static asset from `/pkg/`, cacheable by browsers.

For the family scale (single-digit concurrent users), even non-optimized CSS loads instantly. JIT compilation ensures production builds stay minimal.

Configuration example (Tailwind v4 CSS-first):
```css
/* style/tailwind.css */
@import 'tailwindcss';

@theme {
  --color-status-todo: #f3f4f6;
  --color-status-done: #d1fae5;
}
```

No separate CSS files, no naming conflicts, instant styling.
