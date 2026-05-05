use crate::fixtures::{check, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::then;

#[then(regex = r"^I see the page title is (.+)$")]
async fn i_see_the_page_title_is(
    world: &mut AppWorld,
    text: String,
) -> Result<()> {
    check::page_title(&world.client(), &text).await?;
    Ok(())
}

#[then("I see at least one task")]
async fn i_see_at_least_one_task(world: &mut AppWorld) -> Result<()> {
    check::has_tasks(&world.client()).await?;
    Ok(())
}

#[then("I see a task checkbox")]
async fn i_see_a_task_checkbox(world: &mut AppWorld) -> Result<()> {
    check::has_task_checkbox(&world.client()).await?;
    Ok(())
}
