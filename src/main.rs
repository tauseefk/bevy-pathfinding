mod components;

mod prelude {
    pub use std::ops::Not;

    pub use bevy::prelude::*;
    pub use bevy_ecs_ldtk::prelude::*;
    pub use pathfinding::prelude::*;

    pub use crate::components::*;
}

const GRID_SIZE: i32 = 8;
const GRID_BLOCK_SIZE: i32 = 32;
const WINDOW_HEIGHT: i32 = 256;
const WINDOW_WIDTH: i32 = 256;

use prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::GRAY))
        .insert_resource(WindowDescriptor {
            title: "Pathfinding".to_string(),
            width: (WINDOW_WIDTH) as f32,
            height: (WINDOW_HEIGHT) as f32,
            resizable: false,
            ..Default::default()
        })
        .add_event::<ToggleBlockEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_startup_system(setup)
        .insert_resource(LevelSelection::Index(0))
        .add_system_to_stage(CoreStage::PostUpdate, grid_to_transform)
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::ChestBundle>("Chest")
        .add_system(mouse_click_system)
        .add_system(toggle_block)
        .add_system(pathfinding)
        .add_system(bevy::window::close_on_esc);

    app.run();
}

#[derive(Component, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct Pos {
    x: i32,
    y: i32,
}
impl Pos {
    const fn try_new(x: i32, y: i32) -> Option<Self> {
        if x <= 0 || y <= 0 || x > GRID_SIZE as i32 || y > GRID_SIZE as i32 {
            None
        } else {
            Some(Self {
                x: x as i32,
                y: y as i32,
            })
        }
    }

    const fn min(self) -> bool {
        self.x == 1 && self.y == 1
    }

    const fn max(self) -> bool {
        self.x == GRID_SIZE && self.y == GRID_SIZE
    }
}

struct ToggleBlockEvent {
    pos: Pos,
}

#[derive(Component)]
struct Path;

const YELLOW: Color = Color::hsl(53.0, 0.99, 0.50);
const PALE: Color = Color::hsl(237.0, 0.45, 0.9);
const BLUE: Color = Color::hsl(232.0, 0.62, 0.57);
const WHITE: Color = Color::hsl(0., 0., 1.);
const BLACK: Color = Color::hsl(0., 0., 0.);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(GRID_BLOCK_SIZE as f32, GRID_BLOCK_SIZE as f32)),
                color: WHITE,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Pos::try_new(1, 1).unwrap())
        .insert(Player);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(GRID_BLOCK_SIZE as f32, GRID_BLOCK_SIZE as f32)),
                color: YELLOW,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Pos::try_new(8, 8).unwrap())
        .insert(Chest);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            color: BLACK,
            ..Default::default()
        },
        ..Default::default()
    });

    // let ldtk_handle = asset_server.load("basic_map.ldtk");
    // commands.spawn_bundle(LdtkWorldBundle {
    //     ldtk_handle,
    //     ..Default::default()
    // });
}

fn grid_to_transform(mut query: Query<(&Pos, &mut Transform)>) {
    query.for_each_mut(|(pos, mut transform): (&Pos, Mut<Transform>)| {
        transform.translation.x =
            ((pos.x as i32 * GRID_BLOCK_SIZE) - (WINDOW_WIDTH + GRID_BLOCK_SIZE) / 2) as f32;
        transform.translation.y =
            ((pos.y as i32 * GRID_BLOCK_SIZE) - (WINDOW_HEIGHT + GRID_BLOCK_SIZE) / 2) as f32;
        transform.translation.z = 2.;
    });
}

fn mouse_click_system(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut my_events: EventWriter<ToggleBlockEvent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(window) = windows.get_primary() {
            if let Some(cursor_pos) = window.cursor_position() {
                let x = (cursor_pos.x as i32) / GRID_BLOCK_SIZE + 1;
                let y = (cursor_pos.y as i32) / GRID_BLOCK_SIZE + 1;

                if let Some(pos) = Pos::try_new(x as i32, y as i32) {
                    my_events.send(ToggleBlockEvent { pos });
                }
            }
        }
    }
}

fn toggle_block(
    mut my_events: EventReader<ToggleBlockEvent>,
    blocks: Query<(Entity, &Pos), With<Wall>>,
    mut commands: Commands,
) {
    for event in my_events.iter() {
        let event: &ToggleBlockEvent = event;
        if event.pos.min() || event.pos.max() {
            continue;
        }
        match blocks.iter().find(|(_, pos)| pos == &&event.pos) {
            None => {
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(
                                GRID_BLOCK_SIZE as f32,
                                GRID_BLOCK_SIZE as f32,
                            )),
                            color: PALE,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(event.pos)
                    .insert(Wall);
            }
            Some((entity, _)) => {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Pathfinding logic
/// find shortest path between Start and End
fn pathfinding(
    start: Query<&Pos, With<Player>>,
    end: Query<&Pos, With<Chest>>,
    blocks: Query<&Pos, With<Wall>>,
    paths: Query<Entity, With<Path>>,
    mut commands: Commands,
) {
    if start.get_single().is_err() || end.get_single().is_err() {
        return;
    }

    let start = start.get_single().expect("No start block");
    let end = end.get_single().expect("No end block");

    let blocks = blocks.iter().collect::<Vec<_>>();

    let result = bfs(
        start,
        |p| {
            let &Pos { x, y } = p;
            vec![(x, y - 1), (x, y + 1), (x - 1, y), (x + 1, y)]
                .into_iter()
                .filter_map(|(x, y)| Pos::try_new(x, y))
                .filter(|pos| blocks.contains(&pos).not())
        },
        |p| p == end,
    );

    for entity in paths.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if let Some(path) = result {
        for pos in path {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        color: BLUE,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(pos)
                .insert(Path);
        }
    }
}
