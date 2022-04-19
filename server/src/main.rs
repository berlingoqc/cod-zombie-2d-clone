//use arugio_shared::{BallId, ClientMessage, Position, ServerMessage, TargetVelocity, Velocity};
use bevy::{
    app::ScheduleRunnerSettings,
    prelude::App,
    prelude::{Component, EventReader, ResMut},
    MinimalPlugins,
};
use bevy::{math::vec2, prelude::*};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, net::Ipv4Addr, net::SocketAddr, time::Duration};
use turbulence::message_channels::ChannelMessage;

#[derive(Component, Serialize, Deserialize)]
struct NetworkHandle(u32);

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            1000 / 30,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(shared::setup_network_channels)
        .add_startup_system(setup_server)
        .add_system(network_events)
        //.add_system(arugio_shared::update_velocity)
        //.add_system(arugio_shared::update_position)
        //.add_system(spawn_ball_system)
        //.add_system(unowned_ball_input)
        //.add_system_to_stage(CoreStage::PreUpdate, read_component_channel::<Position>)
        //.add_system_to_stage(
        //    CoreStage::PreUpdate,
        //    read_component_channel::<TargetVelocity>,
        //)
        .add_system_to_stage(CoreStage::PreUpdate, read_network_channels)
        .add_system_to_stage(CoreStage::PostUpdate, broadcast_changes)
        .run();
}

fn setup_server(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    net.listen(socket_address, None, None);
    println!("Listening...");
}

fn read_network_channels(mut net: ResMut<NetworkResource>) {
    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        /*while let Some(message) = channels.recv::<ClientMessage>() {
            println!("Received message: {:?}", message);
        }*/
    }
}

fn network_events(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
 //   unowned_balls: Query<(Entity, &BallId), Without<NetworkHandle>>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    println!("New connection handle: {:?}", &handle);

                    /*
                    let (entity, ball) = unowned_balls.iter().next().expect("No unowned balls");
                    cmd.entity(entity).insert(NetworkHandle(*handle));
                    net.send_message(*handle, ServerMessage::Welcome(*ball))
                        .expect("Could not send welcome");
                    */
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

fn spawn_ball_system(
    mut cmd: Commands,
//    unowned_balls: Query<&BallId, Without<NetworkHandle>>
) {
    /*let mut count = 0;
    let mut highest_id = 0;
    for ball in unowned_balls.iter() {
        count += 1;
        highest_id = highest_id.max(ball.0);
    }

    if count < 3 {
        cmd.spawn_bundle((
            BallId(highest_id + 1),
            Position(vec2(
                rand::random::<f32>() * 10.0 - 5.0,
                rand::random::<f32>() * 10.0 - 5.0,
            )),
            Velocity::default(),
            TargetVelocity::default(),
        ));

        println!("Spawned ball {:?}", highest_id + 1);
    }
    */
}

fn unowned_ball_input(
    //mut unowned_balls: Query<(&BallId, &mut TargetVelocity), Without<NetworkHandle>>,
) {
    /*
    for (_, mut target_velocity) in unowned_balls.iter_mut() {
        target_velocity.0.x = rand::random::<f32>() * 2.0 - 1.0;
        target_velocity.0.y = rand::random::<f32>() * 2.0 - 1.0;
    }
    */
}

fn broadcast_changes(
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

fn read_component_channel<C: Component + ChannelMessage>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
) {

}
