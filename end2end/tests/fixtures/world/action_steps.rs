use crate::fixtures::{action, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::{given, then, when};

#[given("I see the app")]
#[when("I open the app")]
async fn i_open_the_app(world: &mut AppWorld) -> Result<()> {
    action::goto_path(&world.client(), "/").await?;
    Ok(())
}

#[when(regex = r"^I open the view (\S+) with expand first$")]
async fn i_open_view_expand_first(world: &mut AppWorld, view: String) -> Result<()> {
    action::goto_view_expand_first(&world.client(), &view).await?;
    Ok(())
}

#[when(regex = r"^I open the view (\S+)$")]
async fn i_open_the_view(world: &mut AppWorld, view: String) -> Result<()> {
    action::goto_view(&world.client(), &view).await?;
    Ok(())
}

#[when("I click the next view arrow")]
async fn i_click_next_view_arrow(world: &mut AppWorld) -> Result<()> {
    action::click_next_view_arrow(&world.client()).await?;
    Ok(())
}

#[when(regex = r"^I save a screenshot as (.+)$")]
#[then(regex = r"^I save a screenshot as (.+)$")]
async fn i_save_screenshot(world: &mut AppWorld, filename: String) -> Result<()> {
    action::save_screenshot(&world.client(), &filename).await?;
    Ok(())
}
