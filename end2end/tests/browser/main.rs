mod helpers;
mod oauth;
mod screenshots;
mod seeds;
mod steps;
mod world;

use anyhow::Result;
use assert_fs::TempDir;
use cucumber::World as _;

use std::env;

use helpers::TEST_CONTROL_ADDR;
use world::AppWorld;

#[tokio::main]
async fn main() -> Result<()> {
    AppWorld::cucumber()
        .max_concurrent_scenarios(Some(1))
        .fail_on_skipped()
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                let dir = TempDir::with_prefix("kid-e2e-")
                    .expect("failed to create temp dir");
                // Don't switch_dir yet — the Given step writes task
                // files first, then switches so the server loads them
                // with fully controlled timestamps.
                world.tasks_dir = Some(dir);
            })
        })
        .after(|_feature, _rule, _scenario, _finished, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    world
                        .admin
                        .post(format!("http://{TEST_CONTROL_ADDR}/reset-time-offset"))
                        .send()
                        .await
                        .and_then(reqwest::Response::error_for_status)
                        .expect("reset_time_offset failed");
                    let cwd = env::current_dir().expect("CWD available");
                    world
                        .admin
                        .post(format!("http://{TEST_CONTROL_ADDR}/switch-dir"))
                        .json(&serde_json::json!({ "dir": cwd }))
                        .send()
                        .await
                        .and_then(reqwest::Response::error_for_status)
                        .expect("switch_dir failed");
                    world.tasks_dir = None;
                }
            })
        })
        .run_and_exit("./features/browser")
        .await;

    Ok(())
}
