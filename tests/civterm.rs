mod world;

use cucumber::World as _;

#[tokio::main]
async fn main() {
    world::CivWorld::run("tests/features").await;
}
