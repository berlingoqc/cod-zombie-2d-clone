use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkingPlugin, NetworkResource, NetworkEvent };
use std::net::{IpAddr, SocketAddr, Ipv4Addr};

use shared::{ClientMessage, ServerMessage};

pub struct NetworkServerRessource {
    host: String,
    port: i32
}

pub struct NetworkClientPlugin {}

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(NetworkingPlugin::default())
            .add_startup_system(shared::setup_network_channels)
            .add_startup_system(setup_client)
            //.add_system(read_server_message_channel)
            .add_system(network_events);
    }
}

fn setup_client(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(192, 168, 50, 19));
    let socket_address = SocketAddr::new(ip_address, 9001);
    info!("Connecting to {}...", socket_address);
    net.connect(socket_address);
}

fn network_events(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    info!("Connection successful");
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {
                info!("Unhandle event {:?}", event);
            }
        }
    }
}

fn read_server_message_channel(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>
) {

    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ServerMessage>() {
            println!("RECEIVE MESSAGE FROM SERVER");
            match message {
                ServerMessage::Welcome(your_player_id) => {

                    info!("Receive my entity : {:?}", your_player_id);
                }
            }
        }
    }

}
