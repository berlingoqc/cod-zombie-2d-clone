
use bevy::{
    input::gamepad::{GamepadEvent, GamepadEventType},
    prelude::*
};

use crate::{game::{GameState, GameSpeed}, character::{CharacterMovementState, LookingAt, Death}, collider::{MovementCollider, is_colliding}, utils::get_cursor_location};

use super::{Player, MainCamera, PLAYER_SIZE};


#[derive(Default, PartialEq, Debug, Clone)]
pub enum SupportedController {
	#[default]
	Keyboard,
	Gamepad
}

#[derive(Component, Default, Debug, Clone)]
pub struct PlayerCurrentInput {
	pub input_source: SupportedController,

	pub gamepad: Option<Gamepad>
}


pub struct AvailableGameController {
    pub keyboard_mouse: bool,
    pub gamepad: Vec<Gamepad>,
}


fn vec_moving(vec: &Vec2) -> bool {
    return vec.x != 0. && vec.y != 0.;
}

fn get_gamepad_input(
    player_gamepad: Gamepad,
    axes: &Res<Axis<GamepadAxis>>,
) -> (Option<Vec3>, Option<Vec2>) {

    let axis_lx = GamepadAxis(player_gamepad, GamepadAxisType::LeftStickX);
    let axis_ly = GamepadAxis(player_gamepad, GamepadAxisType::LeftStickY);

    let axis_rx = GamepadAxis(player_gamepad, GamepadAxisType::RightStickX);
    let axis_ry = GamepadAxis(player_gamepad, GamepadAxisType::RightStickY);


    if let (Some(x), Some(y), Some(rx), Some(ry)) = (axes.get(axis_lx), axes.get(axis_ly), axes.get(axis_rx), axes.get(axis_ry)) {
        let left_stick_pos = Vec2::new(x, y);
        let right_stick_pos = Vec2::new(rx, ry);

        return (if vec_moving(&left_stick_pos) { Some(left_stick_pos.extend(0.0)) } else { None }, if vec_moving(&right_stick_pos) {  Some(right_stick_pos) } else { None });
    }
    
    return (None, None);
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

    mut available_controller: ResMut<AvailableGameController>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        match kind {
            GamepadEventType::Connected => {
                available_controller.gamepad.push(id.clone());
            },
            GamepadEventType::Disconnected => {
                available_controller.gamepad = available_controller.gamepad.iter().filter(|x| x.0 != id.0).map(|x| x.clone()).collect();
            },
            _ => {
                //info!("OTHER EVENT I GUESS {:?}", id);
            }
        }
    }

}

pub fn input_player(
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,

    mut query: Query<(&mut Transform, &mut Player, &mut CharacterMovementState, &mut LookingAt, &PlayerCurrentInput), Without<Death>>,
    collider_query: Query<
        (Entity, &Transform, &MovementCollider),
        (Without<Player>, Without<Death>),
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
				(get_keyboard_input(&keyboard_input), Some(get_cursor_location(&wnds, &q_camera)))
			},
			SupportedController::Gamepad => {
                let gamepad = current_input.gamepad.unwrap();
                looking_at.1 = true;
				get_gamepad_input(gamepad, &axes)
			}
		};
        

        if let Some(looking_direction) = looking_direction {
		    looking_at.0 = looking_direction;
        }
		
		if let Some(movement) = opt_movement {
			character_movement_state.state = "walking".to_string();

			let dest = player_transform.translation + (movement * game_speed.0 * 125.);

			if !is_colliding(dest, PLAYER_SIZE, "player",&collider_query) {
				player_transform.translation = dest;
			}
		} else if character_movement_state.state == "walking" {
            character_movement_state.state = "standing".to_string();
		}
    }
}