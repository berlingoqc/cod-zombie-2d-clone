use bevy::{
    app::ScheduleRunnerSettings,
    prelude::App,
    prelude::{Component, EventReader, ResMut},
    MinimalPlugins,
};
use bevy::{math::vec2, prelude::*};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, net::Ipv4Addr, net::SocketAddr, time::Duration};
use turbulence::message_channels::ChannelMessage;
use pathfinding::prelude::astar;
use rand::prelude::SliceRandom;

use shared::game::*;
use shared::map::*;


pub fn setup_zombie_game(
    mut state: ResMut<ZombieLevelAssetState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //mut zombie_game: ResMut<ZombieGame>,
) {
    let handle: Handle<ZombieLevelAsset> = asset_server.load("level_1.level");
    state.handle = handle;
    state.loaded = false;

    commands.insert_resource(ZombieSpawnerConfig {
        timer: Timer::new(Duration::from_secs(5), true),
        nums_ndg: (-50..50).map(|x| x as f32).collect(),
    });
}

pub fn system_zombie_game(
    mut commands: Commands,

    level_asset_state: Res<ZombieLevelAssetState>,
    custom_assets: ResMut<Assets<ZombieLevelAsset>>,

    mut zombie_game: ResMut<ZombieGame>,

    zombie_query: Query<&Zombie>,

    time: Res<Time>,
    mut config: ResMut<ZombieSpawnerConfig>,

    query_spawner: Query<&MapElementPosition, With<ZombieSpawner>>,
    query_window: Query<&MapElementPosition, With<shared::map::Window>>,
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
        ZombieGameState::Starting => {
            // Waiting for all player to be ready to start the game.

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
    /*let mut text = query.single_mut();
    text.sections[0].value = format!("Round: {} \n", zombie_game.round);
    text.sections[1].value = format!(
        "Remaining: {} ",
        zombie_game.current_round.zombie_remaining + nbr_zombie
    );*/
}