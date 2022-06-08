pub mod interaction;
pub mod input;

use bevy::{prelude::*, math::const_vec2};
use bevy_ggrs::{Rollback, RollbackIdProvider};

use crate::{
    collider::{MovementCollider, is_colliding},
    game::{ZombieGame, GameState, GameSpeed, ZombiePlayerInformation},
    map::{MapElementPosition},
    utils::get_cursor_location, weapons::{weapons::{WeaponBundle, ActiveWeapon}, loader::WeaponAssetState}, animation::AnimationTimer, character::{LookingAt, CharacterMovementState, Death}, health::{Health, HealthChangeState, HealthRegeneration}
};

use self::{interaction::{PlayerCurrentInteraction, PlayerInteractionType}, input::{PlayerCurrentInput, AvailableGameController}};


pub const PLAYER_SIZE: Vec2 = const_vec2!([25., 25.]);

const SPAWN_OFFSET_0: Vec2 = const_vec2!([-50., 50.]);
const SPAWN_OFFSET_1: Vec2 = const_vec2!([50., 50.]);
const SPAWN_OFFSET_2: Vec2 = const_vec2!([-50., -50.]);
const SPAWN_OFFSET_3: Vec2 = const_vec2!([50., -50.]);

fn get_spawn_offset(player_index: usize) -> Vec2 {
    return match player_index {
        0=> SPAWN_OFFSET_0,
        1=> SPAWN_OFFSET_1,
        2=> SPAWN_OFFSET_2,
        3=> SPAWN_OFFSET_3,
        _=> Vec2::default(),
    };
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Default, Component)]
pub struct Player {
    handle: usize,
}

pub struct PlayerDeadEvent {
    pub player: Entity,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    #[bundle] 
    pub sprite: SpriteSheetBundle,
    pub interaction: PlayerCurrentInteraction,
    pub looking_direction: LookingAt,
    pub animation_timer: AnimationTimer,
    pub map_element_position: MapElementPosition,
    pub movement_collider: MovementCollider,
    pub health: Health,
    pub health_regeneration: HealthRegeneration,
    pub character_movement_state: CharacterMovementState,

    pub player_current_input: PlayerCurrentInput,
}

impl PlayerBundle {
    fn new(starting_weapon_name: &str, input: PlayerCurrentInput, index_player: usize) -> PlayerBundle {
        PlayerBundle { 
            player: Player{
                handle: index_player,
                ..default()
            },
            player_current_input: input,
            sprite : SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 10.) + get_spawn_offset(index_player).extend(0.),
                    ..Transform::default()
                },
                ..default()
            },
            movement_collider: MovementCollider {
                size: PLAYER_SIZE,
                ..default()
            },
            map_element_position: MapElementPosition { position: Vec2::new(0.0, 0.), size: Vec2::new(50., 50.), rotation: 0 },
            // velocity: Velocity { v: Vec2::new(0.,0.)},
            character_movement_state: CharacterMovementState { state: String::from("walking"), sub_state: "".to_string() },
            looking_direction: LookingAt(Vec2::new(0., 0.), false),
            animation_timer: AnimationTimer{ 
                timer: Timer::from_seconds(0.1, true),
                index: 0,
                offset: 0,
                asset_type: "player".to_string(),
                current_state: "".to_string(),
            },
            health: Health { current_health: 3., tmp_health: 3., max_health: 3., ..default() },
            health_regeneration: HealthRegeneration{
                timeout_regeneration: 2.,
                regeneration_amount: 1.,
                timer: None,
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

pub fn setup_player(
    mut rip: &mut ResMut<RollbackIdProvider>,
    mut commands: &mut Commands,
    zombie_game: &ResMut<ZombieGame>,
    weapons: &Res<WeaponAssetState>,
    
    config: &ZombiePlayerInformation,

    index_player: usize
) {
    // TODO: for multiplayer
    // Fetch the location of the player spawner in the map
    // Use your player index in the player array of the game
    // to select your color and where your spawn

    // get the default weapon for the map

    info!("Initializing player {:?}", config.name);
    let default_weapon_name = zombie_game.starting_weapons.starting_weapon.as_str();

    let weapon = weapons.weapons.iter().find(|w| w.name.eq(default_weapon_name)).unwrap().clone();

    let player = commands.spawn_bundle(PlayerBundle::new(default_weapon_name, config.controller.clone(), index_player)).id();

    commands.entity(player).insert(Rollback::new(rip.next_id()));

    let weapon = commands.spawn()
        .insert_bundle(WeaponBundle::new(weapon)).insert(ActiveWeapon{}).id();

    commands.entity(player).add_child(weapon);

    if let Some(alternate_weapon) = &zombie_game.starting_weapons.starting_alternate_weapon {
        let weapon = weapons.weapons.iter().find(|w| w.name.eq(alternate_weapon.as_str())).unwrap().clone();
        let weapon = commands.spawn()
            .insert_bundle(WeaponBundle::new(weapon)).id();
        commands.entity(player).add_child(weapon);
    }
}



pub fn system_health_player(
	mut q_player_health: Query<(Entity, &mut Health, &mut HealthRegeneration, &mut CharacterMovementState, &mut AnimationTimer, &mut Transform), (With<Player>)>,

    mut game_state: ResMut<State<GameState>>,

    mut commands: Commands,

    time: Res<Time>,

    mut ev_player_dead: EventWriter<PlayerDeadEvent>,

) {
    for (entity, mut health, mut regeneration, mut character_movement_state, mut timer, mut transform) in q_player_health.iter_mut() {
        match health.get_health_change_state() {
            HealthChangeState::GainHealth => {
                health.apply_change();
            },
            HealthChangeState::LostHealth => {
                health.apply_change();
                regeneration.on_health_change();
            },
            HealthChangeState::Dead => {
                health.current_health = 0.;
                
                character_movement_state.state = "dying".to_string();
                character_movement_state.sub_state = "".to_string();
                timer.offset = 0;

                transform.translation.z = 5.;
                commands.entity(entity).insert(Death{});
                ev_player_dead.send(PlayerDeadEvent { player: entity.clone() });
            },
            _ => {
                regeneration.apply_regeneration_if(time.delta(), &mut health)
            },
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

