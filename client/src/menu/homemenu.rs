use bevy::{prelude::*, app::AppExit};

use super::ui_utils::*;
use shared::{
    game::{GameState, ZombieGame, ZombiePlayerInformation},
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
                    add_button(ActionButtonComponent(ButtonActions::StartLocalMultiplayerGame), "Start multiplayer game", parent, &asset_server);
                    add_button(ActionButtonComponent(ButtonActions::StartLocalGame), "Start single player game", parent, &asset_server);
                });
        });
}

pub fn system_button_handle(
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
                                PlayerCurrentInput{ input_source: SupportedController::Gamepad, gamepad: Some(controller.gamepad.get(0).unwrap().clone())}
                            } else { PlayerCurrentInput{ input_source: SupportedController::Keyboard, gamepad: None}}
                        }];
                        app_state.set(GameState::PlayingZombie).unwrap();
                    },
                    ButtonActions::StartLocalMultiplayerGame => {
                        // Add a player with the keyboard and add one player by present input
                        zombie_game.players = vec![ZombiePlayerInformation {
                            name: "Player 1".to_string(),
                            controller: PlayerCurrentInput { input_source: SupportedController::Keyboard, gamepad: None }
                        }];
                        for (i, gamepad) in controller.gamepad.iter().enumerate() {
                            zombie_game.players.push(ZombiePlayerInformation {
                                name: format!("Player {}", i + 2),
                                controller: PlayerCurrentInput { input_source: SupportedController::Gamepad, gamepad: Some(gamepad.clone()) }
                            })
                        }
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
