
use bevy::prelude::*;

use crate::collider::MovementCollider;

use super::{WindowPanel, Window};



pub fn system_window_panel_destroy(
	// Change on Window component
	mut q_window_change: Query<(&Window, &mut MovementCollider), Changed<Window>>,
) {

	for (w, mut movement_collider) in q_window_change.iter_mut() {
		if w.destroy {
			movement_collider.allowed_entity_type = vec!["zombie".to_string()];
		} else {
			movement_collider.allowed_entity_type = vec![];
		}
	}



}