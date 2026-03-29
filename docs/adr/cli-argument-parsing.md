---
status: accepted
date: 2026-03-29
---

# CLI Argument Parsing with clap derive

## Context and Problem Statement

The `kid` CLI requires multiple subcommands, each with distinct arguments. How should argument parsing be implemented to ensure type safety, generate useful help output, and minimize boilerplate as the command set evolves?

## Decision Drivers

- Compile-time type safety for parsed arguments
- Automatic help generation from code
- Minimal boilerplate as new commands are added
- Rust-idiomatic approach

## Considered Options

- clap with derive macros
- clap builder pattern
- Manual argument parsing

## Decision Outcome

Chosen option: "clap with derive macros", because it enforces argument types at compile time, generates help text from doc comments automatically, and maps the command hierarchy onto Rust enums cleanly — all with less boilerplate than the alternatives.

### Consequences

- Good, because argument types are enforced at compile time
- Good, because help text is generated from doc comments automatically
- Good, because subcommands map naturally to Rust enum variants
- Good, because version flags work out of the box
- Bad, because proc macros add a small compile-time overhead
- Bad, because the `derive` feature must be explicitly enabled

## Pros and Cons of the Options

### clap with derive macros

Subcommands are Rust enum variants annotated with `#[derive(Subcommand)]`. Each variant holds its own typed fields as named struct members.

- Good, because argument definitions live with the types — declarative and co-located
- Good, because renaming or removing arguments is caught by the compiler
- Bad, because derive macro attribute syntax has a learning curve

### clap builder pattern

Runtime configuration of commands and arguments using a fluent API.

- Good, because no proc macro dependency
- Bad, because more verbose — each argument requires explicit builder calls
- Bad, because argument types are not enforced at compile time

### Manual argument parsing

Inspect `std::env::args()` directly.

- Bad, because no validation, no automatic help output
- Bad, because error-prone and requires manual maintenance

## More Information

Subcommands are defined as Rust enum variants annotated with `#[derive(Subcommand)]`. The top-level `Cli` struct uses `#[command(subcommand)]` to delegate dispatch to the enum. See `kid-cli/src/cli.rs` for the full definition.
