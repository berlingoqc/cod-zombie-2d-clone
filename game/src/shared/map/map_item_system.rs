
use bevy::prelude::*;

use crate::shared::{collider::MovementCollider, health::{Health, HealthChangeState}};

use super::{WindowPanel, Window};


pub fn system_window_panel_destroy(
	// Change on Window component
	mut q_window_change: Query<(&Window, &mut MovementCollider, &mut Health, &Children), Changed<Health>>,
	
    mut q_panel: Query<(&WindowPanel, &mut Sprite)>,
) {

	for (w, mut movement_collider, mut health, children) in q_window_change.iter_mut() {
		match health.get_health_change_state() {
			HealthChangeState::Dead => {
				movement_collider.allowed_entity_type = vec!["zombie".to_string()];
				health.current_health = 0.;
				health.tmp_health = 0.;

				for first_child in children.iter() {
					if let Ok((_, mut sprite)) = q_panel.get_mut(first_child.clone()) {
						if sprite.custom_size.unwrap().x != 0. {
							sprite.custom_size = Some(Vec2::new(0., 0.));
						}
					}
				}
			},
			HealthChangeState::GainHealth => {
				movement_collider.allowed_entity_type = vec![];
				health.current_health = health.tmp_health;
			},
			HealthChangeState::LostHealth => {
				// Remove the first child
				for first_child in children.iter() {
					if let Ok((_, mut sprite)) = q_panel.get_mut(first_child.clone()) {
						if sprite.custom_size.unwrap().x != 0. {
							sprite.custom_size = Some(Vec2::new(0., 0.));
							break;
						}
					}
				}

				health.current_health = health.tmp_health;
			},
			_ => {}

		}
	}



}