use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkingPlugin, NetworkResource};
use std::net::{IpAddr, SocketAddr, Ipv4Addr};

pub struct NetworkServerRessource {
    host: String,
    port: i32
}

pub struct NetworkClientPlugin {}

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(NetworkingPlugin::default())
            //.add_startup_system(setup_client)
            .add_startup_system(shared::setup_network_channels);
    }
}

fn setup_client(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    info!("Connecting to {}...", socket_address);
    net.connect(socket_address);
}
