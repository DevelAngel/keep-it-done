pub mod action_steps;
pub mod check_steps;

use anyhow::Result;
use cucumber::World;
use fantoccini::{
    error::NewSessionError, wd::Capabilities, Client, ClientBuilder,
};

pub const HOST: &str = "http://127.0.0.1:3000";

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct AppWorld {
    pub client: Client,
}

impl AppWorld {
    async fn new() -> Result<Self, anyhow::Error> {
        let client = build_client().await?;
        Ok(Self { client })
    }
}

/// Pixel 8 viewport (CSS pixels) — used for all tests.
pub const VIEWPORT_WIDTH: u32 = 412;
pub const VIEWPORT_HEIGHT: u32 = 915;

async fn build_client() -> Result<Client, NewSessionError> {
    let mut cap = Capabilities::new();
    let chrome_opts = serde_json::json!({
        "args": ["-headless"],
        "mobileEmulation": {
            "deviceMetrics": {
                "width": VIEWPORT_WIDTH,
                "height": VIEWPORT_HEIGHT,
                "pixelRatio": 1.0,
                "mobile": true
            }
        }
    });
    cap.insert("goog:chromeOptions".to_string(), chrome_opts);

    ClientBuilder::native()
        .capabilities(cap)
        .connect("http://localhost:4444")
        .await
}
