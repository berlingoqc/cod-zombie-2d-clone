use std::time::Duration;

use bevy::prelude::*;


#[derive(Component, Default)]
pub struct HealthRegeneration {
	pub timeout_regeneration: f32,
	pub regeneration_amount: f32,
	pub timer: Option<Timer>,
}


impl HealthRegeneration {

	// call when the health is updated to calculate when i can regenerate
	// start a new timer
	pub fn on_health_change(&mut self) -> () {
		self.timer = Some(Timer::from_seconds(self.timeout_regeneration, false));
	}

	// trigger each time to modify health if possible
	// tick the timer
	pub fn apply_regeneration_if(&mut self, delta: Duration, health: &mut Health) -> () {
		if health.current_health > 0. {
			if let Some(timer) = self.timer.as_mut() {
				timer.tick(delta);
				if timer.finished() {
					health.tmp_health += self.regeneration_amount;
					if health.tmp_health < health.max_health {
						timer.reset();
					} else {
						self.timer = None;
					}
				}
			};
		}
	}
}

#[derive(Component, Default)]
pub struct Health {
	pub current_health: f32,
	pub tmp_health: f32,
	pub max_health: f32,
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

	pub fn apply_change(&mut self) -> () {
		self.current_health = self.tmp_health;
	}
}
