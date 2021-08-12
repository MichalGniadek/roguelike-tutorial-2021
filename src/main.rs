#![feature(iter_intersperse)]
#![feature(option_result_contains)]

mod bundles;
mod dungeon_crawl;
mod world_generation;
mod world_map;

use bevy::{app::AppExit, prelude::*};
use dungeon_crawl::TurnState;
use world_map::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    MainMenu,
    WorldGeneration,
    DungeonCrawlEnter,
    DungeonCrawl(TurnState),
    DungeonCrawlExitToMenu,
    DungeonCrawlDescend,
}

#[cfg_attr(target_arch = "wasm32", global_allocator)]
#[cfg(target_arch = "wasm32")]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct UiCamera;

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

            commands
                .spawn_bundle(UiCameraBundle::default())
                .insert(UiCamera);
        })
        .system(),
    );

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.add_state(AppState::MainMenu)
        .add_system_set(
            SystemSet::on_enter(AppState::MainMenu).with_system(main_menu_ui_create.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu).with_system(main_menu_interaction.system()),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MainMenu).with_system(main_menu_cleanup.system()),
        )
        .add_plugin(dungeon_crawl::DungeonCrawlPlugin)
        .add_plugins(world_generation::WorldGenerationPlugins);

    app.run();
}

pub struct MainMenuCanvas;
pub enum MainMenuButton {
    Play,
    Quit,
}

pub fn main_menu_interaction(
    q: Query<(&Interaction, &MainMenuButton)>,
    mut app_state: ResMut<State<AppState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (i, b) in q.iter() {
        match (i, b) {
            (Interaction::Clicked, MainMenuButton::Play) => {
                app_state.set(AppState::WorldGeneration).unwrap();
            }
            (Interaction::Clicked, MainMenuButton::Quit) => app_exit_events.send(AppExit),
            _ => {}
        }
    }
}

pub fn main_menu_ui_create(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::hex("101010").unwrap().into()),
            ..Default::default()
        })
        .insert(MainMenuCanvas)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(50.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::hex("101010").unwrap().into()),
                    ..Default::default()
                })
                .insert(MainMenuButton::Play)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "PLAY",
                            TextStyle {
                                font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                                font_size: 100.0,
                                color: Color::WHITE,
                            },
                            TextAlignment::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(50.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::hex("101010").unwrap().into()),
                    ..Default::default()
                })
                .insert(MainMenuButton::Quit)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "QUIT",
                            TextStyle {
                                font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                                font_size: 100.0,
                                color: Color::WHITE,
                            },
                            TextAlignment::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

pub fn main_menu_cleanup(mut commands: Commands, q: Query<Entity, With<MainMenuCanvas>>) {
    commands.entity(q.single().unwrap()).despawn_recursive();
}
