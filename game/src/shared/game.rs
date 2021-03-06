use std::time::Duration;

use crate::shared::health::Health;
use crate::shared::map::{Window, WindowPanelBundle};
use crate::shared::player::PlayerDeadEvent;
use crate::shared::player::input::{AvailableGameController, PlayerCurrentInput, FrameCount, BoxInput};
use crate::shared::player::{
    setup_player,
    interaction::PlayerInteraction
};
use crate::shared::utils::Checksum;
use crate::shared::weapons::loader::{WeaponAssetPlugin, WeaponAssetState};
use crate::shared::weapons::weapons::Weapon;

use super::map::{MapElementPosition,  ZombieSpawner, render::MapDataState};
use super::player::Player;
use super::zombies::spawner::*;
use super::zombies::zombie::*;


use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_ggrs::{RollbackIdProvider, Rollback};
use ggrs::InputStatus;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use pathfinding::prelude::astar;

#[derive(Component)]
pub struct GameSpeed(pub f32, pub usize);

impl Default for GameSpeed {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        return GameSpeed(1.0 / 60.0, 60);
        #[cfg(not(target_arch = "wasm32"))]
        return GameSpeed(1.0 / 60.0, 60);
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    Menu,
    OnlineMenu,
    PlayingZombie,
    GameOver,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Reflect)]
#[repr(i32)]
pub enum ZombieGameState {
    #[default]
    Initializing = 0,
    Starting,
    Round,
    RoundInterlude,
    Over,
}



#[derive(Default, Deserialize, Clone, Debug, Reflect)]
pub struct MapRoundConfiguration {
    pub starting_zombie: i32,
    pub round_increments: i32,
    pub initial_timeout: u64,
}

#[derive(Default, Deserialize, Clone, Debug, Reflect)]
pub struct StartingWeapons {
    pub starting_weapon: String,
    pub starting_alternate_weapon: Option<String>,
}

#[derive(Default, Deserialize, Clone, Debug, Reflect)]
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


#[derive(Default, Debug, Reflect)]
pub struct CurrentRoundInfo {
    pub total_zombie: i32,
    pub zombie_remaining: i32,
}


#[derive(Component, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct EntityId(pub u32);

#[derive(Component)]
pub struct ZombiePlayer {}


#[derive(Debug)]
pub struct ZombiePlayerInformation {
    pub name: String,
    pub controller: PlayerCurrentInput,
    pub index: usize,
    pub is_local: bool,
}

#[derive(Default, Debug, Reflect, Component)]
pub struct ZombieGame {
    pub round: i32,
    pub state: ZombieGameState,
    pub current_round: CurrentRoundInfo,
}


#[derive(Default, Debug)]
pub struct ZombieGameConfig {
    pub configuration: MapRoundConfiguration,
    pub starting_weapons: StartingWeapons,
    pub window_panel: WindowPanelConfiguration,

    pub players: Vec<ZombiePlayerInformation>
}


pub struct ZombieGameStateChangeEvent {}
pub struct ZombieGamePanelEvent {}
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
#[derive(Default)]
pub struct LevelMapRequested {
    pub map: String,
    pub level: String
}

pub struct ZombieGamePlugin {}

impl Plugin for ZombieGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WeaponAssetPlugin{})
            .add_event::<ZombieGameStateChangeEvent>()
            .add_event::<ZombieGamePanelEvent>()
            .add_event::<PlayerDeadEvent>()
            .init_resource::<GameSpeed>()
            .init_resource::<ZombieGameConfig>()
            .init_resource::<ZombieLevelAssetState>()
            .init_resource::<ZombieSpawnerConfig>()

            .add_system(change_game_state_event)
            .add_system(system_panel_event)

            .add_asset::<ZombieLevelAsset>()
            .init_asset_loader::<ZombieLevelAssetLoader>()

            .add_state(GameState::Menu);
    }
}

#[allow(dead_code)]
pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>, inputs: Res<Vec<(BoxInput, InputStatus)>>,) {
    frame_count.frame += 1;
}

// make on client and server side ....
pub fn setup_zombie_game(
    mut state: ResMut<ZombieLevelAssetState>,
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<AssetServer>,
    requested_level: Res<LevelMapRequested>,
) {
    let handle: Handle<ZombieLevelAsset> = asset_server.load(requested_level.level.as_str());
    state.handle = handle;
    state.loaded = false;


    commands.spawn().insert(ZombieGame{
        ..Default::default()
    }).insert(Checksum::default()).insert(Rollback::new(rip.next_id()));


}

pub fn system_end_game(
    q_player: Query<&Health, With<Player>>,
    mut ev_player_dead: EventReader<PlayerDeadEvent>,
    mut game_state: ResMut<State<GameState>>,

) {

    for ev in ev_player_dead.iter() {
        if q_player.iter().map(|x| x.current_health <= 0.).filter(|x| !x).count() == 0 {
            game_state.set(GameState::Menu).unwrap();
        }
    }
}

