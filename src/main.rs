mod components;
mod events;
mod systems;

mod prelude {
    pub use std::ops::Not;

    pub use bevy::prelude::*;
    pub use bevy_ecs_ldtk::prelude::*;
    pub use pathfinding::prelude::*;

    pub use crate::components::*;
    pub use crate::events::*;
    pub use crate::systems::*;

    pub const GRID_SIZE: i32 = 8;
    pub const GRID_BLOCK_SIZE: i32 = 32;
    pub const WINDOW_HEIGHT: i32 = 256;
    pub const WINDOW_WIDTH: i32 = 256;

    pub const YELLOW: Color = Color::hsl(53.0, 0.99, 0.50);
    pub const PALE: Color = Color::hsl(237.0, 0.45, 0.9);
    pub const BLUE: Color = Color::hsl(232.0, 0.62, 0.57);
    pub const WHITE: Color = Color::hsl(0., 0., 1.);
    pub const BLACK: Color = Color::hsl(0., 0., 0.);
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
