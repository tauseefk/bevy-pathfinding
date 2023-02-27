pub use crate::prelude::*;

#[derive(Default, Clone, PartialEq, Debug)]
pub enum PlayerAnimationVariant {
    #[default]
    Idle,
    Walking,
}

impl AnimationLoop for PlayerAnimationVariant {
    fn page(&self) -> (usize, usize) {
        match self {
            PlayerAnimationVariant::Idle => (0, 6),
            PlayerAnimationVariant::Walking => (24, 6),
        }
    }
}

#[derive(Default, Clone, Component, AnimationTransitionMacro)]
pub struct PlayerAnimationState {
    #[variant]
    pub variant: PlayerAnimationVariant,
    pub idx: usize,
}
