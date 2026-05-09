use anyhow::Result;
use cucumber::{then, when};
use thirtyfour::prelude::*;

use std::path::PathBuf;

use crate::helpers::{ViewName, ViewSwitch};
use crate::screenshots;
use crate::world::AppWorld;

#[when("I open the app")]
async fn open_webapp(world: &mut AppWorld) -> Result<()> {
    world.http.goto("http://localhost:3000/").await?;
    Ok(())
}

#[when(expr = "I open the app in {} view")]
async fn open_webapp_in_view(world: &mut AppWorld, view: ViewName) -> Result<()> {
    let view = view.url_param();
    world
        .http
        .goto(format!("http://localhost:3000/?view={view}"))
        .await?;
    Ok(())
}

#[when(expr = "I open the app in {} view with expanded details")]
async fn open_webapp_in_view_expanded(world: &mut AppWorld, view: ViewName) -> Result<()> {
    let view = view.url_param();
    world
        .http
        .goto(format!("http://localhost:3000/?view={view}&expand=first"))
        .await?;
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

#[then("I see tasks in the list")]
async fn wait_for_tasks(world: &mut AppWorld) -> Result<()> {
    // tasks have a checkbox toggle
    world.http
        .query(By::Css("input[type='checkbox']"))
        .first()
        .await?;
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

#[then(expr = "I save a screenshot for the {} view")]
async fn save_view_screenshot(world: &mut AppWorld, view: ViewName) -> Result<()> {
    screenshots::save_screenshot(&world.http, &view.screenshot_file()).await;
    Ok(())
}

#[then(expr = "I save a screenshot as {string}")]
async fn save_named_screenshot(world: &mut AppWorld, mut name: PathBuf) -> Result<()> {
    name.set_extension("png");
    screenshots::save_screenshot(&world.http, &name).await;
    Ok(())
}
