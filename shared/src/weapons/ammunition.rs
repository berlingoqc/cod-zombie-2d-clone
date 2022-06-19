use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{character::Velocity, game::GameSpeed, map::MapElementPosition, zombies::zombie::Zombie, collider::ProjectileCollider, player::Player};

use super::weapons::{ExpiringComponent, Projectile};

pub fn apply_velocity(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Velocity, Entity), Without<Player>>,

    game_speed: Res<GameSpeed>
) {
    for (mut transform, velocity, entity) in query.iter_mut() {
        let x_vel = velocity.v.x * game_speed.0;
        let y_vel = velocity.v.y * game_speed.0;
        if x_vel == 0. && y_vel == 0. {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += x_vel;
        transform.translation.y += y_vel;
    }
}

pub fn movement_projectile(
    mut commands: Commands,
    time: Res<Time>,
    projectile_query: Query<(Entity, &Transform, &ExpiringComponent), With<Projectile>>,
    collider_query: Query<
        (Entity, &Transform, &MapElementPosition, Option<&Zombie>),
        (
            With<ProjectileCollider>,
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,
) {
    let mut i = 0;
    'outer: for (projectile_entity, transform, expiring) in projectile_query.iter() {
        i += 1;
        if expiring.created_at + expiring.duration <= time.time_since_startup().as_secs_f32() {
            commands.entity(projectile_entity).despawn();
            break;
        }
        for (hit_entity, transform_collider, info, zombie) in collider_query.iter() {
            let collision = collide(
               transform_collider.translation,
                info.size,
                transform.translation,
                Vec2::new(10., 10.),
            );
            if collision.is_some() {
                if let Some(_zombie) = zombie {
                    println!("HITTINH ZOMBIEEE");
                    commands.entity(hit_entity).despawn();
                }
                commands.entity(projectile_entity).despawn();
                break 'outer;
            }
        }
    }
}
