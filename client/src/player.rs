use bevy::{prelude::*, text::Text2dBounds};
use shared::{player::{MainCamera, Player, interaction::PlayerCurrentInteraction}};


#[derive(Default, Component)]
pub struct FollowingPlayer {}

#[derive(Default, Component)]
pub struct PlayerInteractionText {}


pub fn setup_player_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());
}


pub fn system_player_added(
    mut commands: Commands,
    query: Query<&Transform, Added<Player>>,

    asset_server: Res<AssetServer>,

    mut q_following_player: Query<(&mut Transform, &mut Text, Option<&PlayerInteractionText>), (With<FollowingPlayer>, Without<Player>)>,
    q_player: Query<(&Transform, &PlayerCurrentInteraction), With<Player>>
) {

    for transform in query.iter() {
        println!("Player spawn");
        commands.spawn().insert_bundle(Text2dBundle {
                text: Text::with_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    TextAlignment { vertical: VerticalAlign::Center, horizontal: HorizontalAlign::Center },
                ),
                text_2d_bounds: Text2dBounds {
                   // Wrap text in the rectangle
                    size: Size::new(100.0, 30.0),
                },
                transform: Transform {
                    translation: Vec3::new(
                        transform.translation.x,
                        transform.translation.y - 50.,
                        transform.translation.z
                    ),
                    ..default()
                },
                ..default()
            }).insert(FollowingPlayer{}).insert(PlayerInteractionText{});
    }


    if let Ok((player_transform, player_interaction)) = q_player.get_single() {
        for (mut tranform, mut text, opt_interaction) in q_following_player.iter_mut() {
            tranform.translation = Vec3::new(
                player_transform.translation.x,
                player_transform.translation.y - 50.,
                player_transform.translation.z
            );

            if opt_interaction.is_some() && player_interaction.interaction {
                text.sections[0].value = format!("Press F to repair window")
            } else {
                text.sections[0].value = "".to_string();
            }
        }
    }
}




