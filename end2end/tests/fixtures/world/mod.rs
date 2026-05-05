pub mod action_steps;
pub mod check_steps;
pub mod given_steps;

use anyhow::Result;
use cucumber::World;
use fantoccini::{
    error::NewSessionError, wd::Capabilities, Client, ClientBuilder,
};
use std::path::{Path, PathBuf};

pub const HOST: &str = "http://127.0.0.1:3000";

/// Pixel 8 viewport (CSS pixels) — used for all tests.
pub const VIEWPORT_WIDTH: u32 = 412;
pub const VIEWPORT_HEIGHT: u32 = 915;

#[derive(Debug, World)]
pub struct AppWorld {
    client: Option<Client>,
    tasks_dir: PathBuf,
}

impl Default for AppWorld {
    fn default() -> Self {
        Self {
            client: None,
            tasks_dir: PathBuf::from(
                std::env::var("KID_TASKS_DIR")
                    .unwrap_or_else(|_| "target/e2e-fixtures".into()),
            ),
        }
    }
}

impl AppWorld {
    pub fn client(&self) -> &Client {
        self.client
            .as_ref()
            .expect("WebDriver client not initialized")
    }

    pub fn tasks_dir(&self) -> &Path {
        &self.tasks_dir
    }

    /// Called from the `before` hook — creates a fresh geckodriver session.
    pub async fn open_session(&mut self) -> Result<()> {
        self.client = Some(build_client().await?);
        Ok(())
    }

    /// Called from the `after` hook — closes the geckodriver session so
    /// the next scenario can start a new one.
    pub async fn close_session(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.close().await;
        }
    }
}

async fn build_client() -> Result<Client, NewSessionError> {
    let mut cap = Capabilities::new();
    let firefox_opts = serde_json::json!({
        "args": [
            "-headless",
            "-width", VIEWPORT_WIDTH.to_string(),
            "-height", VIEWPORT_HEIGHT.to_string()
        ]
    });
    cap.insert("moz:firefoxOptions".to_string(), firefox_opts);

    ClientBuilder::native()
        .capabilities(cap)
        .connect("http://localhost:4444")
        .await
}
