use anyhow::Result;
use cucumber::{then, when};
use thirtyfour::prelude::*;

use crate::helpers::ViewSwitch;
use crate::screenshots;
use crate::world::AppWorld;

#[when("I open the app")]
async fn open_webapp(world: &mut AppWorld) -> Result<()> {
    world.http.goto("http://localhost:3000/").await?;
    Ok(())
}

#[when(expr = "I click the {word} view arrow")]
async fn switch_view_by_arrow(world: &mut AppWorld, dir: ViewSwitch) -> Result<()> {
    let header = world.http.find(By::Tag("header")).await?;
    let button = header
        .find(By::Testid(format!("{dir}-view-arrow")))
        .await?;
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

#[then(expr = "I save a screenshot as {string}")]
async fn save_screenshot(world: &mut AppWorld, name: String) -> Result<()> {
    screenshots::save_screenshot(&world.http, &name).await;
    Ok(())
}
