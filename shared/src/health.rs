use bevy::prelude::*;


#[derive(Component, Default)]
pub struct Health {
	pub current_health: f32,
	pub max_health: f32
}

impl Health {
	pub fn alive(&self) -> bool {
		self.current_health <= 0.
	}
}