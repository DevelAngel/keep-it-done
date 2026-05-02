//! E2E screenshot capture for the README.
//!
//! Starts kid-server against fixture data, launches headless Chrome,
//! navigates to each view via `?view=` query parameter (SSR-rendered),
//! and saves full-page screenshots into `screenshots/` at the workspace
//! root.
//!
//! # Prerequisites
//!
//! * `cargo leptos build` must have been run at least once so that
//!   `target/site/pkg/` contains the compiled CSS.
//! * Chrome or Chromium must be installed on the system.
//!
//! # Run
//!
//! ```sh
//! cargo test -p kid-end2end --test screenshots -- --nocapture
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::net::SocketAddr;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use kid_server::http::HttpServer;
use kid_server::SharedTaskCache;
use kid_types::server::TaskCache;
use leptos::config::LeptosOptions;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn screenshot_dir() -> PathBuf {
    workspace_dir().join("screenshots")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

/// Copy fixture task-JSONs into a temp directory and load them into a
/// [`TaskCache`].
async fn setup_task_cache() -> (tempfile::TempDir, SharedTaskCache) {
    let tmp = tempfile::TempDir::with_prefix("kid-e2e-").expect("create temp dir");

    for entry in std::fs::read_dir(fixtures_dir()).expect("read fixtures dir") {
        let entry = entry.expect("read fixture entry");
        if entry.path().extension().is_some_and(|e| e == "json") {
            let dest = tmp.path().join(entry.file_name());
            std::fs::copy(entry.path(), &dest).expect("copy fixture");
        }
    }

    let mut cache = TaskCache::default().with_dir(tmp.path());
    let (loaded, _migrated): (usize, usize) = cache.load().await.expect("load fixture tasks");
    assert!(loaded > 0, "no fixture tasks loaded — check end2end/fixtures/");

    let shared: SharedTaskCache = Arc::new(RwLock::new(cache));
    (tmp, shared)
}

/// Start the HTTP server on a random OS-assigned port. Returns the port
/// and a [`CancellationToken`] to trigger graceful shutdown.
async fn start_http_server(task_cache: SharedTaskCache) -> (u16, CancellationToken) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind to random port");
    let port = listener.local_addr().unwrap().port();

    // Construct LeptosOptions directly — avoids coupling to cargo-leptos
    // which sets LEPTOS_OUTPUT_NAME only during its own build.
    let site_root = workspace_dir().join("target").join("site");

    // Leptos's HydrationScripts component appends "_bg" to the WASM
    // filename when LEPTOS_OUTPUT_NAME is unset at compile time (which
    // is always the case under `cargo test`).  Create a symlink so the
    // browser can find kid_bg.wasm → kid.wasm.
    let wasm_src = site_root.join("pkg/kid.wasm");
    let wasm_link = site_root.join("pkg/kid_bg.wasm");
    if wasm_src.exists() && !wasm_link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&wasm_src, &wasm_link)
            .expect("create kid_bg.wasm symlink");
    }
    let options = LeptosOptions::builder()
        .output_name("kid")
        .site_root(site_root.to_string_lossy().as_ref())
        .site_pkg_dir("pkg")
        .site_addr(format!("127.0.0.1:{port}").parse::<SocketAddr>().unwrap())
        .build();

    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Set fallback user so server functions don't complain.
    // SAFETY: Called before spawning any threads that read this variable.
    unsafe { std::env::set_var("KID_FALLBACK_USER", "e2e-test") };

    tokio::spawn(async move {
        HttpServer::serve(listener, options, shutdown_clone, task_cache)
            .await
            .expect("HTTP server crashed");
    });

    // Wait for the server to be ready.
    wait_for_port(port).await;

    (port, shutdown)
}

