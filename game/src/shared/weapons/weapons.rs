use bevy::{prelude::*, sprite::{SpriteBundle, Sprite}, math::Vec2};
use bevy_ggrs::{RollbackIdProvider, Rollback};
use ggrs::InputStatus;
use serde::Deserialize;

use rand::prelude::SliceRandom;

use crate::shared::{
    utils::{get_cursor_location, vec2_perpendicular_counter_clockwise, vec2_perpendicular_clockwise},
    collider::ProjectileCollider,
    animation::AnimationTimer, player::{MainCamera, Player, input::{PlayerCurrentInput, SupportedController, INPUT_FIRE, INPUT_JUST_FIRE, BoxInput, INPUT_WEAPON_CHANGED, INPUT_WEAPON_RELOAD}}, character::{CharacterMovementState, Velocity, LookingAt, Death}
};


fn default_firing_ammunition() -> u32 {
    1
}

fn default_ammo_sprite_config() -> AmmunitionSpriteConfig {
    AmmunitionSpriteConfig { 
        size: Vec2::new(5., 5.)
    }
}

#[derive(Default, Component)]
pub struct ExpiringComponent {
    pub created_at: f32,
    pub duration: f32,
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

#[derive(Default, Component, Reflect)]
pub struct AmmunitionState {
	pub mag_remaining: i32,
	pub remaining_ammunition: i32
}


#[derive(Default, PartialEq)]
pub enum WeaponCurrentAction {
	#[default]
	Firing = 0,
	Reloading,
}

#[derive(Default, Component)]
pub struct WeaponState {
	pub fired_at: f32,
	pub state: WeaponCurrentAction
}

#[derive(Default, Component)]
pub struct ActiveWeapon {}

#[derive(Default, Component, Reflect)]
pub struct Projectile {}


#[derive(Bundle)]
pub struct WeaponBundle {
	pub weapon: Weapon,
	pub ammunition_state: AmmunitionState,
	pub weapon_state: WeaponState
}


impl WeaponBundle {
	pub fn new(weapon: Weapon) -> Self {
		WeaponBundle { 
			ammunition_state: AmmunitionState {
				mag_remaining: weapon.ammunition.magasin_size,
				remaining_ammunition: (weapon.ammunition.magasin_nbr_starting - 1) * weapon.ammunition.magasin_size
			},
			weapon,
			weapon_state: WeaponState{
				fired_at: 0.,
				state: WeaponCurrentAction::Firing
			}
		}
	}
}

pub type GameButton = (KeyCode, MouseButton, GamepadButtonType);

pub const CHANGE_WEAPON_BTN: GameButton = (KeyCode::Tab, MouseButton::Middle, GamepadButtonType::North);
pub const RELOAD_WEAPON_BTN: GameButton = (KeyCode::R, MouseButton::Right, GamepadButtonType::West);
pub const FIRED_WEAPON_BTN: GameButton = (KeyCode::Space, MouseButton::Left, GamepadButtonType::RightTrigger);
pub const INTERACTION_BTN: GameButton = (KeyCode::F, MouseButton::Other(0), GamepadButtonType::South);

pub struct PlayerInputs<'a> {
    pub keyboard_input: &'a Res<'a,Input<KeyCode>>,
    pub buttons_mouse: &'a Res<'a, Input<MouseButton>>,
    pub buttons_gamepad: &'a Res<'a, Input<GamepadButton>>,

    pub current_controller: &'a PlayerCurrentInput,
}

impl <'a> PlayerInputs <'a> {

    pub fn pressed(&self, button: &GameButton) -> bool {
        return if self.current_controller.input_source == SupportedController::Keyboard {
            return self.keyboard_input.pressed(button.0) || self.buttons_mouse.pressed(button.1);
        } else {
            let gamepad_button = GamepadButton(self.current_controller.gamepad.unwrap(), button.2);
            return self.buttons_gamepad.pressed(gamepad_button);
        }
    }

    pub fn just_pressed(&self, button: &GameButton) -> bool {
        return if self.current_controller.input_source == SupportedController::Keyboard {
            return self.keyboard_input.just_pressed(button.0) || self.buttons_mouse.just_pressed(button.1);
        } else {
            let gamepad_button = GamepadButton(self.current_controller.gamepad.unwrap(), button.2);
            return self.buttons_gamepad.just_pressed(gamepad_button);
        }
    }
}

// get the input press for the player

pub fn handle_weapon_input(
    mut commands: Commands,
    time: Res<Time>,
    
    query_unequiped_weapon: Query<(Entity, &Weapon), Without<ActiveWeapon>>,
	mut query_player_weapon: Query<(Entity, &mut AmmunitionState, &mut WeaponState, &Weapon, &ActiveWeapon), With<WeaponState>>,
	
    mut q_player: Query<(&GlobalTransform, &PlayerCurrentInput, &LookingAt, &mut CharacterMovementState, &mut AnimationTimer, &Children, &Player), (Without<Death>)>,

    inputs: Res<Vec<(BoxInput, InputStatus)>>,

    mut rip: ResMut<RollbackIdProvider>,
) {
    for (player_global_transform, current_input, looking_at, mut movement_state, mut timer, childrens, player) in q_player.iter_mut() {

        if inputs.len() <= player.handle {
            continue;
        }

        let box_input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0,
            InputStatus::Predicted => inputs[player.handle].0,
            InputStatus::Disconnected => BoxInput::default(), // disconnected players do nothing
        };
        
        if box_input.inp & INPUT_WEAPON_CHANGED == INPUT_WEAPON_CHANGED {
            for children in childrens.iter() {
                if let Ok((weapon_entity,_,_, _, _)) = query_player_weapon.get(*children) {
                    commands.entity(weapon_entity).remove::<ActiveWeapon>();
                } else if let Ok((weapon_entity, weapon)) = query_unequiped_weapon.get(*children) {
                    commands.entity(weapon_entity).insert(ActiveWeapon{});

                    movement_state.sub_state = weapon.name.clone();
                    timer.offset = weapon.sprite_sheet_offset;
                }
            }
        }

        for children in childrens.iter() {
            if let Ok((_,mut ammunition_state, mut weapon_state, weapon, _)) = query_player_weapon.get_mut(*children) {
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

                if box_input.inp & INPUT_WEAPON_RELOAD == INPUT_WEAPON_RELOAD {


                    if ammunition_state.mag_remaining < weapon.ammunition.magasin_size {
                        weapon_state.state = WeaponCurrentAction::Reloading;
                        weapon_state.fired_at = time.time_since_startup().as_secs_f32();
                        continue;
                    }
                }

                let firing = if weapon.automatic {
                    box_input.inp & INPUT_FIRE == INPUT_FIRE
                } else {
                    box_input.inp & INPUT_JUST_FIRE == INPUT_JUST_FIRE
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

                    let parent_location = player_global_transform.translation;

                    let mut diff = (if !looking_at.1 { 
                        let mouse_location = looking_at.0;
                        Vec2::new(mouse_location.x - parent_location.x, mouse_location.y - parent_location.y).normalize()
                    } else { looking_at.0 }).normalize();

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

                                spawn_bullet(&mut commands, &mut rip, &weapon, &time, &starting_point, &offset_each, &Vec2::new(new_x, new_y), i);
                            }
                        } else {
                            spawn_bullet(&mut commands, &mut rip, &weapon, &time, &starting_point, &offset_each, &diff, i);
                        }

                    }

                }
            }
        }
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    mut rip: &mut ResMut<RollbackIdProvider>,
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
        })
        .insert(Rollback::new(rip.next_id()));
}

