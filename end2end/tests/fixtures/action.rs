use super::{find, world::HOST};
use anyhow::Result;
use fantoccini::Client;

pub async fn goto_path(client: &Client, path: &str) -> Result<()> {
    let url = format!("{HOST}{path}");
    client.goto(&url).await?;
    Ok(())
}

pub async fn goto_view(client: &Client, view: &str) -> Result<()> {
    goto_path(client, &format!("/?view={view}")).await
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
