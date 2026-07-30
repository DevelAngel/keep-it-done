use anyhow::Result;
use cucumber::{then, when};
use rmcp::model::CallToolRequestParams;
use thirtyfour::prelude::*;

use std::fs;

use crate::helpers::TEST_CONTROL_ADDR;
use crate::world::AppWorld;

// -- Triggers -----------------------------------------------------

#[when(expr = "I add a task {string} via MCP")]
async fn add_task_via_mcp(world: &mut AppWorld, summary: String) -> Result<()> {
    world
        .mcp()
        .await?
        .call_tool(
            CallToolRequestParams::new("add").with_arguments(
                serde_json::json!({
                    "summary": summary,
                    "category": "Inbox",
                    "on_behalf_of": "e2e-flush",
                })
                .as_object()
                .cloned()
                .expect("object"),
            ),
        )
        .await?;
    Ok(())
}

#[when("I flush the task cache")]
async fn flush_task_cache(world: &mut AppWorld) -> Result<()> {
    world
        .admin
        .post(format!("http://{TEST_CONTROL_ADDR}/flush"))
        .send()
        .await?
        .error_for_status()?;
    // Give the SSE event time to arrive and the DOM to update.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

#[when("I wait for the LED to auto-dismiss")]
async fn wait_for_led_dismiss(_world: &mut AppWorld) -> Result<()> {
    // The LED auto-dismisses after 3 s; wait a bit beyond that.
    tokio::time::sleep(std::time::Duration::from_millis(3500)).await;
    Ok(())
}

#[when("I wait briefly for any event")]
async fn wait_briefly(_world: &mut AppWorld) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

#[when("I click the flush LED")]
async fn click_flush_led(world: &mut AppWorld) -> Result<()> {
    let led = world
        .http
        .query(By::Testid("flush-led-err"))
        .first()
        .await?;
    led.click().await?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    Ok(())
}

#[when("I make the tasks directory read-only")]
async fn make_dir_read_only(world: &mut AppWorld) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    let mut perms = fs::metadata(dir.path())?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(dir.path(), perms)?;
    Ok(())
}

#[when("I restore the tasks directory permissions")]
async fn restore_dir_permissions(world: &mut AppWorld) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    let mut perms = fs::metadata(dir.path())?.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(dir.path(), perms)?;
    Ok(())
}

// -- Assertions ---------------------------------------------------

#[then("the flush LED is hidden")]
async fn led_is_hidden(world: &mut AppWorld) -> Result<()> {
    let led = world
        .http
        .query(By::Testid("flush-led-hidden"))
        .nowait()
        .first_opt()
        .await?;
    assert!(led.is_some(), "expected flush LED to be in hidden state");
    Ok(())
}

#[then("the flush LED shows success")]
async fn led_shows_success(world: &mut AppWorld) -> Result<()> {
    let led = world
        .http
        .query(By::Testid("flush-led-ok"))
        .first()
        .await?;
    assert!(led.is_displayed().await?, "expected green LED to be visible");
    Ok(())
}

#[then("the flush LED shows error")]
async fn led_shows_error(world: &mut AppWorld) -> Result<()> {
    let led = world
        .http
        .query(By::Testid("flush-led-err"))
        .first()
        .await?;
    assert!(led.is_displayed().await?, "expected red LED to be visible");
    Ok(())
}

#[then("the flush error panel is visible")]
async fn error_panel_visible(world: &mut AppWorld) -> Result<()> {
    let panel = world
        .http
        .query(By::Testid("flush-error-panel"))
        .first()
        .await?;
    assert!(
        panel.is_displayed().await?,
        "expected flush error panel to be visible"
    );
    Ok(())
}

#[then("the flush error panel is not visible")]
async fn error_panel_not_visible(world: &mut AppWorld) -> Result<()> {
    let panel = world
        .http
        .query(By::Testid("flush-error-panel"))
        .nowait()
        .first_opt()
        .await?;
    assert!(
        panel.is_none(),
        "expected flush error panel to not be visible"
    );
    Ok(())
}
