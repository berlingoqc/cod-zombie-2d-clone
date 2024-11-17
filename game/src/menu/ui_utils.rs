use bevy::prelude::*;

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component, Default)]
pub struct MenuComponent {}


#[derive(Default)]
pub enum ButtonActions {
    #[default]
    StartLocalGame,
    StartLocalMultiplayerGame,
    StartOnlineMultiplayerGame,
    QuitApplication
}

#[derive(Component, Default)]
pub struct ActionButtonComponent(pub ButtonActions);


pub fn add_button(
    action: ActionButtonComponent,
    text: &str,
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    parent.spawn((MenuComponent{},
        action,
        ButtonBundle {
            style: Style {
                height: Val::Px(400.0),
                width: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
               ..default()
            },
            background_color: Color::rgb(0.15,0.15,0.15).into(),
            ..default()
        }))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    text,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..default()
            });
        });
}
