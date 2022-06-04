
use bevy::{
    input::gamepad::{GamepadEvent, GamepadEventType},
    prelude::*
};

use crate::{game::{GameState, GameSpeed}, character::{CharacterMovementState, LookingAt}, collider::{MovementCollider, is_colliding}, utils::get_cursor_location};

use super::{Player, MainCamera, PLAYER_SIZE};


#[derive(Default)]
pub enum SupportedController {
	#[default]
	Keyboard,
	Gamepad
}

#[derive(Component, Default)]
pub struct PlayerCurrentInput {
	pub input_source: SupportedController,

	pub gamepad: Option<Gamepad>
}


fn get_gamepad_input(
    player_gamepad: Gamepad,
    axes: &Res<Axis<GamepadAxis>>,
) -> Option<Vec3> {

    let axis_lx = GamepadAxis(player_gamepad, GamepadAxisType::LeftStickX);
    let axis_ly = GamepadAxis(player_gamepad, GamepadAxisType::LeftStickY);

    if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
        let left_stick_pos = Vec2::new(x, y);

        if left_stick_pos.length() > 0.9 && left_stick_pos.y > 0.5 {
            return Some(left_stick_pos.extend(0.0))
        }
    }
    
    return None;
}

fn get_gamepad_looking_at(
    player_gamepad: Gamepad,
    axes: &Res<Axis<GamepadAxis>>,
) -> Vec2 {
    let axis_rx = GamepadAxis(player_gamepad, GamepadAxisType::RightStickX);
    let axis_ry = GamepadAxis(player_gamepad, GamepadAxisType::RightStickY);

    if let (Some(x), Some(y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
        let right_stick_pos = Vec2::new(x, y);

        if right_stick_pos.length() > 0.9 && right_stick_pos.y > 0.5 {
            return right_stick_pos;
        }
    }

    return Vec2::default();
}

fn get_keyboard_input(
    keyboard_input: &Res<Input<KeyCode>>,
) -> Option<Vec3> {

    let mut movement = Vec3::default();
    let mut moved = false;


    if keyboard_input.pressed(KeyCode::W) {
        movement += Vec3::new(0., 1., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement += Vec3::new(0., -1., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement += Vec3::new(-1., 0., 0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement += Vec3::new(1., 0., 0.);
        moved = true;
    }

	return if moved { Some(movement) }  else { None };
}


pub fn system_gamepad_event(
    mut q_player_input: Query<&mut PlayerCurrentInput, With<Player>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        match kind {
            GamepadEventType::Connected => {
                info!("New gamepad connected with ID: {:?}", id);
                if let Ok(mut current_input) = q_player_input.get_single_mut() {
                    current_input.input_source = SupportedController::Gamepad;
                    current_input.gamepad = Some(id.clone());
                    println!("ASsigned to palyer")
                }
            },
            GamepadEventType::Disconnected => {

                info!("New gamepad discconnected with ID: {:?}", id);
            },
            _ => {
                info!("OTHER EVENT I GUESS {:?}", id);
            }
        }
    }

}

pub fn input_player(
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,

    mut query: Query<(&mut Transform, &mut Player, &mut CharacterMovementState, &mut LookingAt, &PlayerCurrentInput)>,
    collider_query: Query<
        (Entity, &Transform, &MovementCollider),
        Without<Player>,
    >,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

    game_speed: Res<GameSpeed>,

    mut game_state: ResMut<State<GameState>>
) {


    if keyboard_input.pressed(KeyCode::F2) {
        game_state.set(GameState::Menu).unwrap();
        return;
    }
		

    // TODO : split player input and movement (IF I WANT TO NETWORK AT SOME POINT)
    for (
		mut player_transform,
		mut player,
		mut character_movement_state,
		mut looking_at,
		current_input
	) in query.iter_mut() {

        let (opt_movement, looking_direction) = match current_input.input_source {
			SupportedController::Keyboard => {
				(get_keyboard_input(&keyboard_input), get_cursor_location(&wnds, &q_camera))
			},
			SupportedController::Gamepad => {
                let gamepad = current_input.gamepad.unwrap();
				(get_gamepad_input(gamepad, &axes), get_gamepad_looking_at(gamepad, &axes))
			}
		};

		looking_at.0 = looking_direction;
		
		
		if let Some(movement) = opt_movement {
			character_movement_state.state = "walking".to_string();

			let dest = player_transform.translation + (movement * game_speed.0 * 125.);

			if !is_colliding(dest, PLAYER_SIZE, "player",&collider_query) {
				player_transform.translation = dest;
			}
		} else {
            character_movement_state.state = "standing".to_string();
		}
    }
}