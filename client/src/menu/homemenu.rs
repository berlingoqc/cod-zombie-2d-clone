use bevy::{prelude::*, app::AppExit};
use ggrs::UdpNonBlockingSocket;

use crate::{p2p::{local::create_local_session, online::{NetworkPlayer, create_network_session}}, config::Opts};

use super::ui_utils::*;
use shared::{
    game::{GameState, ZombieGame, ZombiePlayerInformation, GameSpeed},
    player::input::{AvailableGameController, PlayerCurrentInput, SupportedController}
};



pub fn setup_home_menu(
    mut commands: Commands, asset_server: Res<AssetServer>
) {
    commands
        .spawn()
        .insert(MenuComponent{})
        .insert_bundle(NodeBundle{
            style: Style {
                // center button
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: Rect::all(Val::Auto),
               ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn()
                .insert_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                }).with_children(|parent| {
                    add_button(ActionButtonComponent(ButtonActions::QuitApplication), "Close", parent, &asset_server);
                    add_button(ActionButtonComponent(ButtonActions::StartOnlineMultiplayerGame), "online multiplayer", parent, &asset_server);
                    add_button(ActionButtonComponent(ButtonActions::StartLocalMultiplayerGame), "local multiplayer", parent, &asset_server);
                    add_button(ActionButtonComponent(ButtonActions::StartLocalGame), "single player", parent, &asset_server);
                });
        });
}

pub fn system_button_handle(

    mut commands: Commands,
    game_speed: Res<GameSpeed>,

    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ActionButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,

    mut exit: EventWriter<AppExit>,
    mut app_state: ResMut<State<GameState>>,

    mut zombie_game: ResMut<ZombieGame>,
    controller: Res<AvailableGameController>
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                match action.0 {
                    ButtonActions::StartLocalGame => {
                        // Add player to the game
                        zombie_game.players = vec![ZombiePlayerInformation {
                            name: "Player 1".to_string(),
                            controller: if controller.gamepad.len() > 0 {
                                PlayerCurrentInput{ input_source: SupportedController::Gamepad, gamepad: Some(controller.gamepad.get(0).unwrap().clone()), ..default()}
                            } else { PlayerCurrentInput{ input_source: SupportedController::Keyboard, gamepad: None, ..default()}},
                            index: 0,
                        }];

                        create_local_session(&mut commands, &game_speed, 1);

                        app_state.set(GameState::PlayingZombie).unwrap();
                    },
                    ButtonActions::StartOnlineMultiplayerGame => {
                        let opts = Opts::get();

                        let mut players: Vec<NetworkPlayer> = vec![];

                        zombie_game.players.push(ZombiePlayerInformation {
                            name: format!("Player {}", 0),
                            controller: PlayerCurrentInput { input_source: SupportedController::Keyboard,  ..default() },
                            index: opts.index as usize,

                        });

                        zombie_game.players.push(ZombiePlayerInformation {
                            name: format!("Player {}", 1),

                            controller: PlayerCurrentInput { input_source: SupportedController::Keyboard, ..default() },
                            index: if opts.index == 0 { 1 } else { 0 },
                        });

                        // HARCODE OF THE IP AND THE ORDER
                        if opts.index == 0 {
                            players.push(NetworkPlayer{address: "localhost".to_string()});
                            players.push(NetworkPlayer{address: "127.0.0.1:7001".to_string()});
                        } else if opts.index == 1 {
                            players.push(NetworkPlayer{address: "127.0.0.1:7000".to_string()});
                            players.push(NetworkPlayer{address: "localhost".to_string()});
                        };

                        let socket = UdpNonBlockingSocket::bind_to_port(opts.port as u16).unwrap();

                        create_network_session(&mut commands, &game_speed, socket, players);

                        app_state.set(GameState::PlayingZombie).unwrap();
                    },
                    ButtonActions::StartLocalMultiplayerGame => {
                        // Add a player with the keyboard and add one player by present input
                        zombie_game.players = vec![ZombiePlayerInformation {
                            name: "Player 1".to_string(),
                            controller: PlayerCurrentInput { input_source: SupportedController::Keyboard, gamepad: None, ..default() },
                            index: 0,
                        }];
                        for (i, gamepad) in controller.gamepad.iter().enumerate() {
                            zombie_game.players.push(ZombiePlayerInformation {
                                name: format!("Player {}", i + 2),
                                controller: PlayerCurrentInput { input_source: SupportedController::Gamepad, gamepad: Some(gamepad.clone()), ..default() },
                                index: i + 1,
                            })
                        }

                        create_local_session(&mut commands, &game_speed, zombie_game.players.iter().count());

                        app_state.set(GameState::PlayingZombie).unwrap();
                    },
                    ButtonActions::QuitApplication => {
                        exit.send(AppExit);
                    },
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn clear_home_menu(
    mut commands: Commands,
    mut interaction_query: Query<
        Entity,
        With<MenuComponent>,
    >,
) {
    for entity in interaction_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}


pub struct HomeMenuPlugin {}

impl Plugin for HomeMenuPlugin{
    fn build(&self, app: &mut App) {
        app
        .add_system_set(
            SystemSet::on_enter(GameState::Menu)
                .with_system(setup_home_menu)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Menu)
                .with_system(system_button_handle)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Menu)
                .with_system(clear_home_menu)
        );
    }
}
