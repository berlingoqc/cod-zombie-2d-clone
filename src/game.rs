use std::time::Duration;

use bevy::{prelude::*};
use crate::player::Player;
use crate::map::data::{ProjectileCollider, MovementCollider, MapElementPosition, ZombieSpawner, Window};

use pathfinding::prelude::{Grid, astar};



#[derive(Default)]
pub struct Game {
    pub player: Player,

    pub zombie_game: Option<ZombieGame>
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Menu,
    PlayingZombie,
    GameOver
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[repr(i32)]
pub enum ZombieGameState {
    Starting = 0,
    Round,
    RoundInterlude,
    Over
}

#[derive(Default)]
pub struct MapRoundConfiguration {
    pub starting_zombie: i32,
    pub round_increments: i32
}

#[derive(Default)]
pub struct CurrentRoundInfo {
    pub total_zombie: i32,
    pub zombie_remaining: i32
}

#[derive(Default)]
pub struct ZombieGame {
    pub round: i32,
    pub state: i32,//ZombieGameState,
    pub current_round: CurrentRoundInfo,
    pub configuration: MapRoundConfiguration,
}

#[derive(Component, Default)]
pub struct Zombie {}


#[derive(Component, Default)]
pub struct BotDestination {
    pub destination: MapElementPosition,
    pub path: Vec<(i32, i32)>
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

impl ZombieBundle {
    fn new(info: MapElementPosition, dest: BotDestination) -> ZombieBundle {
        ZombieBundle{
            sprite_bundle: SpriteBundle{
                sprite: Sprite {
                    color: Color::rgb(1., 1., 1.),
                    custom_size: Some(info.size),
                        ..Sprite::default()
                    },
                    transform: Transform {
                       translation: info.position.extend(10.0),
                       ..Transform::default()
                    },
             
                        ..SpriteBundle::default()
                    },
                    collider: MovementCollider{},
                    projectile_collider: ProjectileCollider{},
                    zombie: Zombie{},
                    info,
                    destination: dest
                }
            }
}


pub struct ZombieSpawnerConfig {
    timer: Timer,
}


pub fn setup_zombie_game(
    mut commands: Commands, asset_server: Res<AssetServer>, game: Res<Game>, mut zombie_game: ResMut<ZombieGame>
) {
    zombie_game.round = 1;
    zombie_game.configuration = MapRoundConfiguration{
        starting_zombie: 10,
        round_increments: 5
    };
    zombie_game.current_round = CurrentRoundInfo{
        total_zombie: zombie_game.configuration.starting_zombie,
        zombie_remaining: zombie_game.configuration.starting_zombie
    };


    commands.insert_resource(ZombieSpawnerConfig{
        timer: Timer::new(Duration::from_secs(5), true)
    });

    commands
       .spawn_bundle(TextBundle {
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
    mut query_zombies: Query<(&mut Transform, &mut BotDestination), With<Zombie>>,
) {
    let player = query_player.get_single().unwrap();
    for (mut pos,mut dest) in query_zombies.iter_mut() {
       if let Some(el) = dest.path.pop() {
            pos.translation.x = el.0 as f32;
            pos.translation.y = el.1 as f32;
       }

       if dest.path.len() == 0 {
           // change target for the user
           let goal = (player.translation.x as i32, player.translation.y as i32);
           let mut result = astar(&(pos.translation.x as i32, pos.translation.y as i32),
            |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
                  (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)]
                  .into_iter().map(|p| (p, 1)),
            |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
            |&p| p == goal).unwrap().0;

           result.reverse();
           let len = result.len() as f32;
           dest.path = if len >= 10. {
                let len_taken = ((len  * 0.25)).ceil() as usize;
                let start = result.len() - 1 - len_taken;
                let end = result.len() - 1;
                result[start..end].to_vec()
           } else {
               result
           };
       }
    }
}

pub fn system_zombie_game(
    mut commands: Commands,

    game: Res<Game>,
    mut zombie_game: ResMut<ZombieGame>,

    mut query: Query<&mut Text>,
    mut zombie_query: Query<&Zombie>,

    time: Res<Time>,
    mut config: ResMut<ZombieSpawnerConfig>,

    query_spawner: Query<&MapElementPosition, With<ZombieSpawner>>,
    query_window: Query<&MapElementPosition, With<Window>>,

    mut app_state: ResMut<State<GameState>>
) {
    let mut nbr_zombie = 0;
    for _ in zombie_query.iter() {
        nbr_zombie += 1;
    }


    match unsafe { ::std::mem::transmute(zombie_game.state) } {
        ZombieGameState::Starting => {
            zombie_game.state = ZombieGameState::Round as i32;
        },
        ZombieGameState::Round => {
            if nbr_zombie == 0 && zombie_game.current_round.zombie_remaining == 0 {
                zombie_game.state = ZombieGameState::RoundInterlude as i32;

                return;
            }

            config.timer.tick(time.delta());

            if config.timer.finished() && zombie_game.current_round.zombie_remaining > 0 && nbr_zombie < 20 {
                for position in query_spawner.iter() {
                    if zombie_game.current_round.zombie_remaining > 0 {
                        let mut closest_window = MapElementPosition{..MapElementPosition::default()};
                        let mut closest_window_dst = 90000.;
                        for w in query_window.iter() {
                            let distance = position.position.distance(w.position);
                            if distance < closest_window_dst {
                                closest_window_dst = distance;
                                closest_window = w.clone();
                            }
                        }

                        let goal = (closest_window.position.x as i32, closest_window.position.y as i32);
                        let mut result = astar(&(position.position.x as i32, position.position.y as i32),
                            |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
                                  (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)]
                                  .into_iter().map(|p| (p, 1)),
                            |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
                            |&p| p == goal).unwrap().0;

                        result.reverse();

                        commands.spawn().insert_bundle(ZombieBundle::new(MapElementPosition{
                            position: position.position,
                            size: Vec2::new(25.,25.),
                            rotation: 0
                        }, BotDestination { destination: closest_window.clone(), path: result }));


                        zombie_game.current_round.zombie_remaining -= 1;
                    }
                }
            }
        },
        ZombieGameState::RoundInterlude => {
            zombie_game.round += 1;
            let zombie_count = zombie_game.configuration.starting_zombie + ((zombie_game.round - 1) * zombie_game.configuration.round_increments);
            zombie_game.current_round = CurrentRoundInfo {
               zombie_remaining: zombie_count,
               total_zombie: zombie_count
            };
            zombie_game.state = ZombieGameState::Round as i32;
        },
        ZombieGameState::Over => {

        }
    }
    let mut text = query.single_mut();
    text.sections[0].value = format!("Round: {} \n", zombie_game.round);
    text.sections[1].value = format!("Remaining: {} ",  zombie_game.current_round.zombie_remaining + nbr_zombie);
}
