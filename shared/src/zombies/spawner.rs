use std::time::Duration;

use bevy::prelude::*;

pub struct ZombieSpawnerConfig {
    pub timer: Timer,
    pub nums_ndg: Vec<f32>,
}

impl FromWorld for ZombieSpawnerConfig {
    fn from_world(world: &mut World) -> Self {
        ZombieSpawnerConfig{
            timer: Timer::new(Duration::from_secs(5), true),
            nums_ndg: (-50..50).map(|x| x as f32).collect()
        }
    }
}


