use super::find;
use anyhow::{Ok, Result};
use fantoccini::Client;

/// Assert page title matches. WebDriver waits for the `<h1>` text
/// to appear — handles both initial SSR and client-side view switches.
pub async fn page_title(client: &Client, expected: &str) -> Result<()> {
    assert!(
        find::page_title_with_text(client, expected).await.is_some(),
        "h1 with text '{expected}' not found — WASM hydration may have failed",
    );
    Ok(())
}

pub async fn has_tasks(client: &Client) -> Result<()> {
    let items = find::task_items(client).await;
    assert!(!items.is_empty(), "expected at least one task item");
    Ok(())
}

pub async fn has_task_checkbox(client: &Client) -> Result<()> {
    let checkbox = find::first_task_checkbox(client).await;
    assert!(checkbox.is_some(), "expected a task checkbox");
    Ok(())
}
