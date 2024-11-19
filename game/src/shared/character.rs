use bevy::prelude::*;

#[derive(Default, Component, Clone,  Reflect)]
pub struct CharacterMovementState {
    pub state: String,
    pub sub_state: String,
}

#[derive(Default, Component, Copy, Clone, Reflect)]
pub struct LookingAt(pub Vec2, pub bool);


#[derive(Default, Component,Clone, Copy, Reflect)]
pub struct Death {}


#[derive(Component, Reflect, Clone, Copy, Default)]
pub struct Velocity {
    pub v: Vec2,
}
