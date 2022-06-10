

use bevy::prelude::*;
use shared::{
    game::ZombieGame,
    weapons::weapons::{AmmunitionState, Weapon, WeaponState, WeaponCurrentAction, ActiveWeapon},
    zombies::zombie::Zombie, player::Player,
};

#[derive(Component)]
pub struct InGameUI;

#[derive(Component)]
pub struct PlayerUI {
    pub player: Entity,
}

#[derive(Component)]
pub struct RoundText;

#[derive(Component)]
pub struct WeaponText {}

#[derive(Component)]
pub struct WeaponUiImage;

pub fn setup_ingame_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

) {
    // Text bundle for the round info 
	  commands.spawn().insert(InGameUI{}).insert_bundle(TextBundle {
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
    }).insert(RoundText{});
}

pub fn system_ingame_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    zombie_game: Res<ZombieGame>,
    zombie_query: Query<&Zombie>,
    mut query_round: Query<&mut Text, With<RoundText>>,

    player_added: Query<Entity, Added<Player>>,
) {
    let mut nbr_zombie = 0;
    for _ in zombie_query.iter() {
        nbr_zombie += 1;
    }

	let mut text = query_round.single_mut();
    text.sections[0].value = format!("Round: {} \n", zombie_game.round);
    text.sections[1].value = format!(
        "Remaining: {} ",
        zombie_game.current_round.zombie_remaining + nbr_zombie
    );


    for (i, entity) in player_added.iter().enumerate() {
        println!("Creating one set of ui");
        commands.spawn().insert(InGameUI{}).insert(PlayerUI{ player: entity.clone()}).insert_bundle(NodeBundle {
            style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        bottom: Val::Px(5.0 + ((i as f32) * 75.)),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
                color: UiColor(Color::NONE),
            ..default()
        }).with_children(|parent| {

            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(25.), Val::Auto),
                    ..default()
                },
                ..default()
            }).insert(WeaponUiImage{});

            parent.spawn_bundle(TextBundle{
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
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
            }).insert(WeaponText{});

        });
    }
}

pub fn system_weapon_ui(
    query_player: Query<&Children, With<Player>>,
	query_player_weapon: Query<(&AmmunitionState, &Weapon, &WeaponState), With<ActiveWeapon>>,
    query_player_ui: Query<(Entity, &Children, &PlayerUI)>,

    mut query_ammo_text: Query<&mut Text, With<WeaponText>>,
    mut query_weapon_image: Query<&mut UiImage, With<WeaponUiImage>>,

    asset_server: Res<AssetServer>,
) {
    for (_, childrens_ui, player_ui) in query_player_ui.iter() {
            if let Ok(childrens_player) = query_player.get(player_ui.player) {

                for children in childrens_player.iter() {
                    if let Ok((ammo_state, weapon, weapon_state)) = query_player_weapon.get(*children) {

                        for children in childrens_ui.iter() {
                            if let Ok(mut text) = query_ammo_text.get_mut(*children) {
							    text.sections[0].value = format!("{}\n-\n{}", ammo_state.mag_remaining, ammo_state.remaining_ammunition);
                            }
                            if let Ok(mut weapon_image) = query_weapon_image.get_mut(*children) {
                                // TODO: added a event of weapon change to trigger this instead of every frame lol
                                weapon_image.0 = asset_server.load(weapon.asset_name.as_str());
                            } 
                        }
                    }
                }
            }
    }
}

pub fn system_clear_ingame_ui(
    mut commands: Commands,
    q_ingame_ui: Query<Entity, With<InGameUI>>
) {
    for entity in q_ingame_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
