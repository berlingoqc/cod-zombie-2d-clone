use bevy::{prelude::*, sprite::{SpriteBundle, Sprite}, math::Vec2};
use serde::Deserialize;
use std::time::Duration;

use crate::{utils::get_cursor_location, collider::ProjectileCollider, player::{Velocity, ExpiringComponent, MainCamera}};

#[derive(Default, Component, Clone, Deserialize)]
pub struct Weapon {
	pub name: String,
	pub asset_name: String,
	pub ammunition: Ammunition,
	pub firing_rate: f32,
	pub reloading_time: f32,
	pub automatic: bool
}

#[derive(Default, Clone, Deserialize)]
pub struct Ammunition {
	pub magasin_size: i32,
	pub magasin_limit: i32,
	pub magasin_nbr_starting: i32
}

#[derive(Default, Component)]
pub struct AmmunitionState {
	pub mag_remaining: i32,
	pub remaining_ammunition: i32
}


#[derive(Default, PartialEq)]
pub enum WeaponCurrentAction {
	#[default]
	Firing = 0,
	Reloading
}

#[derive(Default, Component)]
pub struct WeaponState {
	pub fired_at: f32,
    pub equiped: bool,
	pub state: WeaponCurrentAction
}

#[derive(Default, Component)]
pub struct ActivePlayerWeapon {}


#[derive(Default, Component)]
pub struct Projectile {}



#[derive(Bundle)]
pub struct WeaponBundle {
	pub weapon: Weapon,
	pub ammunition_state: AmmunitionState,
	pub weapon_state: WeaponState
}


impl WeaponBundle {
	pub fn new(weapon: Weapon, equiped: bool) -> Self {
		WeaponBundle { 
			ammunition_state: AmmunitionState {
				mag_remaining: weapon.ammunition.magasin_size,
				remaining_ammunition: (weapon.ammunition.magasin_nbr_starting - 1) * weapon.ammunition.magasin_size
			},
			weapon,
			weapon_state: WeaponState{
				fired_at: 0.,
                equiped,
				state: WeaponCurrentAction::Firing
			}
		}
	}
}

pub fn handle_weapon_input(
    mut commands: Commands,
    time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
	mut query_player_weapon: Query<(&mut AmmunitionState, &mut WeaponState, &Weapon, &Parent)>,
	
	wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

	q_parent: Query<&GlobalTransform>,
) {
	if (query_player_weapon.is_empty()) {
		return;
	}
    if keyboard_input.just_pressed(KeyCode::Tab) {
        let mut i = 0;
        for _ in query_player_weapon.iter() {
            i += 1;
        }
        if i == 2 {
            for (_, mut weapon_state, w, _) in query_player_weapon.iter_mut() {
                weapon_state.equiped = !weapon_state.equiped;
            }
        }
        return;
	}


    for (mut ammunition_state, mut weapon_state, weapon, parent) in query_player_weapon.iter_mut() {
        if !weapon_state.equiped {
            return;
        }
        if weapon_state.state == WeaponCurrentAction::Reloading {
            let current_time = time.time_since_startup().as_secs_f32();
            if current_time < weapon_state.fired_at + weapon.reloading_time {
                return
            } 
            let diff = weapon.ammunition.magasin_size - ammunition_state.mag_remaining;
            if diff > ammunition_state.remaining_ammunition {
                ammunition_state.mag_remaining = ammunition_state.remaining_ammunition;
                ammunition_state.remaining_ammunition = 0;
            } else {
                ammunition_state.mag_remaining =  weapon.ammunition.magasin_size;
                ammunition_state.remaining_ammunition -= diff;
            }

            weapon_state.state = WeaponCurrentAction::Firing;
        }

        if keyboard_input.just_pressed(KeyCode::R) {


            if ammunition_state.mag_remaining < weapon.ammunition.magasin_size {
                weapon_state.state = WeaponCurrentAction::Reloading;
                weapon_state.fired_at = time.time_since_startup().as_secs_f32();
                return;
            }
        }

		let firing = if weapon.automatic {
			buttons.pressed(MouseButton::Left) || keyboard_input.pressed(KeyCode::Space)
		} else {
			buttons.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Space)
		};

        if firing {

            if ammunition_state.mag_remaining == 0 {
                weapon_state.state = WeaponCurrentAction::Reloading;
                weapon_state.fired_at = time.time_since_startup().as_secs_f32();
                return;
            }

            let current_time = time.time_since_startup().as_secs_f32();

            if current_time < weapon_state.fired_at + weapon.firing_rate {
                return;
            }

            weapon_state.fired_at = current_time;
            ammunition_state.mag_remaining -= 1;

            let mouse_location = get_cursor_location(&wnds, &q_camera);
            let parent_location = q_parent.get(parent.0).unwrap().translation;

            let diff = Vec2::new(mouse_location.x - parent_location.x, mouse_location.y - parent_location.y).normalize();

            commands
                .spawn()
                .insert(Projectile {})
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: parent_location,
                        ..Transform::default()
                    },
                    sprite: Sprite {
                        color: Color::BISQUE,
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..Sprite::default()
                    },
                    ..SpriteBundle::default()
                })
                .insert(ExpiringComponent {
                    created_at: time.time_since_startup().as_secs_f32(),
                    duration: 2.,
                })
                .insert(ProjectileCollider {})
                .insert(Velocity {
                    v: diff * 1000.,
                });
        }
    }
}
