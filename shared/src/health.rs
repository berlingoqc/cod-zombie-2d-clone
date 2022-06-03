use bevy::prelude::*;


#[derive(Component, Default)]
pub struct Health {
	pub current_health: f32,
	pub tmp_health: f32,
	pub max_health: f32
}

pub enum HealthChangeState {
	GainHealth,
	LostHealth,
	Dead,
	Nothing
}

impl Health {
	pub fn alive(&self) -> bool {
		self.current_health <= 0.
	}

	pub fn get_health_change_state(&self) -> HealthChangeState {
		if self.tmp_health == self.current_health {
			return HealthChangeState::Nothing;
		}
		if self.tmp_health <= 0. {
			return HealthChangeState::Dead;
		}
		if self.tmp_health > self.current_health {
			return HealthChangeState::GainHealth;
		}
		return HealthChangeState::LostHealth;
	}
}
