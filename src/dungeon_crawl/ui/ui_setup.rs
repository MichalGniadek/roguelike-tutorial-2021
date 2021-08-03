use super::{
    MyCanvas, MyDetails, MyFloorText, MyHpBar, MyHpText, MyInventory, MyLog, MyXPBar, MyXPText,
};
use crate::{dungeon_crawl::Cursor, world_map::GridPosition};
use bevy::prelude::*;

pub fn create(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("convergence-target.png")),
                color: Color::hex("EDEDED").unwrap(),
            }),
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..Default::default()
        })
        .insert_bundle((GridPosition { x: 10, y: 10 }, Cursor));

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
        .insert(MyCanvas)
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
                .insert(MyHpText);

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
                        .insert(MyHpBar);
                });

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "XP: 0/3",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 25.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(MyXPText);

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(90.0), Val::Px(20.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::hex("171717").unwrap().into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: materials.add(Color::hex("826007").unwrap().into()),
                            ..Default::default()
                        })
                        .insert(MyXPBar);
                });

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "Floor 0",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(MyFloorText);

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
                .insert(MyLog);

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
                .insert(MyDetails);

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
                        "a\nb\nc\nd\ne",
                        TextStyle {
                            font: asset_server.load("Roboto/Roboto-Regular.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(MyInventory);
        });
}