pub fn system_zombie_game(
    mut commands: Commands,

    level_asset_state: Res<ZombieLevelAssetState>,
    custom_assets: ResMut<Assets<ZombieLevelAsset>>,
    weapon_state: Res<WeaponAssetState>,
    map_state: Res<MapDataState>,

    mut q_zombie_game: Query<&mut ZombieGame>,
    mut zombie_game_config: ResMut<ZombieGameConfig>,

    zombie_query: Query<&Zombie>,

    time: Res<Time>,
    mut config: ResMut<ZombieSpawnerConfig>,

    mut ev_panel_event: EventWriter<ZombieGamePanelEvent>,
    

    query_spawner: Query<&MapElementPosition, With<ZombieSpawner>>,
    query_window: Query<(&MapElementPosition, Entity), With<Window>>,

    mut rip: ResMut<RollbackIdProvider>,

    

    weapons: Res<WeaponAssetState>,
) {
    let mut zombie_game = q_zombie_game.get_single_mut().unwrap();
    let mut nbr_zombie = 0;
    for _ in zombie_query.iter() {
        nbr_zombie += 1;
    }

    match zombie_game.state {
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
            zombie_game.current_round = CurrentRoundInfo {
                total_zombie: zombie_game_config.configuration.starting_zombie,
                zombie_remaining: zombie_game_config.configuration.starting_zombie,
            };

            zombie_game_config.configuration = data_asset.configuration.clone();
            zombie_game_config.starting_weapons = data_asset.starting_weapons.clone();
            zombie_game_config.window_panel = data_asset.window_panel.clone();
 
            config.timer = Timer::new(
                Duration::from_millis(zombie_game_config.configuration.initial_timeout),
                true,
            );

            // creating event
            ev_panel_event.send(ZombieGamePanelEvent{});

            // Spawn players
            for player in zombie_game_config.players.iter() {
                setup_player(&mut rip,&mut commands, &zombie_game_config, &weapons, player, player.index);
            }

            zombie_game.state = ZombieGameState::Round;
        },
        ZombieGameState::Starting => {

        },
        ZombieGameState::Round => {
            if nbr_zombie == 0 && zombie_game.current_round.zombie_remaining == 0 {
                zombie_game.state = ZombieGameState::RoundInterlude;

                return;
            }

            config.timer.tick(time.delta());

            if config.timer.finished()
                && zombie_game.current_round.zombie_remaining > 0
                && nbr_zombie < 20
            {
                // TODO add better option to disable zombie spawning
                for position in query_spawner.iter() {
                    if zombie_game.current_round.zombie_remaining > 0 {
                        let mut ndg = rand::thread_rng();
                        config.nums_ndg.shuffle(&mut ndg);

                        let position = position.position;
                            //+ Vec2::new(config.nums_ndg[0] as f32, config.nums_ndg[50] as f32);
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

                        let mut bot_destination = BotDestination{
                            destination: Vec2::default(),
                            entity: closest_window_entity.clone(),
                            path: vec![],
                            requested_movement: None, 
                        };

                        bot_destination.set_destination(closest_window.position, position, closest_window_entity.clone(), 0.);

                        commands.spawn().insert_bundle(ZombieBundle::new(
                            MapElementPosition {
                                position,
                                size: Vec2::new(25., 25.),
                                rotation: 0,
                            },
                            bot_destination,
                        )).insert(Rollback::new(rip.next_id()));

                        zombie_game.current_round.zombie_remaining -= 1;

                        info!("Spawning zombie , {} remaining", zombie_game.current_round.zombie_remaining);
                    }
                }
            }
        }
        ZombieGameState::RoundInterlude => {
            zombie_game.round += 1;
            let zombie_count = zombie_game_config.configuration.starting_zombie
                + ((zombie_game.round - 1) * zombie_game_config.configuration.round_increments);
            zombie_game.current_round = CurrentRoundInfo {
                zombie_remaining: zombie_count,
                total_zombie: zombie_count,
            };
            zombie_game.state = ZombieGameState::Round;
            config.timer.reset();
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
    mut q_zombie_game: Query<&mut ZombieGame>,
    query_zombies: Query<Entity, With<Zombie>>,
    query_player: Query<Entity, With<Player>>,
    query_weapons: Query<Entity, With<Weapon>>,
    query_window: Query<Entity, With<Window>>,
) {
    for _ in ev_change_state.iter() {
        let mut zombie_game = q_zombie_game.get_single_mut().unwrap();
        zombie_game.state = ZombieGameState::Initializing;
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

    zombie_game: Res<ZombieGameConfig>,
    
    mut q_window: Query<(Entity, &mut Health, &mut PlayerInteraction, &MapElementPosition), With<Window>>,
) {

    let window_panel_health = 1.0;

    for _ in ev_change_state.iter() {
        for (entity, mut health, mut p_interaction, w) in q_window.iter_mut() {
            p_interaction.interaction_timeout = zombie_game.window_panel.interaction_timeout;
            for i in 0..zombie_game.window_panel.nbr {
                let panel = commands.spawn()
                    .insert_bundle(
                        WindowPanelBundle::new(w.clone(), i, zombie_game.window_panel.spacing)
                    ).id();
                commands.entity(entity).add_child(panel);
            }
            health.max_health = window_panel_health * (zombie_game.window_panel.nbr as f32);
            health.current_health = health.max_health;
            health.tmp_health = health.max_health;
        }
    }

}

pub fn system_unload_zombie_game(
    mut commands: Commands,
    q_zombiegame: Query<Entity, With<ZombieGame>>,
) {
    let entity = q_zombiegame.get_single().unwrap();
    commands.entity(entity).despawn();
}
