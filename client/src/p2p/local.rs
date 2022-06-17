use bevy::prelude::*;
use shared::{game::GameSpeed, player::input::GGRSConfig};

use bevy_ggrs::SessionType;
use ggrs::{PlayerType, SessionBuilder};

use super::config::*;

pub fn create_local_session(
	commands: &mut Commands,
	game_speed: &GameSpeed,
	nbr_player: usize
) {
	let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(nbr_player)
        .with_max_prediction_window(8)
        .with_fps(game_speed.1)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);

    for i in 0..nbr_player {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Could not add local player");
    }

    let sess = sess_build.start_synctest_session().expect("");

    commands.insert_resource(sess);
    commands.insert_resource(SessionType::SyncTestSession);
    commands.insert_resource(LocalHandles {
        handles: (0..nbr_player).collect(),
    });

}