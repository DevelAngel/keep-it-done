use anyhow::Result;
use cucumber::given;
use tarpc::context;

use crate::world::AppWorld;

#[given("no tasks at all")]
async fn empty_task_list(world: &mut AppWorld) -> Result<()> {
    let count = world.rpc.count(context::current()).await?;
    assert_eq!(count, 0);
    Ok(())
}
