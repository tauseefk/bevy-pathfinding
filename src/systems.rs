use crate::prelude::*;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

pub fn mouse_click_system(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut toggle_wall: EventWriter<ToggleWallEvent>,
    mut move_chest: EventWriter<MoveChestEvent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(window) = windows.get_primary() {
            if let Some(cursor_pos) = window.cursor_position() {
                toggle_wall.send(ToggleWallEvent {
                    translation: snap_to_grid(Vec3::new(cursor_pos.x, cursor_pos.y, 1.)),
                });
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        if let Some(window) = windows.get_primary() {
            if let Some(cursor_pos) = window.cursor_position() {
                move_chest.send(MoveChestEvent {
                    translation: snap_to_grid(Vec3::new(cursor_pos.x, cursor_pos.y, 1.)),
                });
            }
        }
    }
}

pub fn toggle_wall(
    mut my_events: EventReader<ToggleWallEvent>,
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
        let event: &ToggleWallEvent = event;
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

pub fn move_chest(
    mut my_events: EventReader<MoveChestEvent>,
    mut chest_query: Query<&mut Transform, (With<Chest>, Without<Wall>)>,
    wall_blocks: Query<&Transform, (With<Wall>, Without<Chest>)>,
) {
    for event in my_events.iter() {
        match wall_blocks.iter().find(|transform| {
            translation_to_grid_pos(transform.translation).unwrap()
                == translation_to_grid_pos(event.translation).unwrap()
        }) {
            None => {
                let mut chest = chest_query.single_mut();
                chest.translation.x = event.translation.x;
                chest.translation.y = event.translation.y;
            }
            Some(_) => {}
        }
    }
}

/// Pathfinding logic
/// find shortest path between Start and End
pub fn pathfinding(
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

pub fn path_traversal(
    time: Res<Time>,
    mut timer: ResMut<MovementTimer>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Path>)>,
    path_query: Query<&Transform, (With<Path>, Without<Player>)>,
) {
    let mut player = player_query.single_mut();

    if let Some(path_block) = path_query.iter().nth(1) {
        if timer.0.tick(time.delta()).just_finished() {
            player.translation.x = path_block.translation.x;
            player.translation.y = path_block.translation.y;
        }
    }
}
