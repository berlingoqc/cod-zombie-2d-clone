use bevy::{prelude::*, sprite::{SpriteBundle, Sprite}, math::Vec2};
use serde::Deserialize;
use std::time::Duration;

use rand::prelude::SliceRandom;

use crate::{utils::{get_cursor_location, vec2_perpendicular_counter_clockwise, vec2_perpendicular_clockwise}, collider::ProjectileCollider, player::{Velocity, ExpiringComponent, MainCamera, LookingAt, Player, CharacterMovementState, AnimationTimer}};

use super::loader::WeaponsAsset;


fn default_firing_ammunition() -> u32 {
    1
}

fn default_ammo_sprite_config() -> AmmunitionSpriteConfig {
    AmmunitionSpriteConfig { 
        size: Vec2::new(5., 5.)
    }
}


#[derive(Default, Clone, Deserialize)]
pub struct AmmunitionSpriteConfig {
    //pub color: Vec3,
    pub size: Vec2
}

#[derive(Default, Component, Clone, Deserialize)]
pub struct Weapon {
	pub name: String,
	pub asset_name: String,
	pub ammunition: Ammunition,
	pub firing_rate: f32,
	pub reloading_time: f32,
    #[serde(default = "default_firing_ammunition")]
    pub firing_ammunition: u32,
    #[serde(default = "default_firing_ammunition")]
    pub spreading_ammunition: u32,
    pub offset: i32,
	pub automatic: bool,

    pub sprite_sheet_offset: usize
}

#[derive(Default, Clone, Deserialize)]
pub struct Ammunition {
	pub magasin_size: i32,
	pub magasin_limit: i32,
	pub magasin_nbr_starting: i32,

    pub duration: f32,

    #[serde(default = "default_ammo_sprite_config")]
    pub sprite_config: AmmunitionSpriteConfig
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
	Reloading,
    Hide,
}

#[derive(Default, Component)]
pub struct WeaponState {
	pub fired_at: f32,
	pub state: WeaponCurrentAction
}

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
				state: if equiped { WeaponCurrentAction::Firing } else { WeaponCurrentAction::Hide }
			}
		}
	}
}

pub fn handle_weapon_input(
    mut commands: Commands,
    time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
	mut query_player_weapon: Query<(&mut AmmunitionState, &mut WeaponState, &Weapon, &Parent), With<WeaponState>>,
	
	wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

	q_parent: Query<&GlobalTransform>,

    mut q_movement_player: Query<(&mut CharacterMovementState, &mut AnimationTimer), With<Player>>,

    mut q_player: Query<&mut Player>,
) {

    if keyboard_input.just_pressed(KeyCode::Tab) {
        let mut i = 0;
        // If one of the gun is being reload do do shit when ask to change weapon
        for (_, weapon_state, _, _) in query_player_weapon.iter() {
            if weapon_state.state == WeaponCurrentAction::Reloading {
                return;
            }
        }
        for (_, mut weapon_state, w, parent) in query_player_weapon.iter_mut() {
            if weapon_state.state == WeaponCurrentAction::Firing {
                weapon_state.state = WeaponCurrentAction::Hide;
            } else if weapon_state.state == WeaponCurrentAction::Hide {
                weapon_state.state = WeaponCurrentAction::Firing;

                // CODE FOR THE ANIMATION
                let (mut movement_state, mut timer) = q_movement_player.get_mut(parent.0).unwrap();
                movement_state.sub_state = w.name.clone();
                timer.offset = w.sprite_sheet_offset;
            }
        }

        return;
	}


    for (mut ammunition_state, mut weapon_state, weapon, parent) in query_player_weapon.iter_mut() {

        if weapon_state.state == WeaponCurrentAction::Hide {
            continue;
        }

        if weapon_state.state == WeaponCurrentAction::Reloading {
            let current_time = time.time_since_startup().as_secs_f32();
            if current_time < weapon_state.fired_at + weapon.reloading_time {
                continue;
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
                continue;
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
                continue;
            }

            let current_time = time.time_since_startup().as_secs_f32();

            if current_time < weapon_state.fired_at + weapon.firing_rate {
                continue;
            }

            weapon_state.fired_at = current_time;

            let mouse_location = get_cursor_location(&wnds, &q_camera);
            let parent_location = q_parent.get(parent.0).unwrap().translation;


            let mut diff = Vec2::new(mouse_location.x - parent_location.x, mouse_location.y - parent_location.y).normalize();

            if weapon.offset > 0 {
                let bottom = weapon.offset * -1;
                let top = weapon.offset * 1;

                let mut ndg = rand::thread_rng();
                let mut range: Vec<f32> = (bottom..top).map(|x| x as f32).collect();
                range.shuffle(&mut ndg);
                
                diff.x += range[0] / 100.;
                diff.y += range[1] / 100.;
            }

            let (starting_point, offset_each) = if weapon.firing_ammunition == 1 {
                (parent_location, Vec2::new(0.,0.))
            } else {
                let counter_clock_perpenicular = vec2_perpendicular_counter_clockwise(diff);
                let offset_scale = weapon.firing_ammunition / 2;

                // not perfectly center
                (
                    (counter_clock_perpenicular * (offset_scale as f32) * weapon.ammunition.sprite_config.size.x).extend(10.) + parent_location,
                    vec2_perpendicular_clockwise(diff) * weapon.ammunition.sprite_config.size.x
                )
            };

            for i in (0..weapon.firing_ammunition) {
                ammunition_state.mag_remaining -= 1;
                

                if weapon.spreading_ammunition > 1 {
                    for x in 0..weapon.spreading_ammunition/2 {
                        let scale: f32 = if x % 2 == 0 { 1. } else { -1. };
                        let angle: f32 = ((x as f32) / 20.) * scale;

                        let new_x = diff.x * angle.cos() - diff.y * angle.sin();
                        let new_y = diff.x * angle.sin() + diff.y * angle.cos();

                        spawn_bullet(&mut commands, &weapon, &time, &starting_point, &offset_each, &Vec2::new(new_x, new_y), i);
                    }
                } else {
                    spawn_bullet(&mut commands, &weapon, &time, &starting_point, &offset_each, &diff, i);
                }

            }

        }
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    weapon: &Weapon,
    time: &Res<Time>,
    starting_point: &Vec3,
    offset_each: &Vec2,
    velocity: &Vec2,
    index: u32,
) {
    commands
        .spawn()
        .insert(Projectile {})
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: *starting_point + (offset_each.extend(0.) * index as f32),
                ..Transform::default()
            },
            sprite: Sprite {
                color: Color::BISQUE,
                custom_size: Some(weapon.ammunition.sprite_config.size),
                ..Sprite::default()
            },
        ..SpriteBundle::default()
        })
        .insert(ExpiringComponent {
            created_at: time.time_since_startup().as_secs_f32(),
            duration: weapon.ammunition.duration,
        })
        .insert(ProjectileCollider {})
        .insert(Velocity {
            v: *velocity * 1000.,
        });
}

