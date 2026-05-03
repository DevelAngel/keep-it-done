use super::{find, world::HOST};
use anyhow::Result;
use fantoccini::Client;
use std::path::Path;

pub async fn goto_path(client: &Client, path: &str) -> Result<()> {
    let url = format!("{HOST}{path}");
    client.goto(&url).await?;
    Ok(())
}

pub async fn goto_view(client: &Client, view: &str) -> Result<()> {
    goto_path(client, &format!("/?view={view}")).await
}

pub async fn goto_view_expand_first(client: &Client, view: &str) -> Result<()> {
    goto_path(client, &format!("/?view={view}&expand=first")).await
}

/// Wait for `main[data-hydrated]`, then click the next-view arrow.
pub async fn click_next_view_arrow(client: &Client) -> Result<()> {
    find::hydrated(client)
        .await
        .expect("WASM hydration did not complete — main[data-hydrated] missing");
    let arrow = find::next_view_arrow(client)
        .await
        .expect("Next-view arrow not found");
    arrow.click().await?;
    Ok(())
}

/// Capture a full-page PNG screenshot and save it to `screenshots/`.
pub async fn save_screenshot(client: &Client, filename: &str) -> Result<()> {
    let dir = screenshot_dir();
    std::fs::create_dir_all(&dir)?;
    let png = client.screenshot().await?;
    let path = dir.join(filename);
    std::fs::write(&path, &png)?;
    eprintln!("  screenshot: {}", path.display());
    Ok(())
}

fn screenshot_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("screenshots")
}
