use bevy::prelude::*;

#[derive(Component)]
pub struct DebugComponent {}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum DebugState {
	None,
	On,
}


// debug must be only on debug collider mode and must remove the sprite on leave 
pub fn system_collider_debug_cleanup(
    mut commands: Commands,
    q_debug_component: Query<Entity, With<DebugComponent>>
) {
    for entity in q_debug_component.iter() {
		commands.entity(entity).despawn_recursive();
    }

}


pub fn system_debug_keyboard(
	mut state: ResMut<State<DebugState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {


	if keyboard_input.just_pressed(KeyCode::F10) {
		if let Ok(()) = state.set(DebugState::None) {
			info!("Stop debugging mode");
		} else {
			if let Ok(()) = state.set(DebugState::On) {
				info!("Start debugging mode");
			}
		}
	}
}