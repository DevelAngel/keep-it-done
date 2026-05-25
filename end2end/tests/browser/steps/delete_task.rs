use anyhow::{bail, Result};
use cucumber::{then, when};
use thirtyfour::prelude::*;

use crate::world::AppWorld;

const TASK_SELECTOR: &str = "[data-testid^='task-']";

/// Find the task element whose visible text contains `summary`.
async fn find_task(parent: &WebElement, summary: &str) -> Result<WebElement> {
    let candidates = parent.find_all(By::Css(TASK_SELECTOR)).await?;
    for el in candidates {
        let text = el.text().await?;
        if text.contains(summary) {
            return Ok(el);
        }
    }
    bail!("task \"{summary}\" not found");
}

#[when(expr = "I expand the task {string}")]
async fn expand_task(world: &mut AppWorld, summary: String) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    let task_el = find_task(&body, &summary).await?;
    task_el.click().await?;

    // Wait for the detail panel to appear.
    world
        .http
        .query(By::Testid("task-details"))
        .first()
        .await?;

    Ok(())
}

#[when("I tap the delete button")]
async fn tap_delete_button(world: &mut AppWorld) -> Result<()> {
    let button = world
        .http
        .query(By::Testid("delete-task-button"))
        .first()
        .await?;
    button.click().await?;

    // Brief wait for state transition.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    Ok(())
}

#[when("I confirm the deletion")]
async fn confirm_deletion(world: &mut AppWorld) -> Result<()> {
    let button = world
        .http
        .query(By::Testid("delete-task-button"))
        .first()
        .await?;
    button.click().await?;

    // Wait for the server round-trip and list refresh.
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    Ok(())
}

#[when("I wait for the disarm timeout")]
async fn wait_for_disarm(world: &mut AppWorld) -> Result<()> {
    let _ = world; // keep signature uniform
    // The auto-disarm fires after 3 s; wait a bit beyond that.
    tokio::time::sleep(std::time::Duration::from_millis(3500)).await;
    Ok(())
}

#[then("the task details are collapsed")]
async fn details_are_collapsed(world: &mut AppWorld) -> Result<()> {
    let result = world
        .http
        .query(By::Testid("task-details"))
        .nowait()
        .first_opt()
        .await?;
    assert!(result.is_none(), "expected task details to be collapsed");
    Ok(())
}

#[then("the delete button shows armed state")]
async fn delete_button_is_armed(world: &mut AppWorld) -> Result<()> {
    let button = world
        .http
        .query(By::Testid("delete-task-button"))
        .first()
        .await?;
    let text = button.text().await?;
    assert!(
        text.contains("Confirm"),
        "expected armed state ('Confirm delete'), got: {text}"
    );
    Ok(())
}

#[then("the delete button shows idle state")]
async fn delete_button_is_idle(world: &mut AppWorld) -> Result<()> {
    let button = world
        .http
        .query(By::Testid("delete-task-button"))
        .first()
        .await?;
    let text = button.text().await?;
    assert!(
        text.contains("Delete"),
        "expected idle state ('Delete'), got: {text}"
    );
    // Verify it does NOT say "Confirm" (i.e. disarmed back to idle).
    assert!(
        !text.contains("Confirm"),
        "expected idle state, but button still shows armed text: {text}"
    );
    Ok(())
}

#[then(expr = "I still see the task {string}")]
async fn task_still_visible(world: &mut AppWorld, summary: String) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    find_task(&body, &summary).await.unwrap_or_else(|_| {
        panic!("expected task \"{summary}\" to still be visible")
    });
    Ok(())
}

#[then(expr = "only {int} task files remain on disk")]
async fn task_files_on_disk(world: &mut AppWorld, expected: usize) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    let count = std::fs::read_dir(dir.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("task-") && name.ends_with(".json"))
        })
        .count();
    assert_eq!(
        count, expected,
        "expected {expected} task file(s) on disk, found {count}"
    );
    Ok(())
}
