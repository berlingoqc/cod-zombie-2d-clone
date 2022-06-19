use bevy::{prelude::*, app::AppExit};

use crate::{p2p::{online::{NetworkPlayer, create_session}}};

use super::ui_utils::*;
use shared::{
    game::{GameState, ZombieGame, ZombiePlayerInformation, GameSpeed, ZombieGameConfig},
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

    mut zombie_game: ResMut<ZombieGameConfig>,
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
                            is_local: true,
                        }];

                        let mut players: Vec<NetworkPlayer> = vec![];
                        players.push(NetworkPlayer{address: "localhost".to_string()});
                        create_session(&mut commands, &game_speed, players);

                        app_state.set(GameState::PlayingZombie).unwrap();
                    },
                    ButtonActions::StartOnlineMultiplayerGame => {
                        app_state.set(GameState::OnlineMenu).unwrap();
                    },
                    ButtonActions::StartLocalMultiplayerGame => {
                        // Add a player with the keyboard and add one player by present input
                        zombie_game.players = vec![ZombiePlayerInformation {
                            name: "Player 1".to_string(),
                            controller: PlayerCurrentInput { input_source: SupportedController::Keyboard, gamepad: None, ..default() },
                            index: 0,
                            is_local: true,
                        }];

                        let mut players: Vec<NetworkPlayer> = vec![];
                        players.push(NetworkPlayer{address: "localhost".to_string()});

                        for (i, gamepad) in controller.gamepad.iter().enumerate() {

                            players.push(NetworkPlayer{address: "localhost".to_string()});
                            zombie_game.players.push(ZombiePlayerInformation {
                                name: format!("Player {}", i + 2),
                                controller: PlayerCurrentInput { input_source: SupportedController::Gamepad, gamepad: Some(gamepad.clone()), ..default() },
                                index: i + 1,
                                is_local: true
                            })
                        }

                        create_session(&mut commands, &game_speed, players);

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
