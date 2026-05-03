use crate::fixtures::{action, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::{given, when};

#[given("I see the app")]
#[when("I open the app")]
async fn i_open_the_app(world: &mut AppWorld) -> Result<()> {
    action::goto_path(&world.client, "/").await?;
    Ok(())
}

#[when(regex = r"^I open the view (.+)$")]
async fn i_open_the_view(world: &mut AppWorld, view: String) -> Result<()> {
    action::goto_view(&world.client, &view).await?;
    Ok(())
}

#[when("I click the next view arrow")]
async fn i_click_next_view_arrow(world: &mut AppWorld) -> Result<()> {
    action::click_next_view_arrow(&world.client).await?;
    Ok(())
}
