use bevy::{prelude::*, sprite::collide_aabb::collide};
use serde::Deserialize;


#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MovementCollider {
    pub size: Vec2,
    pub allowed_entity_type: Vec<String>
}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct ProjectileCollider {}

#[derive(Component)]
pub struct CollisionEvent {}


pub fn is_colliding<T : Component, R : Component>(
    destination: Vec3,
    size: Vec2,
    character_type: &str,
    collider_query: &Query<
        (Entity, &Transform, &MovementCollider),
        (Without<T>, Without<R>)
    >,
) -> bool {
    for (_, transform, collider) in collider_query.iter() {
        if collider.allowed_entity_type.iter().any(|x| x == character_type) { continue }
        let collision = collide(destination, size, transform.translation, collider.size);
        if collision.is_some() {
            return true;
        }
    }
    return false;
}