use bevy::{prelude::*, sprite::collide_aabb::collide};
use serde::Deserialize;

use super::debug::DebugComponent;

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MovementCollider {
    pub size: Vec2,
    pub allowed_entity_type: Vec<String>,
}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct ProjectileCollider {}

#[derive(Component)]
pub struct CollisionEvent {}

pub fn is_colliding<T: Component, R: Component>(
    destination: Vec3,
    size: Vec2,
    character_type: &str,
    collider_query: &Query<(Entity, &Transform, &MovementCollider), (Without<T>, Without<R>)>,
) -> bool {
    for (_, transform, collider) in collider_query.iter() {
        if collider
            .allowed_entity_type
            .iter()
            .any(|x| x == character_type)
        {
            continue;
        }
        let collision = collide(destination, size, transform.translation, collider.size);
        if collision.is_some() {
            return true;
        }
    }
    return false;
}

fn spawn_debug_collider_sprite(commands: &mut Commands, position: Vec3, collider_size: Vec2) {
    commands.spawn().insert_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgba(0.10, 0.30, 0.50, 0.3),
            custom_size: Some(collider_size),
            ..Sprite::default()
        },
        transform: Transform {
            translation: position + Vec3::new(0., 0., 10.),
            ..Transform::default()
        },
        ..SpriteBundle::default()
    }).insert(DebugComponent{});
}

pub fn init_collider_debug(
    mut commands: Commands,
    q_new_collider: Query<(&MovementCollider, &Transform)>,
) {
    for (collider, transform) in q_new_collider.iter() {
        println!("MOvement collider {:?} {:?}", transform.translation, collider.size);
        spawn_debug_collider_sprite(&mut commands, transform.translation, collider.size);
    }
}

// debug must be only on debug collider mode and must remove the sprite on leave
pub fn system_collider_debug(
    mut commands: Commands,
    q_new_collider: Query<(&MovementCollider, &Transform), Added<MovementCollider>>,
) {
    for (collider, transform) in q_new_collider.iter() {
        spawn_debug_collider_sprite(&mut commands, transform.translation, collider.size);
    }
}
