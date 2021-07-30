#![feature(iter_intersperse)]

mod bundles;
mod dungeon_crawl;
mod world_generation;
mod world_map;

use bevy::prelude::*;
use dungeon_crawl::TurnState;
use world_map::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    WorldGeneration,
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
        .add_state(AppState::WorldGeneration);

    app.add_startup_system(ui_setup.system());

    app.run();
}

pub mod ui {
    pub struct Camera;
    pub struct HpText;
    pub struct HpBar;
    pub struct Log;
    pub struct Details;
}

fn ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(ui::Camera);
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(300.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::hex("101010").unwrap().into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "HP: 5/5",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 25.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(ui::HpText);

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(90.0), Val::Px(20.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::hex("DA0037").unwrap().into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: materials.add(Color::hex("43ad39").unwrap().into()),
                            ..Default::default()
                        })
                        .insert(ui::HpBar);
                });

            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(5.0)),
                    margin: Rect {
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            });

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(15.0),
                            right: Val::Px(15.0),
                            top: Val::Px(10.0),
                            bottom: Val::Px(10.0),
                        },
                        align_self: AlignSelf::FlexStart,
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "Log\nLog\nLog\nLog\nLog\nLog",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(ui::Log);

            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(5.0)),
                    margin: Rect {
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            });

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(15.0),
                            right: Val::Px(15.0),
                            top: Val::Px(10.0),
                            bottom: Val::Px(10.0),
                        },
                        align_self: AlignSelf::FlexStart,
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "a\nb\nc\nd",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(ui::Details);

            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(5.0)),
                    margin: Rect {
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            });
        });
}
