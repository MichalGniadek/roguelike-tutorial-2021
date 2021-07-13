mod dungeon_crawl;
mod world_generation;
mod world_map;

use bevy::prelude::*;
use dungeon_crawl::TurnState;
use world_generation::WorldGeneratorType;
use world_map::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    WorldGeneration(WorldGeneratorType),
    DungeonCrawl(TurnState),
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::build();
    app.insert_resource(ClearColor(Color::hex("171717").unwrap()));
    app.insert_resource(WindowDescriptor {
        title: String::from("Roguelike"),
        #[cfg(target_arch = "wasm32")]
        canvas: Some(String::from("#canv")),
        ..Default::default()
    });

    app.add_plugins(DefaultPlugins);

    app.insert_resource(Grid {
        cell_size: IVec2::new(512, 512),
    })
    .add_startup_system(
        (|mut commands: Commands| {
            let mut orto = OrthographicCameraBundle::new_2d();
            orto.orthographic_projection.scale = 8.0;
            commands.spawn_bundle(orto);
        })
        .system(),
    );

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.add_system(bevy::input::system::exit_on_esc_system.system())
        .add_plugin(dungeon_crawl::DungeonCrawlPlugin)
        .add_plugins(world_generation::WorldGenerationPlugins)
        .add_state(AppState::WorldGeneration(
            WorldGeneratorType::CellularAutomata,
        ));

    app.run();
}
