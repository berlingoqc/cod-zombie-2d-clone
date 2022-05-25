
use bevy::{prelude::*, app::AppExit};
use shared::game::GameState;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component, Default)]
pub struct MenuComponent {}


#[derive(Default)]
pub enum ButtonActions {
    #[default]
    StartLocalGame,
    QuitApplication
}

#[derive(Component, Default)]
pub struct ActionButtonComponent(ButtonActions);


fn add_button(
    action: ActionButtonComponent,
    text: &str,
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    parent.spawn()
        .insert(MenuComponent{})
        .insert(action)
        .insert_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(400.0), Val::Px(65.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
               ..default()
            },
            color: Color::rgb(0.15,0.15,0.15).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    text,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    default(),
                ),
                ..default()
            });
        });
}

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
    mut app_state: ResMut<State<GameState>>
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                match action.0 {
                    ButtonActions::StartLocalGame => {
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
    println!("Clearing ui");
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