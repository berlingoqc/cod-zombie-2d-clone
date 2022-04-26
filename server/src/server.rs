use bevy::{
    app::ScheduleRunnerSettings,
    prelude::App,
    prelude::{Component, EventReader, ResMut},
    MinimalPlugins,
};
use bevy::{math::vec2, prelude::*};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};
use shared::{game::{ZombieGame, ZombiePlayer, ZombiePlayerInformation, EntityId}, ServerMessage};
use std::{collections::HashMap, net::IpAddr, net::Ipv4Addr, net::SocketAddr, time::Duration};
use turbulence::message_channels::ChannelMessage;

#[derive(Component, Serialize, Deserialize)]
struct NetworkHandle(u32);

pub fn setup_server(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(192, 168, 50, 19));
    let socket_address = SocketAddr::new(ip_address, 9001);
    net.listen(socket_address, Some(socket_address), Some(socket_address));
    println!("Listening...");
}

pub fn read_network_channels(mut net: ResMut<NetworkResource>) {
    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        /*while let Some(message) = channels.recv::<ClientMessage>() {
            println!("Received message: {:?}", message);
        }*/
    }
}

pub fn network_events(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,

    mut zombie_game: ResMut<ZombieGame>,
) {
    for event in network_event_reader.iter() {
        println!("{:?}", event);
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    println!("New connection handle: {:?}", &handle);


                    // Create the player entity
                    let player_entity = cmd.spawn()
                        .insert(ZombiePlayer{})
                        .insert(NetworkHandle(*handle))
                        .id().id();

                    let zombie_player = ZombiePlayerInformation{
                        handle: *handle,
                        entity: player_entity
                    };

                    zombie_game.players.push(zombie_player);


                    net.send_message(*handle, ServerMessage::Welcome(EntityId(player_entity)))
                        .expect("Could not send welcome");

                    println!("Send hello world");
                    
                    /*
                    let (entity, ball) = unowned_balls.iter().next().expect("No unowned balls");
                    cmd.entity(entity).insert(NetworkHandle(*handle));
                    net.send_message(*handle, ServerMessage::Welcome(*ball))
                        .expect("Could not send welcome");
                    */
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                println!("Disconnection from handle {:?}", handle);

            }
            _ => {}
        }
    }
}

pub fn broadcast_changes(
    mut net: ResMut<NetworkResource>,
) {
    /*
    for (ball_id, target_velocity) in changed_target_velocities.iter() {
        let _ = net.broadcast_message((*ball_id, *target_velocity));
    }

    for (ball_id, position) in changed_positions.iter() {
        let _ = net.broadcast_message((*ball_id, *position));
    }
    */
}

pub fn read_component_channel<C: Component + ChannelMessage>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
) {

}
