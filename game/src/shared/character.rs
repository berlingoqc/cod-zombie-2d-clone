use bevy::prelude::*;

#[derive(Default, Component, Reflect)]
pub struct CharacterMovementState {
    pub state: String,
    pub sub_state: String,
}

#[derive(Default, Component, Reflect)]
pub struct LookingAt(pub Vec2, pub bool);


#[derive(Default, Component, Reflect)]
pub struct Death {}


#[derive(Component, Reflect, Default)]
pub struct Velocity {
    pub v: Vec2,
}
