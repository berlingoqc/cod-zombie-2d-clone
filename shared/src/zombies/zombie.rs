use bevy::{prelude::*, math::const_vec2, ecs::query, sprite::collide_aabb::collide};

use crate::{
    map::{MapElementPosition, WindowPanel, Window},
    collider::{MovementCollider, ProjectileCollider, is_colliding},
    weapons::weapons::{WeaponState, WeaponCurrentAction},
    player::{Player, MainCamera, PLAYER_SIZE},
    health::Health, animation::AnimationTimer, character::{LookingAt, CharacterMovementState}, utils::vec2_perpendicular_counter_clockwise
};

use rand::seq::SliceRandom;
use pathfinding::prelude::astar;

use super::spawner::ZombieSpawnerConfig;

pub const ZOMBIE_SIZE: Vec2 = const_vec2!([25. , 25. ]);

// bot destination is a component
// to register and apply the target of a bot
#[derive(Component)]
pub struct BotDestination {
    // Destination element
    pub destination: Vec2,
    // Precalculate path to reach the target
    pub path: Vec<(i32, i32)>,
    // entity trying to reach
    pub entity: Entity
}

impl BotDestination {

    pub fn move_bot<T: Component>(&mut self, pos: &mut Transform, collider_query: &Query<
        (Entity, &Transform, &MovementCollider),
        Without<T>
    >,) -> bool {
        if let Some(el) = self.path.pop() {
            if !is_colliding(Vec3::new(el.0 as f32, el.1 as f32, 10.), ZOMBIE_SIZE, "zombie", &collider_query) {
                pos.translation.x = el.0 as f32;
                pos.translation.y = el.1 as f32;
            }
            return true;
        } else {
            return false;
        }
    }

    // set_destination get a element position and entity id , keep it and recalculate the path required to get there
    pub fn set_destination(&mut self, position_dest: Vec2, bot_position: Vec2, entity: Entity, path_max_length: f32) -> () {
        let goal = (position_dest.x as i32, position_dest.y as i32);
        let mut result = astar(
            &(bot_position.x as i32, bot_position.y as i32),
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
        
        if path_max_length > 0. {
            let len = result.len() as f32;
            result = if len >= path_max_length  {
                let len_taken = (len * 0.25).ceil() as usize;
                let start = result.len() - 1 - len_taken;
                let end = result.len() - 1;
                result[start..end].to_vec()
            } else {
                let end = (len as usize) - 1;
                result[0..end].to_vec()
            };
        }

        self.destination = position_dest;
        self.entity = entity;
        self.path = result;
    }
}


// the different state of a zombie
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ZombieState {
    // When the zombie is spawning
    AwakingFromTheDead,
    // When the zombie is outside the player area
    // and trying to get inside
    FindingEnterace,

    CrossingEntrance,
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
            collider: MovementCollider {
                size: ZOMBIE_SIZE,
                ..default()
            },
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
            looking_at: LookingAt(dest.destination, false),
            info,
            destination: dest,
            weapon_state: WeaponState { fired_at: 0., state: WeaponCurrentAction::Firing }
        }
    }
}


pub fn system_zombie_handle(
    // mut commands: Commands,
    query_player: Query<(Entity, &Transform), (With<Player>, Without<Zombie>)>,
    mut config: ResMut<ZombieSpawnerConfig>,
    mut query_zombies: Query<(&mut Transform, &mut BotDestination, &mut Zombie, &mut WeaponState, &mut LookingAt, &mut CharacterMovementState), With<Zombie>>,
    //mut query_windows: Query<(&mut Window, Entity, &Children)>,
    //mut query_panel: Query<(&WindowPanel, &mut Sprite, &mut Health)>,
    mut query_ennemy: Query<(Entity, &mut Health), Without<Zombie>>,

    collider_query: Query<
        (Entity, &Transform, &MovementCollider),
        Without<Zombie>
    >,

    time: Res<Time>
) {
    let player = query_player.get_single();
    if player.is_err() {
        return;
    }
    let (entity, player) = player.unwrap();
    for (mut pos, mut dest, mut zombie, mut weapon_state, mut looking_at, mut movement_state) in query_zombies.iter_mut() {
        match zombie.state {
            ZombieState::AwakingFromTheDead => {
                if pos.scale.x < 1.0 {
                    let mut rng = rand::thread_rng();
                    config.nums_ndg.shuffle(&mut rng);
                    pos.scale += Vec3::new(0.01, 0.01, 0.01);
                } else {
                    pos.rotation = Quat::from_rotation_z(0.);
                    zombie.state = ZombieState::FindingEnterace;
                    movement_state.state = "walking".to_string();
                }
            }
            ZombieState::FindingEnterace => {
                if !dest.move_bot(&mut pos, &collider_query) {
                    if let Ok((entity, mut health)) = query_ennemy.get_mut(dest.entity) {
                        if health.current_health > 0. {
                            let current_time = time.time_since_startup().as_secs_f32();
                            if !(current_time < weapon_state.fired_at + 1.) {
                                health.tmp_health -= 1.;
                                weapon_state.fired_at = current_time;
                            }
                        } else {

                            zombie.state = ZombieState::CrossingEntrance;


                            let p2 = looking_at.0;
                            let p1 = pos.translation.truncate();
                            let direction_cross = vec2_perpendicular_counter_clockwise(Vec2::new(p2.x - p1.x, p2.y - (p1.y * -1.)).normalize_or_zero());

                            let destination = p1 + (direction_cross * 50.);
                            looking_at.0 = destination;
                            dest.set_destination(destination, p1, entity.clone(), 0.);
                        }
                    } else {
                        println!("ERROR FINDING THE SHIT");
                    }
                }
            }
            ZombieState::CrossingEntrance => {
                if !dest.move_bot(&mut pos, &collider_query) {
                    zombie.state = ZombieState::FollowingPlayer;
                }
            },
            ZombieState::FollowingPlayer => {
                if !dest.move_bot(&mut pos, &collider_query) {

                    // Valid if i'm colliding with him.
                    if let Some(collision) = collide(pos.translation, ZOMBIE_SIZE * 2., player.translation, PLAYER_SIZE) {
                        if let Ok((_, mut health)) = query_ennemy.get_mut(dest.entity) {
                            let current_time = time.time_since_startup().as_secs_f32();
                            if !(current_time < weapon_state.fired_at + 1.) {
                                health.tmp_health -= 1.;
                                weapon_state.fired_at = current_time;
                            }
                        }
                    } else {
                        dest.set_destination(
                            player.translation.truncate(), 
                            pos.translation.truncate(), 
                            entity.clone(), 10.
                        );
                        looking_at.0 = player.translation.truncate();
                    }
                }
            }
        }
    }
}


