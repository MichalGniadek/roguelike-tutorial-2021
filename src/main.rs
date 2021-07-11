mod dungeon_crawl;
mod world_generation;
mod world_map;

use bevy::prelude::*;
use world_generation::WorldGeneratorType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    WorldGeneration(WorldGeneratorType),
    DungeonCrawl,
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::hex("171717").unwrap()))
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_plugin(dungeon_crawl::DungeonCrawlPlugin)
        .add_plugins(world_generation::WorldGenerationPlugins)
        .add_state(AppState::WorldGeneration(WorldGeneratorType::CellularAutomata))
        .run();
}
