use bevy::prelude::*;


#[derive(Default, Component)]
pub struct Player {}



#[derive(Default, Component)]
pub struct Projectile {}

#[derive(Component)]
pub struct Velocity {
    pub v: Vec2
}


