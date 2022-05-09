use std::time::Duration;

use crate::health::Health;
use crate::map::{WindowPanel, Window, WindowPanelBundle};
use crate::player::{setup_players, PlayerInteraction, AnimationTimer, LookingAt, CharacterMovementState };
use crate::weapons::loader::{WeaponAssetPlugin, WeaponAssetState};
use crate::weapons::weapons::{WeaponState, WeaponCurrentAction, Weapon};

use super::collider::{MovementCollider, ProjectileCollider};
use super::map::{MapElementPosition,  ZombieSpawner, render::MapDataState};
use super::player::Player;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use pathfinding::prelude::astar;

#[derive(Default)]
pub struct Game {
    pub player: Player,

    pub zombie_game: Option<ZombieGame>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    Menu,
    PlayingZombie,
    //GameOver,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[repr(i32)]
pub enum ZombieGameState {
    Initializing = 0,
    Starting,
    Round,
    RoundInterlude,
    Over,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ZombieState {
    AwakingFromTheDead,
    FindingEnterace,
    FollowingPlayer,
}




#[derive(Default, Deserialize, Clone)]
pub struct MapRoundConfiguration {
    pub starting_zombie: i32,
    pub round_increments: i32,
    pub initial_timeout: u64,
}

#[derive(Default, Deserialize, Clone)]
pub struct StartingWeapons {
    pub starting_weapon: String,
    pub starting_alternate_weapon: Option<String>,
}

#[derive(Default, Deserialize, Clone)]
pub struct WindowPanelConfiguration {
    pub interaction_timeout: f32,
    pub spacing: f32,
    pub health: f32,
    pub nbr: u32
}


#[derive(Deserialize, TypeUuid, Clone, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5023"]
pub struct ZombieLevelAsset {
    pub configuration: MapRoundConfiguration,
    pub starting_weapons: StartingWeapons,
    pub window_panel: WindowPanelConfiguration,
}


#[derive(Default)]
pub struct CurrentRoundInfo {
    pub total_zombie: i32,
    pub zombie_remaining: i32,
}


#[derive(Component, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct EntityId(pub u32);

#[derive(Component)]
pub struct ZombiePlayer {}


pub struct ZombiePlayerInformation {
    pub handle: u32,
    pub entity: u32
}

#[derive(Default)]
pub struct ZombieGame {
    pub round: i32,
    pub state: i32, //ZombieGameState,
    pub current_round: CurrentRoundInfo,

    pub configuration: MapRoundConfiguration,
    pub starting_weapons: StartingWeapons,
    pub window_panel: WindowPanelConfiguration,

    pub players: Vec<ZombiePlayerInformation>
}


pub struct ZombieGameStateChangeEvent {}
pub struct ZombieGamePanelEvent {}

#[derive(Component)]
pub struct Zombie {
    pub state: ZombieState,
}

#[derive(Component, Default)]
pub struct BotDestination {
    pub destination: MapElementPosition,
    pub path: Vec<(i32, i32)>,
    pub entity: u32
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



#[derive(Default)]
pub struct ZombieLevelAssetState {
    pub handle: Handle<ZombieLevelAsset>,
    pub loaded: bool,
}

#[derive(Default)]
pub struct ZombieLevelAssetLoader;

impl AssetLoader for ZombieLevelAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let map_data_asset = ron::de::from_bytes::<ZombieLevelAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(map_data_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.ron"]
    }
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

#[derive(Default)]
pub struct LevelMapRequested {
    pub map: String,
    pub level: String
}

pub struct ZombieSpawnerConfig {
    pub timer: Timer,
    pub nums_ndg: Vec<f32>,
}

impl FromWorld for ZombieSpawnerConfig {
    fn from_world(world: &mut World) -> Self {
        ZombieSpawnerConfig{
            timer: Timer::new(Duration::from_secs(5), true),
            nums_ndg: (-50..50).map(|x| x as f32).collect()
        }
    }
}

pub struct ZombieGamePlugin {}

impl Plugin for ZombieGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WeaponAssetPlugin{})
            .add_event::<ZombieGameStateChangeEvent>()
            .add_event::<ZombieGamePanelEvent>()
            .init_resource::<Game>()
            .init_resource::<ZombieGame>()
            .init_resource::<ZombieLevelAssetState>()
            .init_resource::<ZombieSpawnerConfig>()

