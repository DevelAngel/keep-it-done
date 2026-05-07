use anyhow::Result;
use cucumber::{given, then, when, World};
use strum::{Display, EnumString};
use thirtyfour::prelude::*;

use std::ops::Deref;


#[derive(Debug, World)]
#[world(init = Self::new)] 
pub struct AppWorld {
    driver: WebDriver,
}

impl Deref for AppWorld {
    type Target = WebDriver;

    fn deref(&self) -> &Self::Target {
        &self.driver
    }
}

impl AppWorld {
    async fn new() -> Result<Self> {
        let mut capa = DesiredCapabilities::chrome();
        capa.set_headless()?;
        let driver = WebDriver::managed(capa).await?;
        Ok(Self { driver })
    }
}

#[when("I open the app")]
async fn open_webapp(world: &mut AppWorld) -> Result<()> {
    world.goto("http://localhost:3000/").await?;
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
    let header = world.find(By::Tag("header")).await?;
    let button = header.find(By::Testid(format!("{dir}-view-arrow"))).await?;
    button.click().await?;
    Ok(())
}

#[then(expr = "I see the page title is {}")]
async fn validate_view_title(
    world: &mut AppWorld,
    expected_title: String,
) -> Result<()> {
    let header = world.find(By::Tag("header")).await?;
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
        .run_and_exit("./features/webapp")
        .await;

    Ok(())
}
