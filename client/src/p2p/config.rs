use bevy::prelude::*;
use ggrs::PlayerHandle;

pub const ROLLBACK_SYSTEMS: &str = "rollback_systems";
pub const CHECKSUM_UPDATE: &str = "checksum_update";
pub const MAX_PREDICTION: usize = 12;
pub const INPUT_DELAY: usize = 2;
pub const CHECK_DISTANCE: usize = 2;


pub struct LocalHandles {
    pub handles: Vec<PlayerHandle>,
}


#[derive(SystemLabel, Debug, Clone, Hash, Eq, PartialEq)]
pub enum P2PSystemLabel {
    Input,
    Velocity,
}