            .add_system(change_game_state_event)
            .add_system(system_panel_event)

            .add_asset::<ZombieLevelAsset>()
            .init_asset_loader::<ZombieLevelAssetLoader>()

            .add_state(GameState::Menu);
    }
}

// make on client and server side ....
pub fn setup_zombie_game(
    mut state: ResMut<ZombieLevelAssetState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    requested_level: Res<LevelMapRequested>
) {
    let handle: Handle<ZombieLevelAsset> = asset_server.load(requested_level.level.as_str());
    state.handle = handle;
    state.loaded = false;

}

pub fn system_zombie_handle(
    mut commands: Commands,
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

pub fn system_zombie_game(
    mut commands: Commands,

    level_asset_state: Res<ZombieLevelAssetState>,
    custom_assets: ResMut<Assets<ZombieLevelAsset>>,
    weapon_state: Res<WeaponAssetState>,
    map_state: Res<MapDataState>,

    mut zombie_game: ResMut<ZombieGame>,

    zombie_query: Query<&Zombie>,

    time: Res<Time>,
    mut config: ResMut<ZombieSpawnerConfig>,

    mut ev_panel_event: EventWriter<ZombieGamePanelEvent>,
    

    query_spawner: Query<&MapElementPosition, With<ZombieSpawner>>,
    query_window: Query<(&MapElementPosition, Entity), With<Window>>,

    weapons: Res<WeaponAssetState>,
) {
    let mut nbr_zombie = 0;
    for _ in zombie_query.iter() {
        nbr_zombie += 1;
    }

    match unsafe { ::std::mem::transmute(zombie_game.state) } {
        ZombieGameState::Initializing => {
            let data_asset = custom_assets.get(&level_asset_state.handle);
            if level_asset_state.loaded || data_asset.is_none() {
                return;
            }
            if weapon_state.loaded == false {
                return;
            }
            if map_state.rendered == false {
                println!("waiting for map");
                return;
            }
            let data_asset = data_asset.unwrap();

            zombie_game.round = 1;
            zombie_game.configuration = data_asset.configuration.clone();
            zombie_game.starting_weapons = data_asset.starting_weapons.clone();
            zombie_game.window_panel = data_asset.window_panel.clone();
            zombie_game.current_round = CurrentRoundInfo {
                total_zombie: zombie_game.configuration.starting_zombie,
                zombie_remaining: zombie_game.configuration.starting_zombie,
            };
            config.timer = Timer::new(
                Duration::from_secs(zombie_game.configuration.initial_timeout),
                true,
            );

            // creating event
            ev_panel_event.send(ZombieGamePanelEvent{});

            // Spawn players
            setup_players(commands, &zombie_game, &weapons);

            zombie_game.state = ZombieGameState::Round as i32;
        },
        ZombieGameState::Starting => {

        },
        ZombieGameState::Round => {
            if nbr_zombie == 0 && zombie_game.current_round.zombie_remaining == 0 {
                zombie_game.state = ZombieGameState::RoundInterlude as i32;

                return;
            }

            config.timer.tick(time.delta());

            if config.timer.finished()
                && zombie_game.current_round.zombie_remaining > 0
                && nbr_zombie < 20
            {
                for position in query_spawner.iter() {
                    if zombie_game.current_round.zombie_remaining > 0 {
                        let mut ndg = rand::thread_rng();
                        config.nums_ndg.shuffle(&mut ndg);

                        let position = position.position
                            + Vec2::new(config.nums_ndg[0] as f32, config.nums_ndg[50] as f32);
                        let mut closest_window = MapElementPosition {
                            ..MapElementPosition::default()
                        };
                        let mut closest_window_entity: Entity = Entity::from_raw(0);
                        let mut closest_window_dst = 90000.;
                        for (w, entity) in query_window.iter() {
                            let distance = position.distance(w.position);
                            if distance < closest_window_dst {
                                closest_window_dst = distance;
                                closest_window = w.clone();
                                closest_window_entity = entity;
                            }
                        }

                        let goal = (
                            closest_window.position.x as i32,
                            closest_window.position.y as i32,
                        );
                        let mut result = astar(
                            &(position.x as i32, position.y as i32),
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

                        commands.spawn().insert_bundle(ZombieBundle::new(
                            MapElementPosition {
                                position,
                                size: Vec2::new(25., 25.),
                                rotation: 0,
                            },
                            BotDestination {
                                destination: closest_window.clone(),
                                entity: closest_window_entity.id(),
                                path: result,
                            },
                        ));

                        zombie_game.current_round.zombie_remaining -= 1;
                    }
                }
            }
        }
        ZombieGameState::RoundInterlude => {
            zombie_game.round += 1;
            let zombie_count = zombie_game.configuration.starting_zombie
                + ((zombie_game.round - 1) * zombie_game.configuration.round_increments);
            zombie_game.current_round = CurrentRoundInfo {
                zombie_remaining: zombie_count,
                total_zombie: zombie_count,
            };
            zombie_game.state = ZombieGameState::Round as i32;
        }
        ZombieGameState::Over => {}
    }
  
}

pub fn react_level_data(
    mut asset_events: EventReader<AssetEvent<ZombieLevelAsset>>,
    mut ev_state_change: EventWriter<ZombieGameStateChangeEvent>,

    keyboard_input: Res<Input<KeyCode>>,
) {
    for event in asset_events.iter() {
        match event {
           AssetEvent::Modified { .. } => {
               ev_state_change.send(ZombieGameStateChangeEvent {  });
           }
            _ => {}
        }
    }
    if keyboard_input.just_pressed(KeyCode::F1) {
        ev_state_change.send(ZombieGameStateChangeEvent {  });
    }
}

pub fn change_game_state_event(

    mut ev_change_state: EventReader<ZombieGameStateChangeEvent>,

    mut commands: Commands,
    mut zombie_game: ResMut<ZombieGame>,
    query_zombies: Query<Entity, With<Zombie>>,
    query_player: Query<Entity, With<Player>>,
    query_weapons: Query<Entity, With<Weapon>>,
    query_window: Query<Entity, With<Window>>,
) {
    for _ in ev_change_state.iter() {
        zombie_game.state = ZombieGameState::Initializing as i32;
        for z in query_zombies.iter() {
            commands.entity(z).despawn();
        }
        for z in query_player.iter() {
            commands.entity(z).despawn();
        }
        for z in query_weapons.iter() {
            commands.entity(z).despawn();
        }
        for z in query_window.iter() {
            commands.entity(z).despawn_descendants();
        }

    }
}

pub fn system_panel_event(
    mut ev_change_state: EventReader<ZombieGamePanelEvent>,

    mut commands: Commands,

    zombie_game: Res<ZombieGame>,
    
    mut q_window: Query<(Entity, &mut PlayerInteraction, &MapElementPosition), With<Window>>,
) {

    for _ in ev_change_state.iter() {
        for (entity, mut p_interaction, w) in q_window.iter_mut() {
            p_interaction.interaction_timeout = zombie_game.window_panel.interaction_timeout;
            for i in 0..zombie_game.window_panel.nbr {
                let panel = commands.spawn()
                    .insert_bundle(
                        WindowPanelBundle::new(w.clone(), zombie_game.window_panel.health, i, zombie_game.window_panel.spacing)
                    ).id();
                commands.entity(entity).add_child(panel);
            }
        }
    }

}

pub fn system_unload_zombie_game(
    mut zombie_game: ResMut<ZombieGame>,
) {
    zombie_game.state = 0;
}
