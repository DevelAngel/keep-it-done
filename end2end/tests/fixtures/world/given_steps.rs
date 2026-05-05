use crate::fixtures::world::{AppWorld, HOST};
use anyhow::Result;
use cucumber::given;

/// Write the 8 standard task fixtures (relative dates) and
/// remove any leftover files from a previous scenario.
#[given("the standard task fixtures")]
async fn the_standard_task_fixtures(world: &mut AppWorld) -> Result<()> {
    let dir = world.tasks_dir();
    kid_end2end::clean_task_dir(dir)?;
    std::fs::create_dir_all(dir)?;
    kid_end2end::write_standard_fixtures(dir)?;
    Ok(())
}

/// Trigger the server to discard its in-memory cache and reload
/// all task files from `KID_TASKS_DIR`.
#[given("tasks are loaded")]
async fn tasks_are_loaded(_world: &mut AppWorld) -> Result<()> {
    reqwest::Client::new()
        .post(format!("{HOST}/api/reload_cache"))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
