use bevy::prelude::*;

#[derive(Default, Component)]
pub struct CharacterMovementState {
    pub state: String,
    pub sub_state: String,
}

#[derive(Default, Component)]
pub struct LookingAt(pub Vec2);


#[derive(Component)]
pub struct Velocity {
    pub v: Vec2,
}