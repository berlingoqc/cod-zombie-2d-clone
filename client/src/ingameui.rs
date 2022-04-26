

use bevy::prelude::*;
use shared::{game::{Zombie, ZombieGame}, weapons::weapons::{AmmunitionState, Weapon, ActivePlayerWeapon, WeaponState}, player::Player};

#[derive(Component)]
pub struct RoundText;

#[derive(Component)]
pub struct WeaponText;

#[derive(Component)]
pub struct WeaponUiImage;

pub fn setup_ingame_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Text bundle for the round info 
	  commands.spawn_bundle(TextBundle {
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

    // Text bundle for the ammunition

    commands.spawn_bundle(NodeBundle {
         style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            color: UiColor(Color::NONE),
        ..default()
    }).with_children(|parent| {
        // bundle for image
        parent.spawn_bundle(NodeBundle {
            ..default()
        }).with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(25.), Val::Auto),
                    ..default()
                },
                image: UiImage(asset_server.load("weapons/pistol.png")),
                ..default()
            }).insert(WeaponUiImage{});
        });

        // Bundle for info 
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

pub fn system_ingame_ui(
    zombie_game: Res<ZombieGame>,
    zombie_query: Query<&Zombie>,
    mut query_round: Query<&mut Text, With<RoundText>>,
    asset_server: Res<AssetServer>,
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
}

pub fn system_weapon_ui(
	zombie_game: Res<ZombieGame>,
	query_player_weapon: Query<(&AmmunitionState, &Weapon, &WeaponState)>,
    mut query_ammo_text: Query<&mut Text, With<WeaponText>>,

    mut query_weapon_image: Query<&mut UiImage, With<WeaponUiImage>>,
    asset_server: Res<AssetServer>,
) {
    if (query_player_weapon.is_empty()) {
        return;
    }
	let mut text = query_ammo_text.single_mut();
    for (ammo_state, weapon, weapon_state) in query_player_weapon.iter() {
        if weapon_state.equiped {
            text.sections[0].value = format!("{}\n-\n{}", ammo_state.mag_remaining, ammo_state.remaining_ammunition);

            let mut weapon_image = query_weapon_image.single_mut();
            weapon_image.0 = asset_server.load(format!("/weapons/{}", weapon.asset_name).as_str());
        }
    }
}