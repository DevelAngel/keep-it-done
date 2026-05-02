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

## End-to-End / Screenshot Tests

The `kid-end2end` crate uses [chromiumoxide](https://crates.io/crates/chromiumoxide)
(Chrome DevTools Protocol) to capture full-page screenshots of every view.
The test starts its own HTTP server with fixture data, so no manual server
setup is required.

### Requirements

- Chrome or Chromium installed on the system
- Frontend assets must be built once before the first run

### Build the frontend assets

```console
$ cargo leptos build
```

This compiles the WASM frontend and generates CSS into `target/site/`.
The E2E test serves these static assets.

### Run the screenshot tests

```console
$ cargo test -p kid-end2end --test screenshots -- --nocapture
     Running tests/screenshots.rs (target/debug/deps/screenshots-...)
running 1 test
  ✓ Upcoming → /path/to/kid/screenshots/task-list-upcoming.png
  ✓ Quick Wins → /path/to/kid/screenshots/task-list-quickwins.png
  ✓ All Open → /path/to/kid/screenshots/task-list-allopen.png
  ✓ What I Finished → /path/to/kid/screenshots/task-list-whatifinished.png
  ✓ Recent Changes → /path/to/kid/screenshots/task-list-recentchanges.png
  ✓ Detail expansion → /path/to/kid/screenshots/task-detail-expansion.png
test capture_readme_screenshots ... ok
```

Screenshots are written to `screenshots/` at the workspace root, matching
the paths referenced in `README.md`.

### How it works

1. Fixture task-JSONs from `end2end/fixtures/` are copied into a temp
   directory.
2. A `TaskCache` is created against that temp directory and loaded.
3. An HTTP server (`kid-server`) is started on a random port.
4. Headless Chrome is launched with a 390 x 844 viewport (iPhone 14).
5. The test navigates to each view by clicking the dot buttons
   (identified via `aria-label`), takes a full-page screenshot, then
   expands the first task on "All Open" for a detail screenshot.
6. Server and browser are shut down.

### Fixture data

The fixtures in `end2end/fixtures/` are hand-crafted task-JSONs that
populate all five views:

| View            | Needs                                              |
| --------------- | -------------------------------------------------- |
| Upcoming        | Tasks with `due_date` or `start_date`, status ToDo |
| Quick Wins      | Tasks with `time_estimate`, status ToDo            |
| All Open        | Any tasks with status ToDo                         |
| What I Finished | Tasks with status Done                             |
| Recent Changes  | Tasks with recent `authors` timestamps             |

File naming follows the storage convention: `task-{UUIDv7}.json`.
To add or change fixtures, create new JSON files using the task format
documented in `kid schema` (CLI) or `docs/concepts/task-storage.md`.

### Troubleshooting

| Problem                                           | Fix                                                            |
| ------------------------------------------------- | -------------------------------------------------------------- |
| `launch headless Chrome — is chromium installed?` | Install `chromium-browser` or `google-chrome`                  |
| Screenshots show unstyled HTML                    | Run `cargo leptos build` first                                 |
| `no fixture tasks loaded`                         | Check that `end2end/fixtures/` contains valid task-JSONs       |
| Server port conflict                              | The test binds to port 0 (OS-assigned); conflicts are unlikely |

## Running all tests

```console
$ cargo test --workspace
```

Note: this includes the E2E screenshot tests, which require Chrome and
pre-built frontend assets (see above).
