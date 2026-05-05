mod fixtures;

use anyhow::Result;
use cucumber::World;
use fixtures::world::AppWorld;

#[tokio::main]
async fn main() -> Result<()> {
    AppWorld::cucumber()
        .max_concurrent_scenarios(Some(1))
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.open_session().await.expect("open geckodriver session");
            })
        })
        .after(|_feature, _rule, _scenario, _ev, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    world.close_session().await;
                }
            })
        })
        .fail_on_skipped()
        .run_and_exit("./features")
        .await;
    Ok(())
}
