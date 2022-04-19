use std::time::Duration;

use crate::map::data::{
    MapElementPosition, MovementCollider, ProjectileCollider, Window, ZombieSpawner,
};
use crate::player::Player;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use rand::prelude::*;
use serde::Deserialize;

use pathfinding::prelude::{astar, Grid};

#[derive(Default)]
pub struct Game {
    pub player: Player,

    pub zombie_game: Option<ZombieGame>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Menu,
    PlayingZombie,
    GameOver,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[repr(i32)]
pub enum ZombieGameState {
    Starting = 0,
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

#[derive(Default)]
pub struct CurrentRoundInfo {
    pub total_zombie: i32,
    pub zombie_remaining: i32,
}

#[derive(Default)]
pub struct ZombieGame {
    pub round: i32,
    pub state: i32, //ZombieGameState,
    pub current_round: CurrentRoundInfo,
    pub configuration: MapRoundConfiguration,
}

#[derive(Component)]
pub struct Zombie {
    pub state: ZombieState,
}

#[derive(Component, Default)]
pub struct BotDestination {
    pub destination: MapElementPosition,
    pub path: Vec<(i32, i32)>,
}

#[derive(Bundle)]
pub struct ZombieBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: MovementCollider,
    destination: BotDestination,
    projectile_collider: ProjectileCollider,
    info: MapElementPosition,
    zombie: Zombie,
}

#[derive(Deserialize, TypeUuid, Clone, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5023"]
pub struct ZombieLevelAsset {
    pub configuration: MapRoundConfiguration,
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
        &["level"]
    }
}

impl ZombieBundle {
    fn new(info: MapElementPosition, dest: BotDestination) -> ZombieBundle {
        ZombieBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1., 1., 1.),
                    custom_size: Some(info.size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: info.position.extend(10.0),
                    scale: Vec3::new(0., 0., 0.),
                    ..Transform::default()
                },

                ..SpriteBundle::default()
            },
            collider: MovementCollider {},
            projectile_collider: ProjectileCollider {},
            zombie: Zombie {
                state: ZombieState::AwakingFromTheDead,
            },
            info,
            destination: dest,
        }
    }
}

pub struct ZombieSpawnerConfig {
    timer: Timer,
    nums_ndg: Vec<f32>,
}

pub fn setup_zombie_game(
    mut state: ResMut<ZombieLevelAssetState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut zombie_game: ResMut<ZombieGame>,
) {
    let handle: Handle<ZombieLevelAsset> = asset_server.load("level_1.level");
    state.handle = handle;
    state.loaded = false;

    commands.insert_resource(ZombieSpawnerConfig {
        timer: Timer::new(Duration::from_secs(5), true),
        nums_ndg: (-50..50).map(|x| x as f32).collect(),
    });

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Round: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: "Remaining: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            ..default()
        },
        ..default()
    });
}

pub fn system_zombie_handle(
    query_player: Query<&Transform, (With<Player>, Without<Zombie>)>,
    mut config: ResMut<ZombieSpawnerConfig>,
    mut query_zombies: Query<(&mut Transform, &mut BotDestination, &mut Zombie), With<Zombie>>,
) {
    let player = query_player.get_single().unwrap();
    for (mut pos, mut dest, mut zombie) in query_zombies.iter_mut() {
        match zombie.state {
            ZombieState::AwakingFromTheDead => {
                if pos.scale.x < 1.0 {
                    let mut rng = rand::thread_rng();
                    config.nums_ndg.shuffle(&mut rng);
                    pos.scale += Vec3::new(0.01, 0.01, 0.01);
                    pos.rotation = Quat::from_rotation_z(config.nums_ndg[75] / 10.);
                } else {
                    pos.rotation = Quat::from_rotation_z(0.);
                    zombie.state = ZombieState::FindingEnterace;
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

    game: Res<Game>,
    mut zombie_game: ResMut<ZombieGame>,

    mut query: Query<&mut Text>,
    mut zombie_query: Query<&Zombie>,

    time: Res<Time>,
    mut config: ResMut<ZombieSpawnerConfig>,

    query_spawner: Query<&MapElementPosition, With<ZombieSpawner>>,
    query_window: Query<&MapElementPosition, With<Window>>,

    mut app_state: ResMut<State<GameState>>,
) {
    let mut nbr_zombie = 0;
    for _ in zombie_query.iter() {
        nbr_zombie += 1;
    }

    match unsafe { ::std::mem::transmute(zombie_game.state) } {
        ZombieGameState::Starting => {
            let data_asset = custom_assets.get(&level_asset_state.handle);
            if level_asset_state.loaded || data_asset.is_none() {
                return;
            }
            let data_asset = data_asset.unwrap();

            zombie_game.round = 1;
            zombie_game.configuration = data_asset.configuration.clone();
            zombie_game.current_round = CurrentRoundInfo {
                total_zombie: zombie_game.configuration.starting_zombie,
                zombie_remaining: zombie_game.configuration.starting_zombie,
            };
            config.timer = Timer::new(
                Duration::from_secs(zombie_game.configuration.initial_timeout),
                true,
            );

            zombie_game.state = ZombieGameState::Round as i32;
        }
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
                        let mut closest_window_dst = 90000.;
                        for w in query_window.iter() {
                            let distance = position.distance(w.position);
                            if distance < closest_window_dst {
                                closest_window_dst = distance;
                                closest_window = w.clone();
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
    let mut text = query.single_mut();
    text.sections[0].value = format!("Round: {} \n", zombie_game.round);
    text.sections[1].value = format!(
        "Remaining: {} ",
        zombie_game.current_round.zombie_remaining + nbr_zombie
    );
}

pub fn react_level_data(
    mut asset_events: EventReader<AssetEvent<ZombieLevelAsset>>,
    custom_assets: ResMut<Assets<ZombieLevelAsset>>,
    mut zombie_game: ResMut<ZombieGame>,
    mut commands: Commands,
    mut query_zombies: Query<Entity, With<Zombie>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Modified { handle } => {
                zombie_game.state = ZombieGameState::Starting as i32;
                for z in query_zombies.iter() {
                    commands.entity(z).despawn();
                }
            }
            _ => {}
        }
    }
}
