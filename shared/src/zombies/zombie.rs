use bevy::prelude::*;

use crate::{
    map::{MapElementPosition, WindowPanel, Window},
    collider::{MovementCollider, ProjectileCollider},
    weapons::weapons::{WeaponState, WeaponCurrentAction},
    player::Player,
    health::Health, animation::AnimationTimer, character::{LookingAt, CharacterMovementState}
};

use rand::seq::SliceRandom;
use pathfinding::prelude::astar;


use super::spawner::ZombieSpawnerConfig;


// bot destination is a component
// to register and apply the target of a bot
#[derive(Component, Default)]
pub struct BotDestination {
    // Destination element
    pub destination: MapElementPosition,
    // Precalculate path to reach the target
    pub path: Vec<(i32, i32)>,
    // entity trying to reach
    pub entity: u32
}


// the different state of a zombie
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ZombieState {
    // When the zombie is spawning
    AwakingFromTheDead,
    // When the zombie is outside the player area
    // and trying to get inside
    FindingEnterace,
    // When the zombie is inside the player area
    // and trying to reach a player
    FollowingPlayer,
}


#[derive(Component)]
pub struct Zombie {
    pub state: ZombieState,
}


#[derive(Bundle)]
pub struct ZombieBundle {
    #[bundle]
    sprite_bundle: SpriteSheetBundle,
    collider: MovementCollider,
    destination: BotDestination,
    projectile_collider: ProjectileCollider,
    info: MapElementPosition,
    zombie: Zombie,
    weapon_state: WeaponState,
    animation_timer: AnimationTimer,
    looking_at: LookingAt,
    chracter_movement_state: CharacterMovementState,
}

impl ZombieBundle {
    pub fn new(info: MapElementPosition, dest: BotDestination) -> ZombieBundle {
        ZombieBundle {
            sprite_bundle: SpriteSheetBundle {
               transform: Transform {
                    translation: info.position.extend(10.0),
                    scale: Vec3::new(0., 0., 0.),
                    ..Transform::default()
                },
                ..default()
            },
            collider: MovementCollider {},
            projectile_collider: ProjectileCollider {},
            zombie: Zombie {
                state: ZombieState::AwakingFromTheDead,
            },
            chracter_movement_state: CharacterMovementState { state: "rising".to_string(), sub_state: "".to_string() },
            animation_timer: AnimationTimer {
                timer: Timer::from_seconds(0.1, true),
                index: 0,
                offset: 0,
                current_state: "".to_string(),
                asset_type: "zombie".to_string(),
            },
            looking_at: LookingAt(dest.destination.position),
            info,
            destination: dest,
            weapon_state: WeaponState { fired_at: 0., state: WeaponCurrentAction::Firing }
        }
    }
}

pub fn system_zombie_handle(
    // mut commands: Commands,
    query_player: Query<&Transform, (With<Player>, Without<Zombie>)>,
    mut config: ResMut<ZombieSpawnerConfig>,
    mut query_zombies: Query<(&mut Transform, &mut BotDestination, &mut Zombie, &mut WeaponState, &mut LookingAt, &mut CharacterMovementState), With<Zombie>>,
    mut query_windows: Query<(&Window, Entity, &Children)>,
    mut query_panel: Query<(&WindowPanel, &mut Sprite, &mut Health)>,

    time: Res<Time>
) {
    let player = query_player.get_single();
    if player.is_err() {
        return;
    }
    let player = player.unwrap();
    for (mut pos, mut dest, mut zombie, mut weapon_state, mut looking_at, mut movement_state) in query_zombies.iter_mut() {
        match zombie.state {
            ZombieState::AwakingFromTheDead => {
                if pos.scale.x < 1.0 {
                    let mut rng = rand::thread_rng();
                    config.nums_ndg.shuffle(&mut rng);
                    pos.scale += Vec3::new(0.01, 0.01, 0.01);
                    //pos.rotation = Quat::from_rotation_z(config.nums_ndg[75] / 10.);
                } else {
                    pos.rotation = Quat::from_rotation_z(0.);
                    zombie.state = ZombieState::FindingEnterace;
                    movement_state.state = "walking".to_string();
                }
            }
            ZombieState::FindingEnterace => {
                if let Some(el) = dest.path.pop() {
                    pos.translation.x = el.0 as f32;
                    pos.translation.y = el.1 as f32;
                } else {

                    let d = query_windows.get_mut(Entity::from_raw(dest.entity));

                    if d.is_err() {
                        // windows no longer exists , find again
                        continue;
                    }

                    let (window, entity, children) = d.unwrap();

                    let mut attack = false;
                    let mut remaining = 0;
                    for &panel_entity in children.iter() {
                        let (pannel, mut sprite, mut health) = query_panel.get_mut(panel_entity).unwrap();
                        if health.current_health > 0. {
                            if !attack {
                                let current_time = time.time_since_startup().as_secs_f32();
                                // TODO : NOT HARDCODE
                                if !(current_time < weapon_state.fired_at + 1.) {

                                    health.current_health = 0.;
                                    attack = true;

                                    sprite.custom_size = Some(Vec2::new(0., 0.));

                                    weapon_state.fired_at = current_time;

                                } else {
                                    remaining += 1;
                                }
                            } else {
                                remaining += 1;
                            }
                        }
                    }

                    if remaining == 0 {
                        zombie.state = ZombieState::FollowingPlayer;
                    }
                }
            }
            ZombieState::FollowingPlayer | ZombieState::FindingEnterace => {
                if let Some(el) = dest.path.pop() {
                    pos.translation.x = el.0 as f32;
                    pos.translation.y = el.1 as f32;
                }

                if dest.path.len() == 0 {
                    // change target for the user
                    let goal = (player.translation.x as i32, player.translation.y as i32);
                    looking_at.0 = player.translation.truncate();
                    let mut result = astar(
                        &(pos.translation.x as i32, pos.translation.y as i32),
                        |&(x, y)| {
                            vec![
                                (x + 1, y + 2),
                                (x + 1, y - 2),
                                (x - 1, y + 2),
                                (x - 1, y - 2),
                                (x + 2, y + 1),
                                (x + 2, y - 1),
                                (x - 2, y + 1),
                                (x - 2, y - 1),
                            ]
                            .into_iter()
                            .map(|p| (p, 1))
                        },
                        |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
                        |&p| p == goal,
                    )
                    .unwrap()
                    .0;

                    result.reverse();
                    let len = result.len() as f32;
                    dest.path = if len >= 10. {
                        let len_taken = (len * 0.25).ceil() as usize;
                        let start = result.len() - 1 - len_taken;
                        let end = result.len() - 1;
                        result[start..end].to_vec()
                    } else {
                        result
                    };
                }
            }
        }
    }
}


