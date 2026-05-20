arch := "aarch64"
libc := "musl"

# --- test ---

# Run all unit tests (types + app)
[group('test')]
test: test-types test-app

# Unit tests for kid-types (task model, storage)
[group('test')]
test-types:
    cargo test -p kid-types --features ssr

# Unit tests for kid-app (scheduling, upcoming logic)
[group('test')]
test-app:
    cargo test -p kid-app --features ssr

# Run a single test by name
[group('test')]
test-one name:
    cargo test -p kid-types --features ssr -- {{name}} \
      || cargo test -p kid-app --features ssr -- {{name}}

# End-to-end browser tests (requires running server)
[group('test')]
test-e2e:
    @killall --quiet --interactive --wait kid-server || true
    cargo leptos end-to-end

# --- build test ---

# Full workspace build check (catches cross-crate issues)
[group('lint')]
check:
    echo "{{os()}}"
    cargo check --workspace --all-targets

# --- debug build ---

[group('build-debug')]
server:
    cargo leptos build

[group('build-debug')]
cli:
    cargo build -p kid-cli

# --- release build ---

[group('build-release')]
release-native: release-native-server release-native-cli

[group('build-release')]
release-cross: release-cross-server release-cross-cli

[group('build-release')]
release-native-server:
    cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"

[group('build-release')]
release-native-cli:
    cargo build -p kid-cli --release --locked

[group('build-release')]
release-cross-server:
    env \
    LEPTOS_BIN_CARGO_COMMAND=cross \
    LEPTOS_BIN_TARGET_TRIPLE={{arch}}-unknown-linux-{{libc}} \
    cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"

[group('build-release')]
release-cross-cli:
    cross build --target {{arch}}-unknown-linux-{{libc}} -p kid-cli --release --locked

[group('build-release')]
release-frontend:
    cargo leptos build --frontend-only --release --lib-cargo-args="--locked"
