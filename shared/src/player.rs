use bevy::{prelude::*, sprite::collide_aabb::collide, reflect::TypeUuid, asset::{LoadContext, AssetLoader, BoxedFuture, LoadedAsset}};
use serde::Deserialize;

use crate::{
    collider::{MovementCollider, ProjectileCollider},
    game::{Zombie, ZombieGame, GameState},
    map::{MapElementPosition, WindowPanel, Window, Size},
    weapons::{weapons::{Projectile, Weapon, WeaponState, WeaponBundle, AmmunitionState, WeaponCurrentAction}, loader::WeaponAssetState}, health::Health, animation::{SpriteSheetAnimationsConfiguration, SpriteSheetConfiguration}, utils::get_cursor_location
};


const TIME_STEP: f32 = 1.0 / 60.0;




#[derive(Default, Component)]
pub struct CharacterMovementState {
    pub state: String,
    pub sub_state: String,
}

#[derive(Default, Component)]
pub struct Player {
    pub active_weapon_name: String,
}

#[derive(Component)]
pub struct PlayerCurrentInteraction {
    // tell if or not there is an interaction available for the user
    pub interaction: bool,
    // cooldown between each interaction
    pub interaction_cooldown: f32,
    // entity that has the interaction component
    pub entity: Entity,
    pub child_entity: Entity,
    // type of interaction
    pub interaction_type: PlayerInteractionType,

    // tell if the player is doing the interaction
    pub interacting: bool,

    // when the user last trigger the interaction
    pub interaction_trigger_at: f32,
}



#[derive(Default, Clone, Copy)]
pub enum PlayerInteractionType {
    #[default]
    None = 0,

    RepairWindow,
}

#[derive(Default, Component)]
pub struct PlayerInteraction {
    pub interaction_type: PlayerInteractionType,
    pub interaction_timeout: f32
}


#[derive(Default, Component)]
pub struct LookingAt(pub Vec2);

#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    pub index: usize,
    pub offset: usize,
    pub asset_type: String,
    pub current_state: String,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    #[bundle] 
    pub sprite: SpriteSheetBundle,
    pub interaction: PlayerCurrentInteraction,
    pub looking_direction: LookingAt,
    pub animation_timer: AnimationTimer,
    pub character_movement_state: CharacterMovementState,
}

impl PlayerBundle {
    fn new(starting_weapon_name: &str) -> PlayerBundle {
        PlayerBundle { 
            player: Player{
                active_weapon_name: starting_weapon_name.to_string(),
                ..default()
            },
            sprite : SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 10.),
                    ..Transform::default()
                },
                ..default()
            },
            character_movement_state: CharacterMovementState { state: String::from("walking"), sub_state: "".to_string() },
            looking_direction: LookingAt(Vec2::new(0., 0.)),
            animation_timer: AnimationTimer{ 
                timer: Timer::from_seconds(0.1, true),
                index: 0,
                offset: 0,
                asset_type: "player".to_string(),
                current_state: "".to_string(),
            },
            interaction: PlayerCurrentInteraction {
                interaction: false,
                interacting: false,
                interaction_cooldown: 0.,
                entity: Entity::from_raw(0),
                child_entity: Entity::from_raw(0),
                interaction_type: PlayerInteractionType::None,
                interaction_trigger_at: 0.
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
    let default_weapon_name = zombie_game.starting_weapons.starting_weapon.as_str();

    let weapon = weapons.weapons.iter().find(|w| w.name.eq(default_weapon_name)).unwrap().clone();

    let player = commands.spawn_bundle(PlayerBundle::new(default_weapon_name)).id();
        
    let weapon = commands.spawn()
        .insert_bundle(WeaponBundle::new(weapon, true)).id();

    commands.entity(player).add_child(weapon);

    if let Some(alternate_weapon) = &zombie_game.starting_weapons.starting_alternate_weapon {
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
    time: Res<Time>,
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
                interaction.entity = entity.clone();
                interaction.interaction_type = player_interaction.interaction_type;
                interaction.interaction_cooldown = player_interaction.interaction_timeout;
            } else {
                if entity.id() == interaction.entity.id() {
                    match interaction.interaction_type {
                        PlayerInteractionType::RepairWindow => {
                            if interaction.interacting == true {
                                // TODO : duplicatate code
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                interaction.interacting = false;
                                health.current_health = 0.;
                                sprite.custom_size = Some(Vec2::new(0.,0.));
                            }
                        },
                        _ => {}
                    }

                    interaction.interaction = false;
                    interaction.interacting = false;
                    interaction.entity = Entity::from_raw(0);
                }
            }
        } 


        if interaction.interaction {
            if keyboard_input.pressed(KeyCode::F) {
                match interaction.interaction_type {
                    PlayerInteractionType::RepairWindow => {
                        if interaction.interacting == true {
                            // repair the window
                            let time_since_startup = time.time_since_startup().as_secs_f32();
                            if interaction.interaction_trigger_at + interaction.interaction_cooldown <= time_since_startup {
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                sprite.custom_size = Some(size.0);
                                health.current_health = 1.;
                                interaction.interacting = false;
                            } else {
                                let (_,size, _ , mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                let time_diff = time_since_startup - (interaction.interaction_trigger_at + interaction.interaction_cooldown);
                                let percentage_time_diff_cooldown = 1. - (time_diff / interaction.interaction_cooldown);
                                sprite.custom_size = Some(size.0 / percentage_time_diff_cooldown);
                            }
                        } else {
                            let (_, children) = query_window.get(interaction.entity).unwrap();

                            for &child_entity in children.iter() {
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(child_entity).unwrap();
                                if health.current_health <= 0. {
                                    // there is a panel to repair
                                    interaction.interacting = true;
                                    interaction.child_entity = child_entity.clone();
                                    interaction.interaction_trigger_at = time.time_since_startup().as_secs_f32();
                                    break;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            } else {
                if interaction.interacting {
                    interaction.interacting = false;
                    match interaction.interaction_type {
                        PlayerInteractionType::RepairWindow => {
                            let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                            health.current_health = 0.;
                            sprite.custom_size = Some(Vec2::new(0.,0.));
                        },
                        _ => {}
                    }
                }
            }
        }
    }

}

pub fn input_player(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<(&mut Transform, &mut Player, &mut CharacterMovementState, &mut LookingAt)>,
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

    mut game_state: ResMut<State<GameState>>
) {

    // TODO : split player input and movement (IF I WANT TO NETWORK AT SOME POINT)
    for (mut player_transform, mut player, mut character_movement_state, mut looking_at) in query.iter_mut() {

        looking_at.0 = get_cursor_location(&wnds, &q_camera);

        let mut movement = Vec3::default();
        let mut moved = false;


        if keyboard_input.pressed(KeyCode::F2) {
            game_state.set(GameState::Menu).unwrap();
            return;
        }

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
            character_movement_state.state = "standing".to_string();
            return;
        }

        character_movement_state.state = "walking".to_string();

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

pub fn system_unload_players(
    mut commands: Commands,
    q_player: Query<Entity, With<Player>>
) {
    for entity in q_player.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

