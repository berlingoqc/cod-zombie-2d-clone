use bevy::{prelude::*, sprite::collide_aabb::collide};
use serde::Deserialize;

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MovementCollider {
    pub size: Vec2,
}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct ProjectileCollider {}

#[derive(Component)]
pub struct CollisionEvent {}


pub fn is_colliding<T : Component>(
    destination: Vec3,
    size: Vec2,
    collider_query: &Query<
        (Entity, &Transform, &MovementCollider),
        Without<T>
    >,
) -> bool {
    for (_, transform, collider) in collider_query.iter() {
        let collision = collide(destination, size, transform.translation, collider.size);
        if collision.is_some() {
            println!("COLLIDING WITH {:?} {:?} | {:?} {:?}", destination, size, transform.translation, collider.size);
            return true;
        }
    }
    return false;
}