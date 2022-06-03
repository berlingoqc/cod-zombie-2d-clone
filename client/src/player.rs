use bevy::{prelude::*, text::Text2dBounds, transform};
use shared::{player::{MainCamera, Player, interaction::PlayerCurrentInteraction}, health::Health};


#[derive(Default, Component)]
pub struct FollowingPlayer {
    pub offset: Vec2,
    pub player: u32
}

#[derive(Default, Component)]
pub struct PlayerInteractionText {
}

#[derive(Default, Component)]
pub struct HealthBar {}


pub fn setup_player_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());
}


pub fn system_healthbar(
    mut q_healthbar: Query<&mut Text, With<HealthBar>>,
    q_player: Query<&Health, With<Player>>,
) {
    for health in q_player.iter() {
        // Get le health bar for a player
    }
}


pub fn system_player_added(
    mut commands: Commands,
    query: Query<Entity, Added<Player>>,

    asset_server: Res<AssetServer>,

    mut q_following_player: Query<(&FollowingPlayer, &mut Transform, &mut Text, Option<&HealthBar>, Option<&PlayerInteractionText>), Without<Player>>,
    q_player: Query<(&Transform, &Health, &PlayerCurrentInteraction), With<Player>>
) {

    for entity in query.iter() {
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
                ..default()
            }).insert(FollowingPlayer{ offset: Vec2::new(0., -50.), player: entity.id()}).insert(PlayerInteractionText{});
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
                ..default()
            }).insert(FollowingPlayer{ offset: Vec2::new(0., 50.), player: entity.id()}).insert(HealthBar{});
    }


    //if let Ok((player_transform, &health, player_interaction)) = q_player.get_single() {
        for (following_player, mut tranform, mut text, opt_healthbar, opt_interaction) in q_following_player.iter_mut() {

            if let Ok((player_transform, health, player_interaction)) = q_player.get(Entity::from_raw(following_player.player)) {
                tranform.translation = Vec3::new(
                    player_transform.translation.x + following_player.offset.x,
                    player_transform.translation.y + following_player.offset.y,
                    100.
                );


                if opt_healthbar.is_some() {
                    text.sections[0].value = format!("{}", health.current_health);
                }

                if opt_interaction.is_some() {
                    if player_interaction.interaction {
                        text.sections[0].value = format!("Press F to repair window")
                    } else {
                        text.sections[0].value = "".to_string();
                    }
                }
            }
        }
    //}
}




