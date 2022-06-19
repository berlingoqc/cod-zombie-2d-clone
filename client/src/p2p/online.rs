

use bevy::prelude::*;
use shared::{game::GameSpeed, player::input::GGRSConfig};

use bevy_ggrs::SessionType;
use ggrs::{PlayerType, SessionBuilder, NonBlockingSocket, P2PSession, Message};

use super::config::{MAX_PREDICTION, INPUT_DELAY, CHECK_DISTANCE, LocalHandles};


pub struct LocalSocket {
	messages: Vec<(String, Message)>,
}

impl Default for LocalSocket {
	fn default() -> Self {
		Self {
			messages: vec![]
		}
	}

}

impl ggrs::NonBlockingSocket<String> for LocalSocket {

	fn send_to(&mut self, msg: &Message, addr: &String) {
		self.messages.push((addr.clone(), msg.clone()));
    }

    fn receive_all_messages(&mut self) -> Vec<(String, Message)> {
		let msg = self.messages.clone();
		self.messages = vec![];
        msg
    }
}


pub struct NetworkPlayer {
	pub address: String,
}

pub fn create_session(
	commands: &mut Commands,
	game_speed: &GameSpeed,
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
			sess_build = sess_build.add_player(PlayerType::Remote(player_addr.address.clone()), i).unwrap();
		}
    }

	let sess = sess_build.start_p2p_session(LocalSocket::default()).unwrap();

    commands.insert_resource(sess);
    commands.insert_resource(SessionType::P2PSession);
    commands.insert_resource(LocalHandles {
        handles: (0..nbr_player).collect(),
    });

}

pub fn system_cleanup_network_session(
	mut commands: Commands,
) {
	commands.remove_resource::<P2PSession<GGRSConfig>>();
	commands.remove_resource::<LocalHandles>();
	commands.remove_resource::<SessionType>();

}