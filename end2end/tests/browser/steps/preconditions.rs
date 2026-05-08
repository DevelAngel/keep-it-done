use anyhow::Result;
use cucumber::gherkin::Step;
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

#[given("the following tasks")]
async fn create_tasks(world: &mut AppWorld, step: &Step) -> Result<()> {
    let table = step.table.as_ref().expect("data table required");
    let headers = &table.rows[0];

    for row in &table.rows[1..] {
        let col = |name: &str| -> Option<&str> {
            let idx = headers.iter().position(|h| h == name)
                .unwrap_or_else(|| panic!("missing column: {name}"));
            let val = row[idx].trim();
            if val.is_empty() { None } else { Some(val) }
        };

        let summary = col("summary").expect("summary required");
        let category = col("category").expect("category required");
        let status = col("status").expect("status required");
        let days_ago: u64 = col("days ago").expect("days ago required")
            .parse().expect("days ago must be a number");

        match status {
            "open" => seeds::add_open(
                &world.rpc,
                summary,
                category,
                col("context").unwrap_or(""),
                col("estimate"),
                col("priority"),
                col("note"),
                days_ago,
            ).await,
            "done" => seeds::add_done(
                &world.rpc,
                summary,
                category,
                days_ago,
            ).await,
            other => panic!("unknown status: {other}"),
        }
    }

    Ok(())
}
