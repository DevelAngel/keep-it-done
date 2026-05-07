use kid_cli::TaskServiceClient;

use anyhow::Result;
use assert_fs::TempDir;
use cucumber::World;
use thirtyfour::prelude::*;

use std::fmt::{self, Debug, Formatter};

use crate::helpers::RPC_ADDR;

#[derive(World)]
#[world(init = Self::new)]
pub struct AppWorld {
    pub http: WebDriver,
    pub rpc: TaskServiceClient,
    pub tasks_dir: Option<TempDir>,
}

impl Debug for AppWorld {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppWorld")
            .field("http", &self.http)
            .field("rpc", &"TaskServiceClient { .. }")
            .field("tasks_dir", &self.tasks_dir)
            .finish()
    }
}

impl AppWorld {
    async fn new() -> Result<Self> {
        let mut capa = DesiredCapabilities::chrome();
        capa.set_headless()?;
        let http = WebDriver::managed(capa).await?;
        let rpc = kid_cli::connect(&RPC_ADDR)
            .await
            .expect("RPC connect failed");
        Ok(Self {
            http,
            rpc,
            tasks_dir: None,
        })
    }
}
