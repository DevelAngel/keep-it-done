use thirtyfour::WebDriver;

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static SCREENSHOTS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../screenshots")
        .canonicalize()
        .expect("screenshots/ directory must exist")
});

/// Save a screenshot to the repo-root `screenshots/` directory.
pub async fn save_screenshot(driver: &WebDriver, name: &str) {
    let path = SCREENSHOTS_DIR.join(format!("{name}.png"));
    if let Err(e) = driver.screenshot(&path).await {
        eprintln!("screenshot failed: {e}");
    }
}
