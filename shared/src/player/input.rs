
use bevy::prelude::*;

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

pub fn input_player(
    keyboard_input: Res<Input<KeyCode>>,
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
			_ => {
				(get_keyboard_input(&keyboard_input), get_cursor_location(&wnds, &q_camera))
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