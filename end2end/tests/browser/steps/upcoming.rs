use anyhow::{bail, Result};
use cucumber::gherkin::Step;
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

#[then("I see the Upcoming groups")]
async fn verify_upcoming_groups(world: &mut AppWorld, step: &Step) -> Result<()> {
    let table = step.table.as_ref().expect("data table required");
    let headers = &table.rows[0];
    let group_col = headers.iter().position(|h| h == "group").expect("group column");
    let summary_col = headers.iter().position(|h| h == "summary").expect("summary column");

    for row in &table.rows[1..] {
        let group = row[group_col].trim();
        let summary = row[summary_col].trim();

        let group_el = world
            .http
            .query(By::Testid(format!("upcoming-group-{group}")))
            .first()
            .await?;

        find_task(&group_el, summary).await.unwrap_or_else(|_| {
            panic!("expected task \"{summary}\" in group \"{group}\"")
        });
    }

    Ok(())
}

#[then(expr = "the task {string} has an attention label")]
async fn task_has_attention_label(world: &mut AppWorld, summary: String) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    let task_el = find_task(&body, &summary).await?;

    let label = task_el
        .find_all(By::Testid("attention-label"))
        .await?;

    assert!(
        !label.is_empty(),
        "expected task \"{summary}\" to have an attention label"
    );

    Ok(())
}

#[then(expr = "the task {string} has no attention label")]
async fn task_has_no_attention_label(world: &mut AppWorld, summary: String) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    let task_el = find_task(&body, &summary).await?;

    let labels = task_el
        .find_all(By::Testid("attention-label"))
        .await?;

    assert!(
        labels.is_empty(),
        "expected task \"{summary}\" to have no attention label, but found one"
    );

    Ok(())
}

#[then(expr = "the task {string} appears before {string}")]
async fn task_appears_before(
    world: &mut AppWorld,
    first: String,
    second: String,
) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    let el_a = find_task(&body, &first).await?;
    let el_b = find_task(&body, &second).await?;

    let rect_a = el_a.rect().await?;
    let rect_b = el_b.rect().await?;

    assert!(
        rect_a.y < rect_b.y,
        "expected \"{first}\" (y={}) before \"{second}\" (y={})",
        rect_a.y,
        rect_b.y
    );

    Ok(())
}

#[when("I expand the backlog")]
async fn expand_backlog(world: &mut AppWorld) -> Result<()> {
    let backlog = world
        .http
        .query(By::Testid("backlog"))
        .first()
        .await?;

    let toggle = backlog.find(By::Tag("button")).await?;
    toggle.click().await?;

    // Wait for task items to render after expansion.
    backlog.query(By::Css(TASK_SELECTOR)).first().await?;

    Ok(())
}

#[then(expr = "the task {string} is in the backlog")]
async fn task_is_in_backlog(world: &mut AppWorld, summary: String) -> Result<()> {
    let backlog = world
        .http
        .query(By::Testid("backlog"))
        .first()
        .await?;

    find_task(&backlog, &summary).await.unwrap_or_else(|_| {
        panic!("expected task \"{summary}\" in backlog")
    });

    Ok(())
}

#[then(expr = "I do not see the task {string}")]
async fn task_not_visible(world: &mut AppWorld, summary: String) -> Result<()> {
    let body = world.http.find(By::Tag("body")).await?;
    let result = find_task(&body, &summary).await;

    assert!(
        result.is_err(),
        "expected task \"{summary}\" to not be present, but it was found"
    );

    Ok(())
}
