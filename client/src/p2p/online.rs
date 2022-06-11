

use std::net::SocketAddr;

use bevy::prelude::*;
use shared::{game::GameSpeed, player::input::GGRSConfig};

use bevy_ggrs::SessionType;
use ggrs::{PlayerType, SessionBuilder, NonBlockingSocket};

use super::config::*;


pub struct NetworkPlayer {
	pub address: String,
}

pub fn create_network_session(
	commands: &mut Commands,
	game_speed: &GameSpeed,
	socket: impl NonBlockingSocket<SocketAddr> + 'static,
	players: Vec<NetworkPlayer>,
) {
	let nbr_player = players.iter().count();
	let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(nbr_player)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(game_speed.1)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);

    for (i, player_addr) in players.iter().enumerate() {
		if player_addr.address == "localhost" {
	        sess_build = sess_build
	            .add_player(PlayerType::Local, i)
	            .expect("Could not add local player");
		} else {
			let remote_addr: SocketAddr = player_addr.address.parse().unwrap();
			sess_build = sess_build.add_player(PlayerType::Remote(remote_addr), i).unwrap();
			println!("ADD REMOTE ADDR {:?}", remote_addr);
		}
    }

    let sess = sess_build.start_p2p_session(socket).unwrap();

    commands.insert_resource(sess);
    commands.insert_resource(SessionType::P2PSession);
    commands.insert_resource(LocalHandles {
        handles: (0..nbr_player).collect(),
    });

}