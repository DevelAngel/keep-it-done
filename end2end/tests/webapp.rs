use kid_cli::TaskServiceClient;

use anyhow::Result;
use assert_fs::TempDir;
use cucumber::{given, then, when, World};
use strum::{Display, EnumString};
use tarpc::context;
use thirtyfour::prelude::*;

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::fmt::{self, Debug, Formatter};

const RPC_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    9000,
);

#[derive(World)]
#[world(init = Self::new)]
pub struct AppWorld {
    http: WebDriver,
    rpc: TaskServiceClient,
    tasks_dir: Option<TempDir>,
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

#[given("no tasks at all")]
async fn empty_task_list(world: &mut AppWorld) -> Result<()> {
    let count = world.rpc.count(context::current()).await?;
    assert_eq!(count, 0);
    Ok(())
}

#[when("I open the app")]
async fn open_webapp(world: &mut AppWorld) -> Result<()> {
    world.http.goto("http://localhost:3000/").await?;
    Ok(())
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "lowercase")]
enum ViewSwitch {
    Next,
    Prev,
}

#[when(expr = "I click the {word} view arrow")]
async fn switch_view_by_arrow(world: &mut AppWorld, dir: ViewSwitch) -> Result<()> {
    let header = world.http.find(By::Tag("header")).await?;
    let button = header.find(By::Testid(format!("{dir}-view-arrow"))).await?;
    button.click().await?;
    Ok(())
}

#[then(expr = "I see the page title is {}")]
async fn validate_view_title(
    world: &mut AppWorld,
    expected_title: String,
) -> Result<()> {
    let header = world.http.find(By::Tag("header")).await?;
    let title = header.find(By::Tag("h1")).await?;
    let title = title.text().await?;
    assert_eq!(title, expected_title);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    AppWorld::cucumber()
        .max_concurrent_scenarios(Some(1))
        .fail_on_skipped()
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                let dir = TempDir::with_prefix("kid-e2e-")
                    .expect("failed to create temp dir");
                world
                    .rpc
                    .switch_dir(context::current(), dir.path().to_path_buf())
                    .await
                    .expect("RPC call failed")
                    .expect("switch_dir failed");
                world.tasks_dir = Some(dir);
            })
        })
        .after(|_feature, _rule, _scenario, _finished, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    let cwd = env::current_dir().expect("CWD available");
                    world
                        .rpc
                        .switch_dir(context::current(), cwd)
                        .await
                        .expect("RPC call failed")
                        .expect("switch_dir failed");
                    world.tasks_dir = None;
                }
            })
        })
        .run_and_exit("./features/webapp")
        .await;

    Ok(())
}
