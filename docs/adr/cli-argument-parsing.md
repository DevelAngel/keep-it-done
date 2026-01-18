# ADR: CLI Argument Parsing with clap derive

## Status

Accepted

## Context

The application requires a command-line interface with multiple commands, each having distinct arguments. Users need accessible help texts and version information.

## Decision

Use `clap` with the `derive` feature for CLI parsing.

**Key aspects:**

- Leverage derive macros for declarative argument definitions
- Structure CLI with multiple subcommands
- Each command defines its own argument set
- Include comprehensive help texts
- Provide version information output

## Consequences

**Positive:**

- Type-safe argument parsing at compile time
- Reduced boilerplate compared to builder pattern
- Automatic help generation from doc comments
- Built-in version flag support
- Clear command structure through Rust enums

**Negative:**

- Requires `derive` feature dependency
- Slightly increased compile times due to proc macros
- Learning curve for derive macro attributes

## Alternatives Considered

- `clap` builder pattern: More verbose, runtime configuration
- `structopt`: Deprecated, merged into clap v3+
- Manual parsing: Error-prone, no validation

## Implementation Notes

```rust
#[derive(Parser)]
#[command(name = "app", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CommandA { /* args */ },
    CommandB { /* args */ },
}
```
