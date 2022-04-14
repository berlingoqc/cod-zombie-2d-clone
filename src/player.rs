use bevy::prelude::*;

#[derive(Default)]
pub struct Player {
    pub entity: Option<Entity>,
    pub x: i32,
    pub y: i32,
}
