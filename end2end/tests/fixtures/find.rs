use fantoccini::{elements::Element, Client, Locator};

/// Wait for an `<h1>` whose text matches `text`.
/// WebDriver retries internally — no manual polling needed.
pub async fn page_title_with_text(client: &Client, text: &str) -> Option<Element> {
    let xpath = format!("//h1[normalize-space()='{text}']");
    client.wait().for_element(Locator::XPath(&xpath)).await.ok()
}

pub async fn task_items(client: &Client) -> Vec<Element> {
    client
        .find_all(Locator::Css("input[type='checkbox']"))
        .await
        .unwrap_or_default()
}

pub async fn first_task_checkbox(client: &Client) -> Option<Element> {
    client
        .wait()
        .for_element(Locator::Css("input[type='checkbox']"))
        .await
        .ok()
}

/// Wait until WASM hydration is complete.
/// The app sets `data-hydrated` on `<main>` via an `Effect` — this
/// only fires once all event handlers are attached.
pub async fn hydrated(client: &Client) -> Option<Element> {
    client
        .wait()
        .for_element(Locator::Css("main[data-hydrated]"))
        .await
        .ok()
}

/// Right-arrow button that advances to the next view.
pub async fn next_view_arrow(client: &Client) -> Option<Element> {
    client
        .wait()
        .for_element(Locator::Css(
            "button[aria-label^='Next view']",
        ))
        .await
        .ok()
}
