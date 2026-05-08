use anyhow::Result;
use cucumber::given;
use tarpc::context;

use crate::seeds;
use crate::world::AppWorld;

#[given("no tasks at all")]
async fn empty_task_list(world: &mut AppWorld) -> Result<()> {
    let count = world.rpc.count(context::current()).await?;
    assert_eq!(count, 0);
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} created {int} days ago")]
async fn open_task(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, None, None, None, days_ago).await;
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} estimate {string} created {int} days ago")]
async fn open_task_estimate(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    estimate: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, Some(&estimate), None, None, days_ago).await;
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} priority {word} created {int} days ago")]
async fn open_task_priority(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    priority: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, None, Some(&priority), None, days_ago).await;
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} estimate {string} note {string} created {int} days ago")]
async fn open_task_estimate_note(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    estimate: String,
    note: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, Some(&estimate), None, Some(&note), days_ago).await;
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} estimate {string} priority {word} note {string} created {int} days ago")]
async fn open_task_estimate_priority_note(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    estimate: String,
    priority: String,
    note: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, Some(&estimate), Some(&priority), Some(&note), days_ago).await;
    Ok(())
}

#[given(expr = "task {string} in {string} at {string} estimate {string} priority {word} created {int} days ago")]
async fn open_task_estimate_priority(
    world: &mut AppWorld,
    summary: String,
    category: String,
    context: String,
    estimate: String,
    priority: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_open(&world.rpc, &summary, &category, &context, Some(&estimate), Some(&priority), None, days_ago).await;
    Ok(())
}

#[given(expr = "completed task {string} in {string} created {int} days ago")]
async fn completed_task(
    world: &mut AppWorld,
    summary: String,
    category: String,
    days_ago: u64,
) -> Result<()> {
    seeds::add_done(&world.rpc, &summary, &category, days_ago).await;
    Ok(())
}
