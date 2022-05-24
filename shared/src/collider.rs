use bevy::prelude::*;
use serde::Deserialize;


#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MovementCollider {}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct ProjectileCollider {}

#[derive(Component)]
pub struct CollisionEvent {}
