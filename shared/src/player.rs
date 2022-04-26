use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    collider::{MovementCollider, ProjectileCollider},
    game::{Zombie, ZombieGame},
    map::MapElementPosition,
    weapons::{weapons::{Projectile, Weapon, WeaponState, WeaponBundle, ActivePlayerWeapon, AmmunitionState, WeaponCurrentAction}, loader::WeaponAssetState}
};


const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Default, Component)]
pub struct Player {}


#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    #[bundle] 
    pub sprite: SpriteBundle,
}

impl PlayerBundle {
    fn new() -> PlayerBundle {
        PlayerBundle { 
            player: Player{},
            sprite : SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.75),
                    custom_size: Some(Vec2::new(25.0, 25.0)),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: Vec3::new(0., 0., 10.),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            }
        }
    }
}

pub fn setup_players(
    mut commands: Commands,

    zombie_game: &ResMut<ZombieGame>,
    weapons: &Res<WeaponAssetState>,
) {
    // TODO: for multiplayer
    // Fetch the location of the player spawner in the map
    // Use your player index in the player array of the game
    // to select your color and where your spawn

    // get the default weapon for the map
    let default_weapon_name = zombie_game.configuration.starting_weapon.as_str();

    let weapon = weapons.weapons.iter().find(|w| w.name.eq(default_weapon_name)).unwrap().clone();

    let player = commands.spawn_bundle(PlayerBundle::new()).id();
        
    let weapon = commands.spawn()
        .insert_bundle(WeaponBundle::new(weapon, true)).id();

    commands.entity(player).add_child(weapon);

}



#[derive(Default, Component)]
pub struct ExpiringComponent {
    pub created_at: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub v: Vec2,
}

#[derive(Component)]
pub struct MainCamera;

pub fn apply_velocity(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Velocity, Entity)>,
) {
    for (mut transform, velocity, entity) in query.iter_mut() {
        let x_vel = velocity.v.x * TIME_STEP;
        let y_vel = velocity.v.y * TIME_STEP;
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
    'outer: for (projectile_entity, transform, expiring) in projectile_query.iter() {
        if expiring.created_at + expiring.duration <= time.time_since_startup().as_secs_f32() {
            commands.entity(projectile_entity).despawn();
            break;
        }
        for (hit_entity, transform_collider, info, zombie) in collider_query.iter() {
            let collision = collide(
                transform.translation,
                Vec2::new(5., 5.),
                transform_collider.translation,
                info.size,
            );
            if collision.is_some() {
                if let Some(_zombie) = zombie {
                    commands.entity(hit_entity).despawn();
                }
                commands.entity(projectile_entity).despawn();
                break 'outer;
            }
        }
    }
}

pub fn input_player(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<&mut Transform, With<Player>>,
	mut query_player_weapon: Query<(&mut AmmunitionState, &mut WeaponState, &Weapon), With<ActivePlayerWeapon>>,
    collider_query: Query<
        (Entity, &Transform, &MapElementPosition),
        (
            With<MovementCollider>,
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    for mut player_transform in query.iter_mut() {

        let mut movement = Vec3::default();
        let mut moved = false;

        if keyboard_input.pressed(KeyCode::W) {
            movement += Vec3::new(0., 1., 0.);
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::S) {
            movement += Vec3::new(0., -1., 0.);
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::A) {
            movement += Vec3::new(-1., 0., 0.);
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::D) {
            movement += Vec3::new(1., 0., 0.);
            moved = true;
        }
        
        if !moved {
            return;
        }

        let dest = player_transform.translation + (movement * 3.);

        let mut save_move = true;
        for (_, transform, info) in collider_query.iter() {
            let collision = collide(dest, Vec2::new(25., 25.), transform.translation, info.size);
            if collision.is_some() {
                save_move = false;
            }
        }

        if save_move {
            player_transform.translation = dest;
        }
    }
}

