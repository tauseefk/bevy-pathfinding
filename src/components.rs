use crate::prelude::*;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub player: Player,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    pub wall: Wall,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct ChestBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub chest: Chest,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Chest;

#[derive(Component, Eq, PartialEq, Copy, Clone, Hash, Debug, Default)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub const fn try_new(x: i32, y: i32) -> Option<Self> {
        if x <= 0 || y <= 0 || x > GRID_SIZE as i32 || y > GRID_SIZE as i32 {
            None
        } else {
            Some(Self {
                x: x as i32,
                y: y as i32,
            })
        }
    }

    pub const fn min(self) -> bool {
        self.x == 1 && self.y == 1
    }

    pub const fn max(self) -> bool {
        self.x == GRID_SIZE && self.y == GRID_SIZE
    }
}

#[derive(Component)]
pub struct Path;