/// Poll until the HTTP server accepts a TCP connection.
async fn wait_for_port(port: u16) {
    let addr = format!("127.0.0.1:{port}");
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(&addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("server on port {port} did not become ready in time");
}

/// Detect a Chrome/Chromium binary on the system.
///
/// chromiumoxide's auto-detection only checks a few well-known paths.
/// We extend the search to cover Playwright's cached download and
/// Flatpak/Snap locations.
fn find_chrome() -> Option<PathBuf> {
    let candidates = [
        // Standard system paths (checked by chromiumoxide automatically)
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        // Snap
        "/snap/bin/chromium",
    ];
    for name in candidates {
        if let Ok(path) = which::which(name) {
            return Some(path);
        }
    }
    // Playwright's cached Chromium download
    if let Some(home) = std::env::var_os("HOME") {
        let pw_dir = Path::new(&home).join(".cache/ms-playwright");
        if let Ok(entries) = std::fs::read_dir(&pw_dir) {
            for entry in entries.flatten() {
                let chrome = entry.path().join("chrome-linux64/chrome");
                if chrome.is_file() {
                    return Some(chrome);
                }
            }
        }
    }
    None
}

/// Launch headless Chrome with a mobile viewport.
async fn launch_browser() -> (Browser, tokio::task::JoinHandle<()>) {
    let mut builder = BrowserConfig::builder();
    builder = builder
        .window_size(412, 915)
        .arg("--force-device-scale-factor=1")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage");

    if let Some(chrome_path) = find_chrome() {
        eprintln!("  using chrome: {}", chrome_path.display());
        builder = builder.chrome_executable(chrome_path);
    }

    let config = builder.build().expect("browser config");

    let (browser, mut handler) = Browser::launch(config)
        .await
        .expect("launch headless Chrome — is chromium installed?");

    let handle = tokio::spawn(async move {
        while handler.next().await.is_some() {}
    });

    (browser, handle)
}

/// Pixel 8 viewport (CSS pixels).
const VIEWPORT_WIDTH: i64 = 412;
const VIEWPORT_HEIGHT: i64 = 915;

/// Set the viewport to Pixel 8 dimensions via CDP.
async fn set_viewport(page: &chromiumoxide::Page) {
    page.execute(SetDeviceMetricsOverrideParams::new(
        VIEWPORT_WIDTH,
        VIEWPORT_HEIGHT,
        1.0,   // device scale factor
        true,  // mobile
    ))
    .await
    .expect("set viewport");
}

/// Wait until a CSS selector matches at least one element.
async fn wait_for_selector(page: &chromiumoxide::Page, selector: &str) {
    for _ in 0..40 {
        if page.find_element(selector).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    panic!("timeout waiting for selector: {selector}");
}

/// Take a viewport-sized PNG screenshot and save it to disk.
async fn save_screenshot(page: &chromiumoxide::Page, path: &Path) {
    let png = page
        .screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build(),
        )
        .await
        .expect("take screenshot");
    std::fs::write(path, png).expect("write screenshot");
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

/// (view title, query-param value, screenshot filename).
const VIEWS: &[(&str, &str, &str)] = &[
    ("Upcoming",        "upcoming",  "task-list-upcoming.png"),
    ("Quick Wins",      "quickwins", "task-list-quickwins.png"),
    ("All Open",        "allopen",   "task-list-allopen.png"),
    ("What I Finished", "finished",  "task-list-whatifinished.png"),
    ("Recent Changes",  "recent",    "task-list-recentchanges.png"),
];

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn capture_readme_screenshots() {
    // -- Setup ---------------------------------------------------------------
    let (_tmp_dir, task_cache) = setup_task_cache().await;
    let (port, shutdown) = start_http_server(task_cache).await;
    let (browser, browser_handle) = launch_browser().await;

    let base_url = format!("http://127.0.0.1:{port}");
    let out = screenshot_dir();
    std::fs::create_dir_all(&out).ok();

    // -- Capture each view via SSR ------------------------------------------
    // Each view is loaded as a fresh page with `?view=` query parameter.
    // The server renders the correct view on the SSR pass — no WASM
    // hydration required.
    for (title, query, filename) in VIEWS {
        let url = format!("{base_url}/?view={query}");
        let page = browser.new_page(&url).await.expect("open page");
        set_viewport(&page).await;
        wait_for_selector(&page, "h1").await;

        // Verify the correct view loaded.
        let h1: Option<String> = page
            .evaluate("document.querySelector('h1')?.textContent?.trim()")
            .await
            .ok()
            .and_then(|v| v.into_value().ok());
        assert_eq!(
            h1.as_deref(),
            Some(*title),
            "SSR rendered wrong view for ?view={query}"
        );

        let path = out.join(filename);
        save_screenshot(&page, &path).await;
        eprintln!("  ✓ {title} → {}", path.display());
    }

    // -- Capture detail expansion -------------------------------------------
    // SSR pre-expands the first task via `?expand=first`.
    {
        let url = format!("{base_url}/?view=allopen&expand=first");
        let page = browser.new_page(&url).await.expect("open page");
        set_viewport(&page).await;
        wait_for_selector(&page, "h1").await;

        let path = out.join("task-detail-expansion.png");
        save_screenshot(&page, &path).await;
        eprintln!("  ✓ Detail expansion → {}", path.display());
    }

    // -- Teardown ------------------------------------------------------------
    shutdown.cancel();
    browser_handle.abort();
}
