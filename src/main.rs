use bevy::prelude::*;
use pathfinding::prelude::*;
use std::ops::Not;

const SIZE: i32 = 10;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::GRAY))
        .insert_resource(WindowDescriptor {
            title: "Pathfinding".to_string(),
            width: 800.,
            height: 600.,
            resizable: false,
            ..Default::default()
        })
        .add_event::<ToggleBlockEvent>()
        .init_resource::<Materials>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, grid_to_transform)
        .add_system(mouse_click_system)
        .add_system(toggle_block)
        .add_system(pathfinding);

    app.run();
}

#[derive(Component)]
struct Start;

#[derive(Component)]
struct End;

#[derive(Component)]
struct Block;

#[derive(Default)]
struct Materials {
    path: Option<Handle<ColorMaterial>>,
    block: Option<Handle<ColorMaterial>>,
}

#[derive(Component, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct Pos {
    x: i32,
    y: i32,
}
impl Pos {
    const fn try_new(x: i32, y: i32) -> Option<Self> {
        if x < 0 || y < 0 || x >= SIZE as i32 || y >= SIZE as i32 {
            None
        } else {
            Some(Self {
                x: x as i32,
                y: y as i32,
            })
        }
    }

    const fn min(self) -> bool {
        self.x == 0 && self.y == 0
    }

    const fn max(self) -> bool {
        self.x == SIZE - 1 && self.y == SIZE - 1
    }
}

struct ToggleBlockEvent {
    pos: Pos,
}

#[derive(Component)]
struct Path;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut my_materials: ResMut<Materials>,
) {
    my_materials.path = Some(materials.add(Color::rgb(1., 1., 1.).into()));
    my_materials.block = Some(materials.add(Color::rgb(0.5, 0.5, 1.0).into()));

    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.0, 1.0)),
                color: Color::hsl(0.1058, 0.1686, 0.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Pos::try_new(0, 0).unwrap())
        .insert(Start);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(35.0, 35.0)),
                color: Color::hsl(1., 1., 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Pos::try_new(9, 9).unwrap())
        .insert(End);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(400.0, 400.0)),
            color: Color::hsl(0., 0., 0.),
            ..Default::default()
        },
        transform: Transform::from_xyz(-20., -20., 1.),
        ..Default::default()
    });
}

fn grid_to_transform(mut query: Query<(&Pos, &mut Transform)>) {
    query.for_each_mut(|(pos, mut transform): (&Pos, Mut<Transform>)| {
        transform.translation.x = ((pos.x as i32 * 40) - 200) as f32;
        transform.translation.y = ((pos.y as i32 * 40) - 200) as f32;
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
                let x = (cursor_pos.x as i32 - 180) / 40;
                let y = (cursor_pos.y as i32 - 85) / 40;

                if let Some(pos) = Pos::try_new(x as i32, y as i32) {
                    my_events.send(ToggleBlockEvent { pos });
                }
            }
        }
    }
}

fn toggle_block(
    mut my_events: EventReader<ToggleBlockEvent>,
    blocks: Query<(Entity, &Pos), With<Block>>,
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
                            custom_size: Some(Vec2::new(35.0, 35.0)),
                            color: Color::hsl(0.1058, 0.1686, 1.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(event.pos)
                    .insert(Block);
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
    start: Query<&Pos, With<Start>>,
    end: Query<&Pos, With<End>>,
    blocks: Query<&Pos, With<Block>>,
    paths: Query<Entity, With<Path>>,
    mut commands: Commands,
) {
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
                        color: Color::hsl(0.1058, 0.1686, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(pos)
                .insert(Path);
        }
    }
}
