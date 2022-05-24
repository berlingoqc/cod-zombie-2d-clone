pub mod interaction;

use bevy::{prelude::*, math::const_vec2};

use crate::{
    collider::{MovementCollider, is_colliding},
    game::{ZombieGame, GameState, GameSpeed},
    map::{MapElementPosition},
    utils::get_cursor_location, weapons::{weapons::{WeaponBundle}, loader::WeaponAssetState}, animation::AnimationTimer, character::{LookingAt, CharacterMovementState}
};

use self::interaction::{PlayerCurrentInteraction, PlayerInteractionType};


pub const PLAYER_SIZE: Vec2 = const_vec2!([25., 25.]);

#[derive(Component)]
pub struct MainCamera;

#[derive(Default, Component)]
pub struct Player {
    pub active_weapon_name: String,
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
    //pub velocity: Velocity,
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
            movement_collider: MovementCollider {
                size: PLAYER_SIZE,
            },
            map_element_position: MapElementPosition { position: Vec2::new(0.0, 0.), size: Vec2::new(50., 50.), rotation: 0 },
            // velocity: Velocity { v: Vec2::new(0.,0.)},
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


pub fn input_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player, &mut CharacterMovementState, &mut LookingAt)>,
    collider_query: Query<
        (Entity, &Transform, &MovementCollider),
        Without<Player>,
    >,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

    game_speed: Res<GameSpeed>,

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

        let dest = player_transform.translation + (movement * game_speed.0 * 125.);

        if !is_colliding(dest, PLAYER_SIZE, &collider_query) {
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

