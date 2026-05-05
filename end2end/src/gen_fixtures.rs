//! Generate e2e task fixtures with dates relative to today.
//!
//! Writes task JSON files to `KID_TASKS_DIR` (default: `target/e2e-fixtures`).
//! Run before `cargo leptos end-to-end` so the server loads fresh data.
//!
//! ```sh
//! cargo run -p kid-end2end --bin gen-e2e-fixtures
//! KID_TASKS_DIR=target/e2e-fixtures KID_FALLBACK_USER=e2e-test cargo leptos end-to-end
//! ```

use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(
        std::env::var("KID_TASKS_DIR").unwrap_or_else(|_| "target/e2e-fixtures".into()),
    );
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let count = kid_end2end::write_standard_fixtures(&out_dir).expect("write fixtures");
    eprintln!("wrote {count} fixtures to {}", out_dir.display());
}
