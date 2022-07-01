use bevy::{prelude::*, math::const_vec2, ecs::query, sprite::collide_aabb::collide};

use crate::shared::{
    map::{MapElementPosition, WindowPanel, Window},
    collider::{MovementCollider, ProjectileCollider, is_colliding},
    weapons::weapons::{WeaponState, WeaponCurrentAction},
    player::{Player, MainCamera, PLAYER_SIZE},
    health::Health, animation::AnimationTimer, character::{LookingAt, CharacterMovementState, Death}, utils::{vec2_perpendicular_counter_clockwise, Checksum}
};

use rand::seq::SliceRandom;
use pathfinding::prelude::astar;

use super::spawner::ZombieSpawnerConfig;

pub const ZOMBIE_SIZE: Vec2 = const_vec2!([9. , 9. ]);

// bot destination is a component
// to register and apply the target of a bot
#[derive(Component, Reflect)]
pub struct BotDestination {
    // Destination element
    pub destination: Vec2,
    // Requested_movement
    pub requested_movement: Option<Vec2>,
    // Precalculate path to reach the target
    pub path: Vec<(i32, i32)>,
    // entity trying to reach
    pub entity: Entity
}

impl Default for BotDestination {
   fn default() -> Self {
       BotDestination { destination: Vec2::default(), path: vec![], entity: Entity::from_raw(0), requested_movement: None, }
   }
}

impl BotDestination {

    pub fn move_bot<T: Component, R: Component>(&mut self, pos: &mut Transform, collider_query: &Query<
        (Entity, &Transform, &MovementCollider),
        (Without<T>, Without<R>)
    >,) -> bool {
        if let Some(el) = self.path.pop() {
            if !is_colliding(Vec3::new(el.0 as f32, el.1 as f32, 10.), ZOMBIE_SIZE, "zombie", &collider_query) {
                self.requested_movement = Some(Vec2::new(el.0 as f32, el.1 as f32));
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
#[derive(Clone, Eq, PartialEq, Debug, Hash, Reflect, Default)]
pub enum ZombieState {
    // When the zombie is spawning
    #[default]
    AwakingFromTheDead = 0,
    // When the zombie is outside the player area
    // and trying to get inside
    FindingEnterace = 1,

    CrossingEntrance = 2,
    // When the zombie is inside the player area
    // and trying to reach a player
    FollowingPlayer = 3,
}


#[derive(Component, Reflect, Default)]
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
    checksum: Checksum,
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
            checksum: Checksum::default(),
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


pub fn system_move_zombie(
    mut query_zombies: Query<(&mut Transform, &mut BotDestination), With<Zombie>>,
) {
    for (mut transform, mut bot_destination) in query_zombies.iter_mut() {
        if let Some(requested_movement) = bot_destination.requested_movement {
            transform.translation.x = requested_movement.x;
            transform.translation.y = requested_movement.y;
            bot_destination.requested_movement = None
        }
    }
}


pub fn system_zombie_handle(
    // mut commands: Commands,
    query_player: Query<(Entity, &Transform), (With<Player>, Without<Zombie>, Without<Death>)>,
    mut config: ResMut<ZombieSpawnerConfig>,
    mut query_zombies: Query<(&mut Transform, &mut BotDestination, &mut Zombie, &mut WeaponState, &mut LookingAt, &mut CharacterMovementState), With<Zombie>>,
    //mut query_windows: Query<(&mut Window, Entity, &Children)>,
    //mut query_panel: Query<(&WindowPanel, &mut Sprite, &mut Health)>,
    mut query_ennemy: Query<(Entity, &mut Health), Without<Zombie>>,

    collider_query: Query<
        (Entity, &Transform, &MovementCollider),
        (Without<Zombie>, Without<Death>)
    >,

    time: Res<Time>
) {
    for (mut pos, mut dest, mut zombie, mut weapon_state, mut looking_at, mut movement_state) in query_zombies.iter_mut() {
        match zombie.state {
            ZombieState::AwakingFromTheDead => {
                if pos.scale.x < 1.0 {
                    //let mut rng = rand::thread_rng();
                    //config.nums_ndg.shuffle(&mut rng);
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

                            println!("Crossing entrace");


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
                    
                    // Query the players to find the closest
                    let mut distance = 50000.;
                    let mut player_entity: Entity = Entity::from_raw(0);
                    let mut player_translation: Vec3 = Vec3::default();

                    for (entity, transform) in query_player.iter() {
                        let dst = pos.translation.truncate().distance(transform.translation.truncate());
                        if dst < distance {
                            player_entity = entity.clone();
                            player_translation = transform.translation;
                            distance = dst;
                        }
                    }

                    // Valid if i'm colliding with him.
                    if let Some(collision) = collide(pos.translation, ZOMBIE_SIZE * 2., player_translation, PLAYER_SIZE) {
                        if let Ok((_, mut health)) = query_ennemy.get_mut(dest.entity) {
                            let current_time = time.time_since_startup().as_secs_f32();
                            if !(current_time < weapon_state.fired_at + 1.) {
                                health.tmp_health -= 1.;
                                weapon_state.fired_at = current_time;
                            }
                        }
                    } else {
                        let player_translation = player_translation.truncate();
                        dest.set_destination(
                            player_translation, 
                            pos.translation.truncate(), 
                            player_entity.clone(), 10.
                        );
                        looking_at.0 = player_translation;
                    }
                }
            }
        }
    }
}


