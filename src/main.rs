mod components;

mod prelude {
    pub use std::ops::Not;

    pub use bevy::prelude::*;
    pub use bevy_ecs_ldtk::prelude::*;
    pub use pathfinding::prelude::*;

    pub use crate::components::*;

    pub const GRID_SIZE: i32 = 8;
    pub const GRID_BLOCK_SIZE: i32 = 32;
    pub const WINDOW_HEIGHT: i32 = 256;
    pub const WINDOW_WIDTH: i32 = 256;
}

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
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::ChestBundle>("Chest")
        .add_system(mouse_click_system)
        .add_system(toggle_block)
        .add_system(pathfinding)
        .add_system(bevy::window::close_on_esc);

    app.run();
}

struct ToggleBlockEvent {
    translation: Vec3,
}

const YELLOW: Color = Color::hsl(53.0, 0.99, 0.50);
const PALE: Color = Color::hsl(237.0, 0.45, 0.9);
const BLUE: Color = Color::hsl(232.0, 0.62, 0.57);
const WHITE: Color = Color::hsl(0., 0., 1.);
const BLACK: Color = Color::hsl(0., 0., 0.);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new((WINDOW_WIDTH / 2) as f32, (WINDOW_HEIGHT / 2) as f32, 3.),
            ..Default::default()
        },
        ..Default::default()
    });

    let ldtk_handle = asset_server.load("basic_map.ldtk");
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

fn grid_to_translation(grid_pos: GridPosition) -> Vec3 {
    Vec3::new(
        (grid_pos.x as i32 * GRID_BLOCK_SIZE - GRID_BLOCK_SIZE / 2) as f32,
        (grid_pos.y as i32 * GRID_BLOCK_SIZE - GRID_BLOCK_SIZE / 2) as f32,
        2.,
    )
}

fn translation_to_grid_pos(translation: Vec3) -> Option<GridPosition> {
    let x = (translation.x as i32) / GRID_BLOCK_SIZE + 1;
    let y = (translation.y as i32) / GRID_BLOCK_SIZE + 1;

    GridPosition::try_new(x, y)
}

fn snap_to_grid(translation: Vec3) -> Vec3 {
    grid_to_translation(translation_to_grid_pos(translation).unwrap())
}

fn mouse_click_system(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut my_events: EventWriter<ToggleBlockEvent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(window) = windows.get_primary() {
            if let Some(cursor_pos) = window.cursor_position() {
                my_events.send(ToggleBlockEvent {
                    translation: snap_to_grid(Vec3::new(cursor_pos.x, cursor_pos.y, 1.)),
                });
            }
        }
    }
}

fn toggle_block(
    mut my_events: EventReader<ToggleBlockEvent>,
    blocks: Query<(Entity, &Transform), With<Wall>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("wall.PNG");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(GRID_BLOCK_SIZE as f32, GRID_BLOCK_SIZE as f32),
        8,
        1,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for event in my_events.iter() {
        let event: &ToggleBlockEvent = event;
        match blocks.iter().find(|(_, transform)| {
            translation_to_grid_pos(transform.translation).unwrap()
                == translation_to_grid_pos(event.translation).unwrap()
        }) {
            None => {
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle.clone(),
                        transform: Transform {
                            translation: event.translation,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
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
    start: Query<&Transform, With<Player>>,
    end: Query<&Transform, With<Chest>>,
    blocks: Query<&Transform, With<Wall>>,
    paths: Query<Entity, With<Path>>,
    mut commands: Commands,
) {
    if start.get_single().is_err() || end.get_single().is_err() {
        return;
    }

    let start = start.get_single().expect("No start block");
    let end = end.get_single().expect("No end block");

    let start_grid_pos = translation_to_grid_pos(start.translation).unwrap();
    let end_grid_pos = translation_to_grid_pos(end.translation).unwrap();

    let blocks = blocks
        .iter()
        .map(|block| translation_to_grid_pos(block.translation).unwrap())
        .collect::<Vec<_>>();

    let result = bfs(
        &start_grid_pos,
        |p| {
            let &GridPosition { x, y } = p;
            vec![(x, y - 1), (x, y + 1), (x - 1, y), (x + 1, y)]
                .into_iter()
                .filter_map(|(x, y)| GridPosition::try_new(x, y))
                .filter(|grid_pos| blocks.contains(&grid_pos).not())
        },
        |p| *p == end_grid_pos,
    );

    for entity in paths.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if let Some(path) = result {
        for grid_pos in path {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(4.0, 4.0)),
                        color: BLUE,
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: grid_to_translation(grid_pos),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Path);
        }
    }
}
