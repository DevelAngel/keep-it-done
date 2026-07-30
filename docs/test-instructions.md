# Test Instructions

## Unit Tests

Unit tests live inside the source files of `kid-types` (task serialization,
storage, legacy-format detection).

### Run all unit tests

```console
$ cargo test -p kid-types
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/lib.rs (target/debug/deps/kid_types-...)
running 33 tests
...
test result: ok.
```

### Run a single test

```console
$ cargo test -p kid-types -- flush
```

### Storage fault injection

`kid-types` exposes two feature flags for testing flush resilience.
They inject errors into `TaskCache::write_task_file` so the server's
retry logic can be exercised.

```console
$ cargo test -p kid-types --features ssr-test-storagefail
$ cargo test -p kid-types --features ssr-test-rand
```

| Feature                | Behaviour                              |
| ---------------------- | -------------------------------------- |
| `ssr-test-storagefail` | Every write returns an error           |
| `ssr-test-rand`        | Each write fails with 50 % probability |

## End-to-End Tests

The `kid-end2end` crate runs browser tests via
[Cucumber](https://crates.io/crates/cucumber) (Gherkin feature files)
and [thirtyfour](https://crates.io/crates/thirtyfour) (WebDriver).
Tests connect to a running `kid-server` instance and seed task data
via a test-control admin channel, with task mutations exercised
through the MCP client.

There are two feature files (tagged for selective execution):

| Tag           | Feature file          | Purpose                                       |
| ------------- | --------------------- | --------------------------------------------- |
| `@smoke`      | `navigation.feature`  | View switching (forward and backward arrows)  |
| `@screenshot` | `screenshots.feature` | Capture a screenshot per view for `README.md` |

### Requirements

- Chrome or Chromium installed on the system
- `kid-server` running on `:3000` (HTTP), `:9100` (MCP), and `:9200`
  (test-control admin channel, requires the `test-control` feature)

### Run all e2e tests (recommended)

`cargo-leptos` builds the frontend, starts the server, runs the tests,
and shuts everything down:

```console
$ cargo leptos end-to-end
```

This executes the command configured in `Cargo.toml`:

```toml
end2end-cmd = "cargo test --test browser"
```

### Run manually (server already running)

```console
$ cargo test -p kid-end2end --test browser
```

Filter by tag to run only a subset:

```console
$ cargo test --test browser -- --tags=@smoke
$ cargo test --test browser -- --tags=@screenshot
```

### Screenshots

The `@screenshot` feature uses a Scenario Outline that runs once per
view. Each scenario seeds tasks via the test-control admin channel
from a Gherkin data table,
opens the view directly via `?view=` query parameter (SSR, no WASM
hydration needed), and saves a PNG to `screenshots/` at the workspace
root. These paths are referenced in `README.md`.

### How it works

1. A **before hook** creates a temp directory; the **Given** step
   writes seed task files directly to disk, then calls `switch-dir`
   on the test-control admin channel so the server loads them.
2. Seed files get a backdated UUID v7 so `days ago` values are
   realistic.
   Dates (`start`, `due`) use relative day offsets (`+5`, `-3`, `0`).
3. Headless Chrome is launched at 360 x 1400 px (mobile width, tall) with
   scrollbars hidden. `WebDriver::managed` handles chromedriver
   lifecycle automatically.
4. The test navigates to each view (via URL param or arrow clicks)
   and asserts the page title.
5. An **after hook** restores the server's working directory and
   drops the temp dir.

### Task seed data

Tasks are defined in the feature file's data table — no fixture files.
Each view needs specific task properties to display content:

| View            | Needs                                              |
| --------------- | -------------------------------------------------- |
| Upcoming        | Tasks with `due_date` or `start_date`, status ToDo |
| Quick Wins      | Tasks with `time_estimate`, status ToDo            |
| All Open        | Any tasks with status ToDo                         |
| What I Finished | Tasks with status Done                             |
| Recent Changes  | Tasks with recent `authors` timestamps             |

### Troubleshooting

| Problem                        | Fix                                                   |
| ------------------------------ | ----------------------------------------------------- |
| WebDriver connection refused   | Install Chrome and ensure `chromedriver` is available |
| Screenshots show unstyled HTML | Run `cargo leptos build` to generate CSS/WASM assets  |
| Test-control connect failed    | Start the server: `cargo leptos watch`                |
| Upcoming view shows no tasks   | Verify seed tasks have `start` or `due` dates set     |

## Running all tests

```console
$ cargo test --workspace
```

Note: `cargo test --workspace` skips the e2e tests when the server is
not running (connection refused). Use `cargo leptos end-to-end` to
include them.
