use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    collider::{MovementCollider, ProjectileCollider},
    game::{Zombie, ZombieGame},
    map::{MapElementPosition, WindowPanel, Window, Size},
    weapons::{weapons::{Projectile, Weapon, WeaponState, WeaponBundle, AmmunitionState, WeaponCurrentAction}, loader::WeaponAssetState}, health::Health
};


const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Default, Component)]
pub struct Player {}

#[derive(Default, Component)]
pub struct PlayerCurrentInteraction {
    pub interaction: bool,
    pub interaction_cooldown_at: f32,
    pub entity: u32,
    pub interaction_type: PlayerInteractionType,
}



#[derive(Default, Clone, Copy)]
pub enum PlayerInteractionType {
    #[default]
    None = 0,

    RepairWindow,
}

#[derive(Default, Component)]
pub struct PlayerInteraction {
    pub interaction_type: PlayerInteractionType
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    #[bundle] 
    pub sprite: SpriteBundle,
    pub interaction: PlayerCurrentInteraction,
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
            },
            interaction: PlayerCurrentInteraction {
                interaction: false,
                interaction_cooldown_at: 0.,
                entity: 0,
                interaction_type: PlayerInteractionType::None
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


    if let Some(alternate_weapon) = &zombie_game.configuration.starting_alternate_weapon {
        let weapon = weapons.weapons.iter().find(|w| w.name.eq(alternate_weapon.as_str())).unwrap().clone();
        let weapon = commands.spawn()
            .insert_bundle(WeaponBundle::new(weapon, false)).id();
        commands.entity(player).add_child(weapon);
    }
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
                    commands.entity(hit_entity).despawn();
                }
                commands.entity(projectile_entity).despawn();
                break 'outer;
            }
        }
    }
}

pub fn system_interaction_player(
    mut commands: Commands,
    mut query_player: Query<(&Transform, &mut PlayerCurrentInteraction), With<Player>>,
    interaction_query: Query<
        (Entity, &Transform, &MapElementPosition, &PlayerInteraction),
        (
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,

    keyboard_input: Res<Input<KeyCode>>,

    query_window: Query<(&Window, &Children)>,
    mut query_panel: Query<(&mut WindowPanel, &Size, &mut Health, &mut Sprite)>
) {

    for (player_transform, mut interaction) in query_player.iter_mut() {
        for (entity, transform, info, player_interaction) in interaction_query.iter() {
            let collision = collide(player_transform.translation, Vec2::new(25., 25.), info.position.extend(10.), info.size * 2.);
            if collision.is_some() {
                // notify use that key perform action
                interaction.interaction = true;
                interaction.entity = entity.id();
                interaction.interaction_type = player_interaction.interaction_type;
            } else {
                if entity.id() == interaction.entity {
                    interaction.interaction = false;
                }
            }
        } 


        if interaction.interaction && keyboard_input.pressed(KeyCode::F) {
            match interaction.interaction_type {
                PlayerInteractionType::RepairWindow => {
                    let (_, children) = query_window.get(Entity::from_raw(interaction.entity)).unwrap();
                    for &child_entity in children.iter() {
                        let (_,size, mut health, mut sprite) = query_panel.get_mut(child_entity).unwrap();
                        if health.current_health <= 0. {
                            sprite.custom_size = Some(size.0);
                            health.current_health = 1.;
                            break;
                        }
                    }
                },
                _ => {}
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
    collider_query: Query<
        (Entity, &Transform, &MapElementPosition),
        (
            With<MovementCollider>,
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,
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

